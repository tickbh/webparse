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
