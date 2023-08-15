use std::{collections::HashMap, fmt};


#[derive(Hash)]
pub enum HeaderName {
    Stand(&'static str),
    Value(String),
}

#[derive(Hash)]
pub enum HeaderValue {
    Stand(&'static str),
    Value(Vec<u8>),
}

impl fmt::Debug for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("HeaderName");
        match &self {
            Self::Stand(name) => f.field("name", name),
            Self::Value(name) => f.field("name", name),
        };
        f.finish()
    }
}

pub struct HeaderMap {
    pub headers : HashMap<HeaderName, HeaderValue>,
}

impl HeaderMap {
    pub fn new() -> HeaderMap {
        HeaderMap { headers: HashMap::new() }
    }
}