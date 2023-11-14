use std::{
    ops::{Index, IndexMut}, fmt::Display, collections::HashMap,
};

use crate::{HeaderName, HeaderValue, WebError, WebResult, Buf, BufMut};



#[derive(Debug, PartialEq, Eq)]
pub struct HeaderMap {
    headers: Vec<(HeaderName, HeaderValue)>,
    systems: HashMap<String, String>,
}

impl HeaderMap {
    pub fn new() -> HeaderMap {
        HeaderMap {
            headers: Vec::new(),
            systems: HashMap::new(),
        }
    }

    pub fn iter(&self) ->  std::slice::Iter<(HeaderName, HeaderValue)> {
        self.headers.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<(HeaderName, HeaderValue)> {
        self.headers.iter_mut()
    }

    pub fn push<T, V>(&mut self, name: T, value: V) -> Option<HeaderValue>
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
        let (name, value) = (name.unwrap(), value.unwrap());
        for v in self.headers.iter_mut() {
            if v.0 == name {
                v.1.push(value);
                return None;
            }
        }
        self.headers.push((name, value));
        None
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
        let (name, value) = (name.unwrap(), value.unwrap());
        for v in self.headers.iter_mut() {
            if v.0 == name {
                v.1 = value;
                return None;
            }
        }
        self.headers.push((name, value));
        None
    }
    
    pub fn remove<T: AsRef<[u8]>>(&mut self, name: &T) -> Option<HeaderValue>
    {
        for i in 0..self.headers.len() {
            let v = &self.headers[i];
            if v.0 == name.as_ref() {
                self.headers.remove(i);
                return None
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.headers.clear()
    }
    
    pub fn contains<T: AsRef<[u8]>>(&self, name: &T) -> bool {
        for i in 0..self.headers.len() {
            let v = &self.headers[i];
            if &v.0 == &name.as_ref() {
                return true
            }
        }
        false
    }

    pub fn get_value(&self, name: &HeaderName) -> &HeaderValue {
        for i in 0..self.headers.len() {
            let v = &self.headers[i];
            if &v.0 == name {
                return &v.1
            }
        }
        unreachable!()
    }

    pub fn get_mut_value<'a>(&'a mut self, name: &HeaderName) -> &'a mut HeaderValue {
        for v in self.headers.iter_mut() {
            if &v.0 == name {
                return &mut v.1
            }
        }
        // for i in 0..self.headers.len() {
        //     let v = &mut self.headers[i];
        //     if &v.0 == name {
        //         return &mut v.1
        //     }
        // }
        unreachable!()
    }


    pub fn get_option_value<T: AsRef<[u8]>>(&self, name: &T) -> Option<&HeaderValue> {
        for i in 0..self.headers.len() {
            let v = &self.headers[i];
            if v.0 == name.as_ref() {
                return Some(&v.1)
            }
        }
        None
    }

    pub fn get_host(&self) -> Option<String> {
        if let Some(value) = self.get_option_value(&HeaderName::HOST) {
            value.try_into().ok()
        } else if let Some(value) = self.get_option_value(&":authority") {
            let value = TryInto::<String>::try_into(value).ok().unwrap();
            // host 信息只取前缀
            if value.contains(":") {
                let v: Vec<&str> = value.splitn(1, ':').collect();
                return Some(v[0].to_string())
            } else {
                return Some(value)
            }
        } else {
            None
        }
    }

    pub fn get_referer(&self) -> Option<String> {
        if let Some(value) = self.get_option_value(&HeaderName::REFERER) {
            value.try_into().ok()
        } else {
            None
        }
    }

    pub fn get_user_agent(&self) -> Option<String> {
        if let Some(value) = self.get_option_value(&HeaderName::USER_AGENT) {
            value.try_into().ok()
        } else {
            None
        }
    }

    pub fn get_cookie(&self) -> Option<String> {
        if let Some(value) = self.get_option_value(&HeaderName::COOKIE) {
            value.try_into().ok()
        } else {
            None
        }
    }


    pub fn get_body_len(&self) -> isize {
        // if self.headers.contains_key(&HeaderName::TRANSFER_ENCODING) {
        //     let value = &self.headers[&HeaderName::CONTENT_LENGTH];
        //     value.try_into().unwrap_or(0)
        // } else

        if let Some(value) = self.get_option_value(&HeaderName::CONTENT_LENGTH) {
            value.try_into().unwrap_or(0)
        } else {
            0
        }
    }

    pub fn is_keep_alive(&self) -> bool {

        if let Some(value) = self.get_option_value(&HeaderName::CONNECTION) {
            Self::contains_bytes(value.as_bytes(), b"Keep-Alive")
        } else {
            false
        }
    }
    
    pub fn is_chunked(&self) -> bool {
        if let Some(value) = self.get_option_value(&HeaderName::TRANSFER_ENCODING) {
            Self::contains_bytes(value.as_bytes(), b"chunked")
        } else {
            false
        }
    }

    pub fn get_upgrade_protocol(&self) -> Option<String> {

        if let Some(value) = self.get_option_value(&HeaderName::CONNECTION) {
            if !Self::contains_bytes(value.as_bytes(), b"Upgrade") {
                return None
            }
        } else {
            return None
        }

        if let Some(value) = self.get_option_value(&HeaderName::UPGRADE) {
            return value.as_string()
        } else {
            return None
        }
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.len() == 0
    }

    pub fn system_insert(&mut self, key: String, value: String) {
        self.systems.insert(key, value);
    }

    pub fn system_get(&self, key: &String) -> Option<&String> {
        self.systems.get(key)
    }
    
    pub fn encode<B: Buf+BufMut>(&self, buffer: &mut B) -> WebResult<usize> {
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
        for i in 0..(src.len() - dst.len() + 1) {
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
        self.get_value(&name)
    }
}

impl IndexMut<&'static str> for HeaderMap {
    fn index_mut(&mut self, index: &'static str) -> &mut Self::Output {
        let name = HeaderName::Stand(index);
        if self.contains(&name) {
            self.get_mut_value(&name)
        } else {
            self.insert(name, HeaderValue::Stand(""));
            self.get_mut_value(&HeaderName::Stand(index))
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
    type IntoIter = std::vec::IntoIter<(HeaderName, HeaderValue)>;

    fn into_iter(self) -> Self::IntoIter {
        self.headers.into_iter()
    }
}

impl Clone for HeaderMap {
    fn clone(&self) -> Self {
        Self {
            headers: self.headers.clone(),
            systems: self.systems.clone(),
        }
    }
}

impl Display for HeaderMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for v in &self.headers {
            v.0.fmt(f)?;
            f.write_str(": ")?;
            v.1.fmt(f)?;
            f.write_str("\r\n")?;
        }
        f.write_str("\r\n")
    }
}
