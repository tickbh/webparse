
// copy a large content from rust-websocket.

mod dataframe;
mod error;
pub mod frame_header;
mod message;
mod mask;

pub use dataframe::{DataFrame, Opcode, DataFrameable};
pub use error::WsError;
pub use frame_header::WsFrameHeader;
pub use message::{Message, OwnedMessage, CloseData, CloseCode};
pub use mask::Masker;