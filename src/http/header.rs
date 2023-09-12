use std::{
    borrow::Cow,
    collections::{hash_map::Iter, HashMap},
    fmt,
    hash::Hash,
    io::Write,
    ops::{Index, IndexMut},
};

use crate::{helper, BinaryMut, HeaderName, HeaderValue, Helper, Serialize, WebError, WebResult, Buf, BufMut, MarkBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct HeaderMap {
    headers: HashMap<HeaderName, HeaderValue>,
}

impl HeaderMap {
    pub fn new() -> HeaderMap {
        HeaderMap {
            headers: HashMap::new(),
        }
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<HeaderName, HeaderValue> {
        self.headers.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<HeaderName, HeaderValue> {
        self.headers.iter_mut()
    }

    pub fn insert_exact(&mut self, name: HeaderName, value: HeaderValue) -> Option<HeaderValue> {
        self.headers.insert(name, value)
    }

    pub fn insert<T, V>(&mut self, name: T, value: V) -> Option<HeaderValue>
    where
        HeaderName: TryFrom<T>,
        <HeaderName as TryFrom<T>>::Error: Into<WebError>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<WebError>,
    {
        let name = HeaderName::try_from(name).map_err(Into::into);
        let value = HeaderValue::try_from(value).map_err(Into::into);
        if name.is_err() || value.is_err() {
            return None;
        }
        self.insert_exact(name.unwrap(), value.unwrap())
    }

    pub fn clear(&mut self) {
        self.headers.clear()
    }

    pub fn contains(&self, name: &HeaderName) -> bool {
        self.headers.contains_key(name)
    }

    pub fn get_host(&self) -> Option<String> {
        for iter in &self.headers {
            println!("name = {:?}", iter.0);
        }
        if self.headers.contains_key(&HeaderName::HOST) {
            let value = &self.headers[&HeaderName::HOST];
            value.try_into().ok()
        } else {
            None
        }
    }

    pub fn get_body_len(&self) -> usize {
        // if self.headers.contains_key(&HeaderName::TRANSFER_ENCODING) {
        //     let value = &self.headers[&HeaderName::CONTENT_LENGTH];
        //     value.try_into().unwrap_or(0)
        // } else
        if self.headers.contains_key(&HeaderName::CONTENT_LENGTH) {
            let value = &self.headers[&HeaderName::CONTENT_LENGTH];
            value.try_into().unwrap_or(0)
        } else {
            0
        }
    }

    pub fn is_keep_alive(&self) -> bool {
        if self.headers.contains_key(&HeaderName::CONNECTION) {
            println!("contain!!!!");
            let value = &self.headers[&HeaderName::CONNECTION];
            Self::contains_bytes(value.as_bytes(), b"Keep-Alive")
        } else {
            false
        }
    }

    pub fn get_upgrade_protocol(&self) -> Option<String> {
        if !self.headers.contains_key(&HeaderName::CONNECTION) || !self.headers.contains_key(&HeaderName::UPGRADE) {
            return None;
        }
        println!("contain!!!!");
        let value = &self.headers[&HeaderName::CONNECTION];
        if !Self::contains_bytes(value.as_bytes(), b"Upgrade") {
            return None;
        }
        let value = &self.headers[&HeaderName::UPGRADE];
        value.as_string()
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.len() == 0
    }

    
    pub fn encode<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        let mut size = 0;
        for value in self.iter() {
            size += value.0.encode(buffer)?;
            size += buffer.put_slice(": ".as_bytes());
            size += value.1.encode(buffer)?;
            size += buffer.put_slice("\r\n".as_bytes());
        }
        size += buffer.put_slice("\r\n".as_bytes());
        Ok(size)
    }

    fn contains_bytes(src: &[u8], dst: &[u8]) -> bool {
        if dst.len() > src.len() {
            return false;
        }
        for i in 0..(src.len() - dst.len()) {
            if &src[i..(i + dst.len())] == dst {
                return true;
            }
        }
        false
    }
}

impl Index<&'static str> for HeaderMap {
    type Output = HeaderValue;

    fn index(&self, index: &'static str) -> &Self::Output {
        let name = HeaderName::Stand(index);
        &self.headers[&name]
    }
}

impl IndexMut<&'static str> for HeaderMap {
    fn index_mut(&mut self, index: &'static str) -> &mut Self::Output {
        let name = HeaderName::Stand(index);
        if self.headers.contains_key(&name) {
            self.headers.get_mut(&name).unwrap()
        } else {
            self.headers.insert(name, HeaderValue::Stand(""));
            self.headers.get_mut(&HeaderName::Stand(index)).unwrap()
        }
    }
}

// impl<'a> Iterator for &'a HeaderMap {
//     type Item = (&'a HeaderName, &'a HeaderValue);

//     fn next(&mut self) -> Option<Self::Item> {
//         self.headers.iter().next()
//     }
// }

// impl<'a> Iterator for &'a mut HeaderMap {
//     type Item = (&'a HeaderName, &'a mut HeaderValue);
//     fn next(&mut self) -> Option<Self::Item> {
//         self.headers.iter()
//     }
// }

impl IntoIterator for HeaderMap {
    type Item = (HeaderName, HeaderValue);
    type IntoIter = std::collections::hash_map::IntoIter<HeaderName, HeaderValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.headers.into_iter()
    }
}

impl Clone for HeaderMap {
    fn clone(&self) -> Self {
        Self {
            headers: self.headers.clone(),
        }
    }
}
