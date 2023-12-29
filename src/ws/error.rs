use crate::WebError;



#[derive(Debug)]
pub enum WsError {
    DataFrameError(&'static str),
    ProtocolError(&'static str),
    NoDataAvailable,
}

impl WsError {
    #[inline]
    pub fn description_str(&self) -> &'static str {
        match *self {
            Self::DataFrameError(s) => s,
            _ => "",
        }
    }

    pub fn into<E: Into<WsError>>(e: E) -> WebError {
        WebError::Ws(e.into())
    }
}
