mod storage;
mod release;

extern crate chrono;
extern crate env_logger;
extern crate kube;
extern crate k8s_openapi;
#[macro_use]
extern crate failure;
extern crate base64;
extern crate serde_yaml;
#[macro_use]
extern crate serde_json;
extern crate flate2;

use storage::driver::secrets::Secrets;
use release::Release;
use kube::config;
use kube::client::APIClient;
use std::collections::HashMap;
use serde_json::Value;
use storage::driver::Driver;

fn main() {
    env_logger::init();
    let config = config::load_kube_config().expect("failed to load kubeconfig");
    let client = APIClient::new(config);

    let sec = Secrets::new(client, "default".to_string());
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

    // Create example
    sec.create(&name, rel).unwrap();

    let mut updated_rel = sec.get(&name).unwrap();
    println!("{:?}", updated_rel);

    // Update example
    updated_rel.version = 2;
    updated_rel.manifest = "kind: Blah\napiVersion:bar".into();

    sec.update(&name, updated_rel).unwrap();

    let updated_rel = sec.get(&name).unwrap();
    println!("{:?}", updated_rel);

    // Delete example
    let rel = sec.delete(&name).unwrap();
    println!("{:?}", rel);
}
