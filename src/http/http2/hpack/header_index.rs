// Copyright 2022 - 2023 Wenmeng See the COPYRIGHT
// file at the top-level directory of this distribution.
// 
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
// 
// Author: tickbh
// -----
// Created Date: 2023/08/22 11:37:23

use crate::{http2::DEFAULT_SETTINGS_HEADER_TABLE_SIZE, HeaderName, HeaderValue};
use lazy_static::lazy_static;
use std::collections::{vec_deque, HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct HeaderIndex {
    table: VecDeque<(HeaderName, HeaderValue)>,
    size: usize,
    max_size: usize,
}

/// An `Iterator` through elements of the `DynamicTable`.
///
/// The implementation of the iterator itself is very tightly coupled
/// to the implementation of the `DynamicTable`.
///
/// This iterator returns tuples of slices. The tuples themselves are
/// constructed as new instances, containing a borrow from the `Vec`s
/// representing the underlying Headers.
struct HeaderIndexIter<'a> {
    /// Stores an iterator through the underlying structure that the
    /// `DynamicTable` uses
    inner: vec_deque::Iter<'a, (HeaderName, HeaderValue)>,
}

impl<'a> Iterator for HeaderIndexIter<'a> {
    type Item = (&'a HeaderName, &'a HeaderValue);

    fn next(&mut self) -> Option<(&'a HeaderName, &'a HeaderValue)> {
        match self.inner.next() {
            Some(ref header) => Some((&header.0, &header.1)),
            None => None,
        }
    }
}

impl HeaderIndex {
    pub fn new() -> HeaderIndex {
        HeaderIndex::with_size(DEFAULT_SETTINGS_HEADER_TABLE_SIZE as usize)
    }

    pub fn get_from_index(&self, index: usize) -> Option<(&HeaderName, &HeaderValue)> {
        let real_index = if index > 0 { index - 1 } else { return None };

        if real_index < STATIC_TABLE.len() {
            let v = &STATIC_TABLE[real_index];
            Some((&v.0, &v.1))
        } else {
            // Maybe it's in the dynamic table then?
            let dynamic_index = real_index - STATIC_TABLE.len();
            if dynamic_index < self.len() {
                match self.get(dynamic_index) {
                    Some(&(ref name, ref value)) => Some((name, value)),
                    None => None,
                }
            } else {
                None
            }
        }
    }

    pub fn find_header(&self, header: (&HeaderName, &HeaderValue)) -> Option<(usize, bool)> {
        if STATIC_HASH.contains_key(header.0) {
            let v = &STATIC_HASH[header.0];
            if v.contains_key(header.1) {
                return Some((v[header.1], true));
            } else if v.contains_key(&EMPTY_HEADER_VALUE) {
                return Some((v[&EMPTY_HEADER_VALUE], false));
            }
        } else {
            for (idx, value) in self.iter().enumerate() {
                if value.0 == header.0 && value.1 == header.1 {
                    return Some((idx + 1 + STATIC_TABLE.len(), true));
                }
            }
        }
        None
    }

    fn with_size(max_size: usize) -> HeaderIndex {
        HeaderIndex {
            table: VecDeque::new(),
            size: 0,
            max_size,
        }
    }

    /// Returns the current size of the table in octets, as defined by the IETF
    /// HPACK spec.
    pub fn get_size(&self) -> usize {
        self.size
    }

    fn iter(&self) -> HeaderIndexIter {
        HeaderIndexIter {
            inner: self.table.iter(),
        }
    }

    pub fn set_max_table_size(&mut self, new_max_size: usize) {
        self.max_size = new_max_size;
        // Make the table size fit within the new constraints.
        self.consolidate_table();
    }

    /// Returns the maximum size of the table in octets.
    pub fn get_max_table_size(&self) -> usize {
        self.max_size
    }

    pub fn add_header(&mut self, name: HeaderName, value: HeaderValue) {
        self.size += name.bytes_len() + value.bytes_len() + 32;
        // debug!("New dynamic table size {}", self.size);
        // Now add it to the internal buffer
        self.table.push_front((name, value));
        // ...and make sure we're not over the maximum size.
        self.consolidate_table();
        // debug!("After consolidation dynamic table size {}", self.size);
    }

