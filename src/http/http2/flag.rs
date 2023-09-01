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
            None => Err(()),
        }
    }

    pub fn load(flag: Flag) -> Flag {
        flag & Flag::ACK
    }

    pub fn ack() -> Flag {
        Flag::ACK
    }
    pub fn is_ack(&self) -> bool {
        self.contains(Flag::ACK)
    }
    pub fn end_stream() -> Flag {
        Flag::END_STREAM
    }
    pub fn is_end_stream(&self) -> bool {
        self.contains(Flag::END_STREAM)
    }
    pub fn end_headers() -> Flag {
        Flag::END_HEADERS
    }
    pub fn is_end_headers(&self) -> bool {
        self.contains(Flag::END_HEADERS)
    }
    pub fn padded() -> Flag {
        Flag::PADDED
    }
    pub fn is_padded(&self) -> bool {
        self.contains(Flag::PADDED)
    }
    pub fn priority() -> Flag {
        Flag::PRIORITY
    }
    pub fn is_priority(&self) -> bool {
        self.contains(Flag::PRIORITY)
    }
}

impl Default for Flag {
    fn default() -> Self {
        Self(Default::default())
    }
}
