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
// Created Date: 2023/08/21 06:16:49

use std::fmt;



#[derive(Debug)]
pub enum UrlError {
    UrlInvalid,
    UrlCodeInvalid,
}


impl UrlError {
    #[inline]
    pub fn description_str(&self) -> &'static str {
        match self {
            UrlError::UrlInvalid => "invalid Url",
            UrlError::UrlCodeInvalid => "invalid Url Code",
        }
    }
}


impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description_str())
    }
}
