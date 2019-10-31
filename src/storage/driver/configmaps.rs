use crate::release::Release;
use crate::storage::driver::*;
use std::collections::HashMap;
use std::vec::Vec;
use kube::api::{Api, v1ConfigMap, ListParams, PostParams, PatchParams, DeleteParams};
use kube::client::APIClient;

pub struct ConfigMaps {
    client: Api<v1ConfigMap>,
}

impl ConfigMaps {
    pub fn new(client: APIClient, namespace: String) -> Self {
        ConfigMaps {
            client: Api::v1ConfigMap(client).within(&namespace)
        }
    }
}

impl ConfigMaps {
    fn get_cm_list<F>(&self, label_selector: Option<String>, filter: F) -> Result<Vec<Release>, DriverError> 
    where
        F: Fn(&Release) -> bool,
    {
        let mut lp = ListParams::default();
        lp.label_selector = label_selector;
        let mut res = self.client.list(&lp)?;
        let mut release_list: Vec<Release> = Vec::with_capacity(res.items.len());
        while let Some(cm) = res.items.pop() {
            // TODO: Change decoding errors to just log to match Helm behavior
            let rel = decode_release(cm.data)?;
            if filter(&rel) {
                release_list.push(rel);
            }
        }
        Ok(release_list)
    }

    fn generate_cm_data(key: &String, rel: Release, addl_labels: HashMap<String, String>) -> Result<Vec<u8>, DriverError> {
        let rel_data = encode_release(&rel)?;
        let mut data = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": key,
                "labels": {
                    "name": rel.name,
                    "owner": "helm",
                    "status": rel.info.status,
                    "version": rel.version.to_string()
                }
            },
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

impl Driver for ConfigMaps {
    fn name(&self) -> String {String::from("configmaps")}
    fn create(&self, key: &String, rel: Release) -> Result<(), DriverError> {
        let mut labels: HashMap<String, String> = HashMap::new();
        labels.insert("createdAt".to_string(), chrono::Utc::now().timestamp().to_string());
        let data = ConfigMaps::generate_cm_data(&key, rel, labels)?;
        self.client.create(&PostParams::default(), data)?;
        Ok(())
    }
    fn update(&self, key: &String, rel: Release) -> Result<(), DriverError> {
        let mut labels: HashMap<String, String> = HashMap::new();
        labels.insert("modifiedAt".to_string(), chrono::Utc::now().timestamp().to_string());
        let data = ConfigMaps::generate_cm_data(&key, rel, labels)?;
        self.client.patch(&key, &PatchParams::default(), data)?;
        Ok(())
    }
    fn delete(&self, key: &String) -> Result<Release, DriverError> {
        let rel = self.get(key)?;
        self.client.delete(key, &DeleteParams::default())?;
        Ok(rel)
    }
    fn get(&self, key: &String) -> Result<Release, DriverError> {
        let sec = self.client.get(key)?;
        let rel = decode_release(sec.data)?;
        Ok(rel)
    }
    fn list<F>(&self, filter: F) -> Result<Vec<Release>, DriverError> 
    where
        F: Fn(&Release) -> bool,
    {
        return self.get_cm_list(Some("owner=helm".to_string()), filter);
    }
    fn query(&self, labels: HashMap<String, String>) -> Result<Vec<Release>, DriverError> {
        // TODO: Actually check the label value is valid (hence why this is not just in the actual ListParams)
        let selector: String = labels.iter().map(|pair| format!("{}={}", pair.0, pair.1)).collect::<Vec<String>>().join(",");
        return self.get_cm_list(Some(selector), |_| true);
    }
}
