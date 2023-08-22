use std::fmt;



#[derive(Debug)]
pub enum Http2Error {

}


impl Http2Error {
    #[inline]
    pub fn description_str(&self) -> &'static str {
        match *self {
        }
    }
}

impl fmt::Display for Http2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description_str())
    }
}
