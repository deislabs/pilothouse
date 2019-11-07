mod storage;
mod release;
mod kube;

extern crate chrono;
extern crate env_logger;
extern crate kube as external_kube;
extern crate k8s_openapi;
#[macro_use]
extern crate failure;
extern crate base64;
extern crate serde_yaml;
extern crate log;
#[macro_use]
extern crate serde_json;
extern crate flate2;
extern crate reqwest;

use storage::driver::secrets::Secrets;
use storage::driver::configmaps::ConfigMaps;
use release::Release;
use external_kube::config;
use external_kube::client::APIClient;
use std::collections::HashMap;
use serde_json::Value;
use log::{info, debug, error};
use storage::{Storage, MaxHistory};
use crate::kube::client::Client;

fn main() {
    env_logger::init();
    let config = config::load_kube_config().expect("failed to load kubeconfig");
    let client = APIClient::new(config);

    let store = Storage::new(Secrets::new(client.clone(), "default".to_string()), MaxHistory::NoLimit);
    let name = "hello".to_string();
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("foo".into(), "bar".into());
    let rel = Release {
        name: name.clone(),
        config: values,
        manifest: "kind: Foo\napiVersion:bar".into(),
        version: 1,
        namespace: "default".into(),
        ..Default::default()
    };

    // Create secret example
    store.create(rel).unwrap();

    let mut updated_rel = store.get(&name, &1).unwrap();
    println!("{:?}", updated_rel);

    // Update secret example
    updated_rel.manifest = "kind: Blah\napiVersion:bar".into();

    store.update(updated_rel).unwrap();

    let mut updated_rel = store.get(&name, &1).unwrap();
    println!("{:?}", updated_rel);

    // new release secret example
    updated_rel.version = 2;
    updated_rel.manifest = "kind: Last\napiVersion:bar".into();

    store.create(updated_rel).unwrap();

    let updated_rel = store.get(&name, &1).unwrap();
    println!("{:?}", updated_rel);

    // Delete secrets example
    let rel = store.delete(&name, &updated_rel.version).unwrap();
    println!("{:?}", rel);

    let rel = store.delete(&name, &2).unwrap();
    println!("{:?}", rel);

    // ConfigMap examples
    let store = Storage::new(ConfigMaps::new(client.clone(), "default".to_string()), MaxHistory::NoLimit);
    let name = "hello".to_string();
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("foo".into(), "bar".into());
    let rel = Release {
        name: name.clone(),
        config: values,
        manifest: "kind: Foo\napiVersion:bar".into(),
        version: 1,
        namespace: "default".into(),
        ..Default::default()
    };

    // Create ConfigMap example
    store.create(rel).unwrap();

    let mut updated_rel = store.get(&name, &1).unwrap();
    println!("{:?}", updated_rel);

    // Update configmap example
    updated_rel.manifest = "kind: Blah\napiVersion:bar".into();

    store.update(updated_rel).unwrap();

    let mut updated_rel = store.get(&name, &1).unwrap();
    println!("{:?}", updated_rel);

    // new release configmap example
    updated_rel.version = 2;
    updated_rel.manifest = "kind: Last\napiVersion:bar".into();

    store.create(updated_rel).unwrap();

    let updated_rel = store.get(&name, &2).unwrap();
    println!("{:?}", updated_rel);

    // Delete configmaps example
    let rel = store.delete(&name, &updated_rel.version).unwrap();
    println!("{:?}", rel);

    let rel = store.delete(&name, &1).unwrap();
    println!("{:?}", rel);
}
