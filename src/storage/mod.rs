pub mod driver;

use crate::release::{Release, Status};
use crate::release::sort::*;
use driver::{Driver, DriverError};
use log::{info, debug, error};
use std::collections::HashMap;

pub const HELM_STORAGE_TYPE: &str = "sh.helm.release.v1";

pub enum MaxHistory {
    NoLimit,
    Limit(usize)
}

pub struct Storage<T> {
    pub driver: T,
    max_history: MaxHistory
}

impl<T: Driver> Storage<T> {
    pub fn new(d: T, max: MaxHistory) -> Self {
        Storage {
            driver: d,
            max_history: max
        }
    }

    fn make_key(&self, release_name: &str, version: &usize) -> String {
        return format!("{}.{}.v{}", HELM_STORAGE_TYPE, release_name, version);
    }

    fn remove_least_recent(&self, release_name: &str) -> Result<(), DriverError> {
        let max = match self.max_history {
            MaxHistory::NoLimit => { return Ok(()) },
            MaxHistory::Limit(n) => n
        };
        let mut rels = self.history(release_name)?;
        if rels.len() <= max {
            return Ok(())
        }
        let overage = rels.len() - max;
        rels.sort_unstable_by(revision);
        // Delete as many as possible. In the case of API throughput
        // limitations, multiple invocations of this function will eventually
        // delete them all, so only log an error
        for r in rels.iter().take(overage) {
            self.driver.delete(&self.make_key(release_name, &r.version)).unwrap_or_else(|e| {
                error!("unable to delete old release {}: {}", release_name, e);
                Release::default()    
            });
        }

        Ok(())
    }

    // TODO: These methods probably shouldn't return a driver error. Take care
    // of this when refactoring the error code
    pub fn get(&self, release_name: &str, version: &usize) -> Result<Release, DriverError> {
        debug!("getting release {}", release_name);
        return self.driver.get(&self.make_key(release_name, version))
    }

    pub fn create(&self, rel: Release) -> Result<(), DriverError> {
        debug!("creating release {}", rel.name);
        self.remove_least_recent(&rel.name)?;
        return self.driver.create(&self.make_key(&rel.name, &rel.version), rel)
    }

    pub fn update(&self, rel: Release) -> Result<(), DriverError> {
        debug!("updating release {}", rel.name);
        return self.driver.update(&self.make_key(&rel.name, &rel.version), rel)
    }

    pub fn delete(&self, release_name: &str, version: &usize) -> Result<Release, DriverError> {
        debug!("deleting release {}", release_name);
        return self.driver.delete(&self.make_key(release_name, version))
    }

    // These are helpful shorthand methods for the most common listing
    // operations. For listing with a custom filter, use the exposed driver list
    // method
    pub fn list_all(&self) -> Result<Vec<Release>, DriverError> {
        debug!("listing all releases in {} storage", self.driver.name());
        return self.driver.list(|_| return true)
    }

    pub fn list_uninstalled(&self) -> Result<Vec<Release>, DriverError> {
        debug!("listing all uninstalled releases in {} storage", self.driver.name());
        return self.driver.list(|rel| return rel.info.status == Status::Uninstalled)
    }

    pub fn list_deployed(&self) -> Result<Vec<Release>, DriverError> {
        debug!("listing all deployed releases in {} storage", self.driver.name());
        return self.driver.list(|rel| return rel.info.status == Status::Deployed)
    }

    pub fn history(&self, release_name: &str) -> Result<Vec<Release>, DriverError> {
        debug!("getting history for {} ", release_name);
        let mut query_labels: HashMap<String, String> = HashMap::new();
        query_labels.insert("name".into(), release_name.into());
        query_labels.insert("owner".into(), "helm".into());

        return self.driver.query(query_labels);
    }

    // Helper methods for getting and listing specific releases (instead of all)
    pub fn get_all_deployed(&self, release_name: &str) -> Result<Vec<Release>, DriverError> {
        debug!("listing all deployed releases for {} ", release_name);
        let mut query_labels: HashMap<String, String> = HashMap::new();
        query_labels.insert("name".into(), release_name.into());
        query_labels.insert("owner".into(), "helm".into());
        query_labels.insert("status".into(), Status::Deployed.to_string());

        return self.driver.query(query_labels);
    }

    // returns the last release with a deployed state
    pub fn last_deployed(&self, release_name: &str) -> Result<Release, DriverError> {
        let mut all = self.get_all_deployed(release_name)?;
        if all.is_empty() {
            return Err(DriverError::ReleaseNotExist)
        }
        all.sort_unstable_by(revision);
        // Pop removes from the end of the sorted vector, so it should be the latest
        Ok(all.pop().unwrap())
    }

    // Returns the last release, regardless of status
    pub fn last(&self, release_name: &str) -> Result<Release, DriverError> {
        let mut all = self.history(release_name)?;
        if all.is_empty() {
            return Err(DriverError::ReleaseNotExist)
        }
        all.sort_unstable_by(revision);
        // Pop removes from the end of the sorted vector, so it should be the latest
        Ok(all.pop().unwrap())
    }
}
