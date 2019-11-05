// This module contains common sorting functions for releases
use std::cmp::Ordering;
use crate::release::Release;

pub fn revision(a: &Release, b: &Release) -> Ordering {
    a.version.cmp(&b.version)
}
