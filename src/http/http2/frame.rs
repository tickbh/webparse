use super::{Kind, Payload, StreamIdentifier, Flag};





#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameHeader {
    pub length: u32,
    pub kind: Kind,
    pub flag: Flag,
    pub id: StreamIdentifier,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Frame<'a> {
    pub header: FrameHeader,
    pub payload: Payload<'a>
}