use external_kube::config::{ConfigOptions, Context, Cluster, AuthInfo, Configuration};
use external_kube::client::APIClient;
// use std::error::Error;
use failure::Error;
use reqwest::ClientBuilder;

// This is a reimplementation of the private KubeConfigLoader that returns from
// create_client_builder
pub struct KubeConfigLoader {
    pub current_context: Context,
    pub cluster: Cluster,
    pub user: AuthInfo,
}

pub struct Client {
    config_loader: KubeConfigLoader,
    config: Configuration,
}

impl Client {
    pub fn new(config_loader: Option<(ClientBuilder, KubeConfigLoader)>) -> Result<Self, Error> {
        let cl = match config_loader {
            Some(c) => c,
            None => {
                let cl = external_kube::config::create_client_builder(ConfigOptions::default())?;
                (cl.0, KubeConfigLoader{
                    current_context: cl.1.current_context,
                    cluster: cl.1.cluster,
                    user: cl.1.user
                })
            }
        };
        let server = &cl.1.cluster.server.clone();
        Ok(Client{
            config_loader: cl.1,
            config: Configuration::new(server.clone(), cl.0.build()?)
        })
    }

    fn namespace(&self) -> String {
        self.config_loader.current_context.namespace.as_ref().unwrap_or(&"default".to_string()).clone()
    }

    fn clientset(&self) -> APIClient {
        APIClient::new(self.config.clone())
    }
}
