use std::{
    borrow::Cow,
    collections::{hash_map::Iter, HashMap},
    fmt,
    hash::Hash,
    io::Write,
    ops::{Index, IndexMut},
};

use crate::{helper, BinaryMut, HeaderName, HeaderValue, Helper, Serialize, WebError, WebResult};

#[derive(Debug)]
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

    pub fn len(&self) -> usize {
        self.headers.len()
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

impl Serialize for HeaderMap {
    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        Err(WebError::Serialize("header map can't call header map"))
    }

    fn serialize(&self, buffer: &mut BinaryMut) -> WebResult<()> {
        for value in self.iter() {
            value.0.serialize(buffer)?;
            buffer.write(": ".as_bytes()).map_err(WebError::from)?;
            value.1.serialize(buffer)?;
            buffer.write("\r\n".as_bytes()).map_err(WebError::from)?;
        }
        buffer.write("\r\n".as_bytes()).map_err(WebError::from)?;
        Ok(())
    }
}
