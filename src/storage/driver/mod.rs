pub mod secrets;
pub mod configmaps;

use crate::release::Release;
use std::collections::HashMap;
use std::vec::Vec;
use std::error::Error;
use flate2::write::{GzEncoder, GzDecoder};
use flate2::Compression;
use std::io::prelude::*;

// TODO: Expand out to a full enum of all driver errors
#[derive(Debug, Fail)]
pub enum DriverError {
    #[fail(display = "unable to perform kubernetes operation")]
    KubeError(#[fail(cause)] kube::Error),
    #[fail(display = "unable to decode release: {}", message)]
    DecodeError {
        message: String,
    },
    // TODO: Figure out how to distinguish between encode and decode error
    #[fail(display = "unable to encode release: {}", message)]
    EncodeError {
        message: String,
    },
    #[fail(display = "unable to decode release data due to invalid or missing data: {}", message)]
    InvalidData {
        message: String,
    },
    #[fail(display = "release was not found")]
    ReleaseNotExist,
    #[fail(display = "release already exists")]
    ReleaseAlreadyExists,
    #[fail(display = "storage object has malformed data")]
    MalformedData,
    #[fail(display = "client is out of sync with server and/or has stale data")]
    OutOfSync
}

impl From<kube::Error> for DriverError {
    fn from(error: kube::Error) -> Self {
        let apierr = match error.api_error() {
            Some(e) => e,
            None => { return DriverError::KubeError(error) }
        };
        // Comes from list of reasons exposed by k8s. See: https://godoc.org/k8s.io/apimachinery/pkg/apis/meta/v1#StatusReason
        return match apierr.reason.as_str() {
            "AlreadyExists" => DriverError::ReleaseAlreadyExists,
            "NotFound" => DriverError::ReleaseNotExist,
            "Invalid" => DriverError::MalformedData,
            "Conflict" | "Gone" => DriverError::OutOfSync,
            _ => DriverError::KubeError(error)
        };
    }
}

impl From<base64::DecodeError> for DriverError {
    fn from(error: base64::DecodeError) -> Self {
        DriverError::DecodeError{
            message: error.description().to_string()
        }
    }
}

impl From<serde_json::Error> for DriverError {
    fn from(error: serde_json::Error) -> Self {
        DriverError::DecodeError{
            message: error.description().to_string()
        }
    }
}

impl From<std::io::Error> for DriverError {
    fn from(error: std::io::Error) -> Self {
        DriverError::DecodeError{
            message: error.description().to_string()
        }
    }
}

pub trait Driver {
    fn name(&self) -> String;
    fn create(&self, key: &String, rel: Release) -> Result<(), DriverError>;
    fn update(&self, key: &String, rel: Release) -> Result<(), DriverError>;
    fn delete(&self, key: &String) -> Result<Release, DriverError>;
    fn get(&self, key: &String) -> Result<Release, DriverError>;
    fn list<F>(&self, filter: F) -> Result<Vec<Release>, DriverError>
    where
        F: Fn(&Release) -> bool;
    fn query(&self, labels: HashMap<String, String>) -> Result<Vec<Release>, DriverError>;
}

fn decode_release(raw: Vec<u8>) -> Result<Release, DriverError> {
    let mut decoder = GzDecoder::new(Vec::new());
    decoder.write_all(&raw)?;
    let buffer = decoder.finish()?;
    let rel: Release = serde_json::from_slice(&buffer)?;
    Ok(rel)
}

fn encode_release(rel: &Release) -> Result<String, DriverError> {
    let enc = serde_json::to_vec(rel)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&enc)?;
    let buffer = encoder.finish()?;
    Ok(base64::encode(&buffer))
}
