use crate::release::Release;
use crate::storage::driver::*;
use std::collections::{HashMap, BTreeMap};
use std::vec::Vec;
use kube::api::{Api, v1Secret, ListParams, PostParams, PatchParams, DeleteParams};
use kube::client::APIClient;
use k8s_openapi::ByteString;

pub struct Secrets {
    client: Api<v1Secret>,
}

impl Secrets {
    pub fn new(client: APIClient, namespace: String) -> Self {
        Secrets {
            client: Api::v1Secret(client).within(&namespace)
        }
    }
}

impl Secrets {
    fn get_secret_list<F>(&self, label_selector: Option<String>, filter: F) -> Result<Vec<Release>, DriverError> 
    where
        F: Fn(&Release) -> bool,
    {
        let mut lp = ListParams::default();
        lp.label_selector = label_selector;
        let mut res = self.client.list(&lp)?;
        let mut release_list: Vec<Release> = Vec::with_capacity(res.items.len());
        while let Some(sec) = res.items.pop() {
            // TODO: Change decoding errors to just log to match Helm behavior
            let rel = decode_release(Secrets::get_raw_data(sec.data)?)?;
            if filter(&rel) {
                release_list.push(rel);
            }
        }
        Ok(release_list)
    }
    
    fn get_raw_data(data: BTreeMap<String, ByteString>) -> Result<Vec<u8>, DriverError> {
        let raw = match data.get("release") {
            Some(b) => b,
            None => { return Err(DriverError::InvalidData{message: "no 'release' key found".to_string()}) }
        };
        Ok(raw.0.clone())
    }

    fn generate_secret_data(key: &String, rel: Release, addl_labels: HashMap<String, String>) -> Result<Vec<u8>, DriverError> {
        let rel_data = encode_release(&rel)?;
        let mut data = json!({
            "apiVersion": "v1",
            "kind": "Secret",
            "metadata": {
                "name": key,
                "labels": {
                    "name": rel.name,
                    "owner": "helm",
                    "status": rel.info.status,
                    "version": rel.version.to_string()
                }
            },
            "type": "helm.sh/release.v1",
            "data": {
                "release": rel_data
            }
        });
        for (k, v) in addl_labels.iter() {
            data["metadata"]["labels"][k] = v.clone().into()
        }
        let bytes = serde_json::to_vec(&data)?;
        Ok(bytes)
    }
}

impl Driver for Secrets {
    fn name(&self) -> String {String::from("secrets")}
    fn create(&self, key: &String, rel: Release) -> Result<(), DriverError> {
        let mut labels: HashMap<String, String> = HashMap::new();
        labels.insert("createdAt".to_string(), chrono::Utc::now().timestamp().to_string());
        let data = Secrets::generate_secret_data(&key, rel, labels)?;
        self.client.create(&PostParams::default(), data)?;
        Ok(())
    }
    fn update(&self, key: &String, rel: Release) -> Result<(), DriverError> {
        let mut labels: HashMap<String, String> = HashMap::new();
        labels.insert("modifiedAt".to_string(), chrono::Utc::now().timestamp().to_string());
        let data = Secrets::generate_secret_data(&key, rel, labels)?;
        self.client.patch(&key, &PatchParams::default(), data)?;
        Ok(())
    }
    fn delete(&self, key: &String) -> Result<Release, DriverError> {
        let rel = self.get(key)?;
        self.client.delete(key, &DeleteParams::default())?;

        // For some reason, this block of code doesn't return the secret, just
        // the status

        // let sec = match thing.left() {
        //     Some(s) => s,
        //     None => { return Err(DriverError::ReleaseNotExist) }
        // };
        // let rel = decode_release(sec.data)?;
        Ok(rel)
    }
    fn get(&self, key: &String) -> Result<Release, DriverError> {
        let sec = self.client.get(key)?;
        let rel = decode_release(Secrets::get_raw_data(sec.data)?)?;
        Ok(rel)
    }
    fn list<F>(&self, filter: F) -> Result<Vec<Release>, DriverError> 
    where
        F: Fn(&Release) -> bool,
    {
        return self.get_secret_list(Some("owner=helm".to_string()), filter);
    }
    fn query(&self, labels: HashMap<String, String>) -> Result<Vec<Release>, DriverError> {
        // TODO: Actually check the label value is valid (hence why this is not just in the actual ListParams)
        let selector: String = labels.iter().map(|pair| format!("{}={}", pair.0, pair.1)).collect::<Vec<String>>().join(",");
        return self.get_secret_list(Some(selector), |_| true);
    }
}
