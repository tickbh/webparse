use crate::{HeaderMap, Version};

use super::StatusCode;

#[derive(Debug)]
pub struct Response {
    parts: Parts,
    body: Vec<u8>,
    partial: bool,
}

#[derive(Debug)]
pub struct Parts {
    pub status: StatusCode,
    pub header: HeaderMap,
    pub version: Version,
    pub path: String,
}