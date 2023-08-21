bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Flag: u8 {
        const END_STREAM = 0x1;
        const ACK = 0x1;
        const END_HEADERS = 0x4;
        const PADDED = 0x8;
        const PRIORITY = 0x20;
    }
}

impl Flag {
    pub fn new(data: u8) -> Result<Flag, ()> {
        match Flag::from_bits(data) {
            Some(v) => Ok(v),
            None => Err(())
        }
    }

    pub fn ack() -> Flag { Flag::ACK }
    pub fn end_stream() -> Flag { Flag::END_STREAM }
    pub fn end_headers() -> Flag { Flag::END_HEADERS }
    pub fn padded() -> Flag { Flag::PADDED }
    pub fn priority() -> Flag { Flag::PRIORITY }
}
