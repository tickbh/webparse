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
// Created Date: 2023/08/15 09:59:05

#[macro_export]
macro_rules! byte_map {
    ($($flag:expr,)*) => ([
        $($flag != 0,)*
    ])
}

#[macro_export]
macro_rules! next {
    ($bytes:ident) => ({
        match $bytes.get_next() {
            Some(b) => Ok(b),
            None => Err(crate::WebError::from(crate::HttpError::Partial))
        }
    })
}

#[macro_export]
macro_rules! must_have {
    ($bytes:ident, $num:expr) => ({
        if $bytes.remaining() >= $num {
            Ok(())
        } else {
            Err(webparse::WebError::from(webparse::HttpError::Partial))
        }
    })
}


#[macro_export]
macro_rules! peek {
    ($bytes:ident) => ({
        match $bytes.peek() {
            Some(b) => Ok(b),
            None => Err(WebError::from(crate::HttpError::Partial))
        }
    })
}


#[macro_export]
macro_rules! expect {
    ($bytes:ident.next() == $pat:pat => $ret:expr) => {
        expect!(next!($bytes) => $pat |? $ret)
    };
    ($e:expr => $pat:pat_param |? $ret:expr) => {
        match $e {
            Ok(_v@$pat) => (),
            Err(e) => return Err(e),
            _ => return $ret
        }
    };
}


#[macro_export]
macro_rules! try_advance {
    ($flag:expr) => {
        if !$flag {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "not enough"));
        }
    };
}
