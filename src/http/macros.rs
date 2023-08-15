
macro_rules! byte_map {
    ($($flag:expr,)*) => ([
        $($flag != 0,)*
    ])
}

macro_rules! next {
    ($bytes:ident) => ({
        match $bytes.next() {
            Some(b) => Ok(b),
            None => Err(WebError::Partial)
        }
    })
}


macro_rules! expect {
    ($bytes:ident.next() == $pat:pat => $ret:expr) => {
        expect!(next!($bytes) => $pat |? $ret)
    };
    ($e:expr => $pat:pat_param |? $ret:expr) => {
        match $e {
            Ok(v@$pat) => v,
            _ => return $ret
        }
    };
}