    /// Consolidates the table entries so that the table size is below the
    /// maximum allowed size, by evicting headers from the table in a FIFO
    /// fashion.
    fn consolidate_table(&mut self) {
        while self.size > self.max_size {
            {
                let last_header = match self.table.back() {
                    Some(x) => x,
                    None => {
                        // Can never happen as the size of the table must reach
                        // 0 by the time we've exhausted all elements.
                        panic!("Size of table != 0, but no headers left!");
                    }
                };
                self.size -= last_header.0.bytes_len() + last_header.1.bytes_len() + 32;
            }
            self.table.pop_back();
        }
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Converts the current state of the table to a `Vec`
    pub fn to_vec(&self) -> Vec<(HeaderName, HeaderValue)> {
        let mut ret: Vec<(HeaderName, HeaderValue)> = Vec::new();
        for elem in self.table.iter() {
            ret.push(elem.clone());
        }
        ret
    }

    fn get(&self, index: usize) -> Option<&(HeaderName, HeaderValue)> {
        self.table.get(index)
    }
}

/// (HPACK, Appendix A)
static STATIC_TABLE_RAW: &'static [(&'static str, &'static str)] = &[
    (":authority", ""),
    (":method", "GET"),
    (":method", "POST"),
    (":path", "/"),
    (":path", "/index.html"),
    (":scheme", "http"),
    (":scheme", "https"),
    (":status", "200"),
    (":status", "204"),
    (":status", "206"),
    (":status", "304"),
    (":status", "400"),
    (":status", "404"),
    (":status", "500"),
    ("accept-", ""),
    ("accept-encoding", "gzip, deflate"),
    ("accept-language", ""),
    ("accept-ranges", ""),
    ("accept", ""),
    ("access-control-allow-origin", ""),
    ("age", ""),
    ("allow", ""),
    ("authorization", ""),
    ("cache-control", ""),
    ("content-disposition", ""),
    ("content-encoding", ""),
    ("content-language", ""),
    ("content-length", ""),
    ("content-location", ""),
    ("content-range", ""),
    ("content-type", ""),
    ("cookie", ""),
    ("date", ""),
    ("etag", ""),
    ("expect", ""),
    ("expires", ""),
    ("from", ""),
    ("host", ""),
    ("if-match", ""),
    ("if-modified-since", ""),
    ("if-none-match", ""),
    ("if-range", ""),
    ("if-unmodified-since", ""),
    ("last-modified", ""),
    ("link", ""),
    ("location", ""),
    ("max-forwards", ""),
    ("proxy-authenticate", ""),
    ("proxy-authorization", ""),
    ("range", ""),
    ("referer", ""),
    ("refresh", ""),
    ("retry-after", ""),
    ("server", ""),
    ("set-cookie", ""),
    ("strict-transport-security", ""),
    ("transfer-encoding", ""),
    ("user-agent", ""),
    ("vary", ""),
    ("via", ""),
    ("www-authenticate", ""),
];

lazy_static! {
    static ref STATIC_TABLE: Vec<(HeaderName, HeaderValue)> = {
        let mut m = Vec::<(HeaderName, HeaderValue)>::new();
        for &(code, code_val) in STATIC_TABLE_RAW.iter() {
            m.push((
                HeaderName::try_from(code).unwrap(),
                HeaderValue::from_static(code_val),
            ));
        }
        m
    };
    static ref STATIC_HASH: HashMap<HeaderName, HashMap<HeaderValue, usize>> = {
        let mut h = HashMap::<HeaderName, HashMap<HeaderValue, usize>>::new();
        for (idx, &(code, code_val)) in STATIC_TABLE_RAW.iter().enumerate() {
            let name = HeaderName::from_static(code);
            let value = HeaderValue::from_static(code_val);
            if !h.contains_key(&name) {
                let mut v = HashMap::new();
                v.insert(value, idx + 1);
                h.insert(name, v);
            } else {
                h.entry(name).and_modify(|v| {
                    v.insert(value, idx + 1);
                });
            }
        }
        h
    };
    static ref EMPTY_HEADER_VALUE: HeaderValue = HeaderValue::Value(vec![]);
}
