use std::borrow::Cow;
use std::io;
use std::io::Write;
use std::str::from_utf8;

use crate::{
    ws::{DataFrame, DataFrameable, Opcode, WsError},
    Buf, BufMut, WebError, WebResult,
};

const FALSE_RESERVED_BITS: &[bool; 3] = &[false; 3];

/// Valid types of messages (in the default implementation)
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Type {
    /// Message with UTF8 test
    Text = 1,
    /// Message containing binary data
    Binary = 2,
    /// Ping message with data
    Ping = 9,
    /// Pong message with data
    Pong = 10,
    /// Close connection message with optional reason
    Close = 8,
}

/// Represents a WebSocket message.
///
/// This message also has the ability to not own its payload, and stores its entire payload in
/// chunks that get written in order when the message gets sent. This makes the `write_payload`
/// allocate less memory than the `payload` method (which creates a new buffer every time).
///
/// Incidentally this (the default implementation of `Message`) implements the `DataFrame` trait
/// because this message just gets sent as one single `DataFrame`.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Message<'a> {
    /// Type of WebSocket message
    pub opcode: Type,
    /// Optional status code to send when closing a connection.
    /// (only used if this message is of Type::Close)
    pub cd_status_code: Option<u16>,
    /// Main payload
    pub payload: Cow<'a, [u8]>,
}

impl<'a> Message<'a> {
    fn new(code: Type, status: Option<u16>, payload: Cow<'a, [u8]>) -> Self {
        Message {
            opcode: code,
            cd_status_code: status,
            payload,
        }
    }

    /// Create a new WebSocket message with text data
    pub fn text<S>(data: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Message::new(
            Type::Text,
            None,
            match data.into() {
                Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
                Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
            },
        )
    }

    /// Create a new WebSocket message with binary data
    pub fn binary<B>(data: B) -> Self
    where
        B: IntoCowBytes<'a>,
    {
        Message::new(Type::Binary, None, data.into())
    }

    /// Create a new WebSocket message that signals the end of a WebSocket
    /// connection, although messages can still be sent after sending this
    pub fn close() -> Self {
        Message::new(Type::Close, None, Cow::Borrowed(&[0 as u8; 0]))
    }

    /// Create a new WebSocket message that signals the end of a WebSocket
    /// connection and provide a text reason and a status code for why.
    /// Messages can still be sent after sending this message.
    pub fn close_because<S>(code: u16, reason: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Message::new(
            Type::Close,
            Some(code),
            match reason.into() {
                Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
                Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
            },
        )
    }

    /// Create a ping WebSocket message, a pong is usually sent back
    /// after sending this with the same data
    pub fn ping<P>(data: P) -> Self
    where
        P: IntoCowBytes<'a>,
    {
        Message::new(Type::Ping, None, data.into())
    }

    /// Create a pong WebSocket message, usually a response to a
    /// ping message
    pub fn pong<P>(data: P) -> Self
    where
        P: IntoCowBytes<'a>,
    {
        Message::new(Type::Pong, None, data.into())
    }

    // TODO: change this to match conventions
    #[allow(clippy::wrong_self_convention)]
    /// Convert a ping message to a pong, keeping the data.
    /// This will fail if the original message is not a ping.
    pub fn into_pong(&mut self) -> Result<(), ()> {
        if self.opcode == Type::Ping {
            self.opcode = Type::Pong;
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<'a> DataFrameable for Message<'a> {
    #[inline(always)]
    fn is_last(&self) -> bool {
        true
    }

    #[inline(always)]
    fn opcode(&self) -> u8 {
        self.opcode as u8
    }

    #[inline(always)]
    fn reserved(&self) -> &[bool; 3] {
        FALSE_RESERVED_BITS
    }

    fn size(&self) -> usize {
        self.payload.len() + if self.cd_status_code.is_some() { 2 } else { 0 }
    }

    fn write_payload(&self, socket: &mut dyn BufMut) -> WebResult<()> {
        if let Some(reason) = self.cd_status_code {
            socket.put_u16(reason);
        }
        socket.put_slice(&*self.payload);
        Ok(())
    }

    fn take_payload(self) -> Vec<u8> {
        if let Some(reason) = self.cd_status_code {
            let mut buf = Vec::with_capacity(2 + self.payload.len());
            buf.put_u16(reason);
            buf.append(&mut self.payload.into_owned());
            buf
        } else {
            self.payload.into_owned()
        }
    }
}

impl<'a> Message<'a> {
    /// Attempt to form a message from a series of data frames
    // fn serialize(&self, writer: &mut dyn BufMut, masked: bool) -> WebResult<usize> {
    //     self.write_to(writer, masked)
    // }

    /// Returns how many bytes this message will take up
    fn message_size(&self, masked: bool) -> usize {
        self.frame_size(masked)
    }

    /// Attempt to form a message from a series of data frames
    fn from_dataframes<D>(frames: Vec<D>) -> WebResult<Self>
    where
        D: DataFrameable,
    {
        let opcode = frames
            .first()
            .ok_or(WsError::ProtocolError("No dataframes provided"))
            .map(DataFrameable::opcode)
            .map_err(|e| WebError::Ws(e))?;
        let opcode = Opcode::new(opcode);

        let payload_size = frames.iter().map(DataFrameable::size).sum();

        let mut data = Vec::with_capacity(payload_size);

        for (i, dataframe) in frames.into_iter().enumerate() {
            if i > 0 && dataframe.opcode() != Opcode::Continuation as u8 {
                return Err(
                    WsError::ProtocolError("Unexpected non-continuation data frame").into(),
                );
            }
            if *dataframe.reserved() != [false; 3] {
                return Err(WsError::ProtocolError("Unsupported reserved bits received").into());
            }
            data.append(&mut dataframe.take_payload());
        }

        if opcode == Some(Opcode::Text) {
            if let Err(e) = from_utf8(data.as_slice()) {
                return Err(crate::WebError::Extension("Convert Utf8 error"));
            }
        }

        let msg = match opcode {
            Some(Opcode::Text) => Message {
                opcode: Type::Text,
                cd_status_code: None,
                payload: Cow::Owned(data),
            },
            Some(Opcode::Binary) => Message::binary(data),
            Some(Opcode::Close) => {
                if !data.is_empty() {
                    let status_code = (&data[..]).try_get_u16()?;
                    let reason = std::str::from_utf8(&data[2..])
                        .map_err(|_| crate::WebError::Extension("Convert Utf8 error"))?
                        .to_string();
                    Message::close_because(status_code, reason)
                } else {
                    Message::close()
                }
            }
            Some(Opcode::Ping) => Message::ping(data),
            Some(Opcode::Pong) => Message::pong(data),
            _ => return Err(WsError::ProtocolError("Unsupported opcode received").into()),
        };
        Ok(msg)
    }
}

/// Represents an owned WebSocket message.
///
/// `OwnedMessage`s are generated when the user receives a message (since the data
/// has to be copied out of the network buffer anyway).
/// If you would like to create a message out of borrowed data to use for sending
/// please use the `Message` struct (which contains a `Cow`).
///
/// Note that `OwnedMessage` and `Message` can be converted into each other.
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum OwnedMessage {
    /// A message containing UTF-8 text data
    Text(String),
    /// A message containing binary data
    Binary(Vec<u8>),
    /// A message which indicates closure of the WebSocket connection.
    /// This message may or may not contain data.
    Close(Option<CloseData>),
    /// A ping message - should be responded to with a pong message.
    /// Usually the pong message will be sent with the same data as the
    /// received ping message.
    Ping(Vec<u8>),
    /// A pong message, sent in response to a Ping message, usually
    /// containing the same data as the received ping message.
    Pong(Vec<u8>),
}

impl OwnedMessage {
    /// Checks if this message is a close message.
    ///
    ///```rust
    ///# use webparse::OwnedMessage;
    ///assert!(OwnedMessage::Close(None).is_close());
    ///```
    pub fn is_close(&self) -> bool {
        match *self {
            OwnedMessage::Close(_) => true,
            _ => false,
        }
    }

    /// Checks if this message is a control message.
    /// Control messages are either `Close`, `Ping`, or `Pong`.
    ///
    ///```rust
    ///# use webparse::OwnedMessage;
    ///assert!(OwnedMessage::Ping(vec![]).is_control());
    ///assert!(OwnedMessage::Pong(vec![]).is_control());
    ///assert!(OwnedMessage::Close(None).is_control());
    ///```
    pub fn is_control(&self) -> bool {
        match *self {
            OwnedMessage::Close(_) => true,
            OwnedMessage::Ping(_) => true,
            OwnedMessage::Pong(_) => true,
            _ => false,
        }
    }

    /// Checks if this message is a data message.
    /// Data messages are either `Text` or `Binary`.
    ///
    ///```rust
    ///# use webparse::OwnedMessage;
    ///assert!(OwnedMessage::Text("1337".to_string()).is_data());
    ///assert!(OwnedMessage::Binary(vec![]).is_data());
    ///```
    pub fn is_data(&self) -> bool {
        !self.is_control()
    }

    /// Checks if this message is a ping message.
    /// `Ping` messages can come at any time and usually generate a `Pong` message
    /// response.
    ///
    ///```rust
    ///# use webparse::OwnedMessage;
    ///assert!(OwnedMessage::Ping("ping".to_string().into_bytes()).is_ping());
    ///```
    pub fn is_ping(&self) -> bool {
        match *self {
            OwnedMessage::Ping(_) => true,
            _ => false,
        }
    }

    /// Checks if this message is a pong message.
    /// `Pong` messages are usually sent only in response to `Ping` messages.
    ///
    ///```rust
    ///# use webparse::OwnedMessage;
    ///assert!(OwnedMessage::Pong("pong".to_string().into_bytes()).is_pong());
    ///```
    pub fn is_pong(&self) -> bool {
        match *self {
            OwnedMessage::Pong(_) => true,
            _ => false,
        }
    }
}

impl OwnedMessage {
    /// Attempt to form a message from a series of data frames
    // pub fn serialize(&self, writer: &mut dyn BufMut, masked: bool) -> WebResult<usize> {
    //     self.write_to(writer, masked)
    // }

    /// Returns how many bytes this message will take up
    pub fn message_size(&self, masked: bool) -> usize {
        self.frame_size(masked)
    }

    /// Attempt to form a message from a series of data frames
    pub fn from_dataframes<D>(frames: Vec<D>) -> WebResult<Self>
    where
        D: DataFrameable,
    {
        Ok(Message::from_dataframes(frames)?.into())
    }
}

impl DataFrameable for OwnedMessage {
    #[inline(always)]
    fn is_last(&self) -> bool {
        true
    }

    #[inline(always)]
    fn opcode(&self) -> u8 {
        (match *self {
            OwnedMessage::Text(_) => Type::Text,
            OwnedMessage::Binary(_) => Type::Binary,
            OwnedMessage::Close(_) => Type::Close,
            OwnedMessage::Ping(_) => Type::Ping,
            OwnedMessage::Pong(_) => Type::Pong,
        }) as u8
    }

    #[inline(always)]
    fn reserved(&self) -> &[bool; 3] {
        FALSE_RESERVED_BITS
    }

    fn size(&self) -> usize {
        match *self {
            OwnedMessage::Text(ref txt) => txt.len(),
            OwnedMessage::Binary(ref bin) => bin.len(),
            OwnedMessage::Ping(ref data) => data.len(),
            OwnedMessage::Pong(ref data) => data.len(),
            OwnedMessage::Close(ref data) => match data {
                &Some(ref c) => c.reason.len() + 2,
                &None => 0,
            },
        }
    }

    fn write_payload(&self, socket: &mut dyn BufMut) -> WebResult<()> {
        match *self {
            OwnedMessage::Text(ref txt) => socket.put_slice(txt.as_bytes()),
            OwnedMessage::Binary(ref bin) => socket.put_slice(bin.as_slice()),
            OwnedMessage::Ping(ref data) => socket.put_slice(data.as_slice()),
            OwnedMessage::Pong(ref data) => socket.put_slice(data.as_slice()),
            OwnedMessage::Close(ref data) => match data {
                &Some(ref c) => {
                    socket.put_u16(c.status_code);
                    socket.put_slice(c.reason.as_bytes())
                }
                &None => return Ok(()),
            },
        };
        Ok(())
    }

    fn take_payload(self) -> Vec<u8> {
        match self {
            OwnedMessage::Text(txt) => txt.into_bytes(),
            OwnedMessage::Binary(bin) => bin,
            OwnedMessage::Ping(data) => data,
            OwnedMessage::Pong(data) => data,
            OwnedMessage::Close(data) => match data {
                Some(c) => {
                    let mut buf = Vec::with_capacity(2 + c.reason.len());
                    buf.put_u16(c.status_code);
                    buf.append(&mut c.reason.into_bytes());
                    buf
                }
                None => vec![],
            },
        }
    }
}

impl From<String> for OwnedMessage {
    fn from(text: String) -> Self {
        OwnedMessage::Text(text)
    }
}

impl From<Vec<u8>> for OwnedMessage {
    fn from(buf: Vec<u8>) -> Self {
        OwnedMessage::Binary(buf)
    }
}

impl<'m> From<Message<'m>> for OwnedMessage {
    fn from(message: Message<'m>) -> Self {
        match message.opcode {
            Type::Text => {
                let convert = String::from_utf8_lossy(&message.payload).into_owned();
                OwnedMessage::Text(convert)
            }
            Type::Close => match message.cd_status_code {
                Some(code) => OwnedMessage::Close(Some(CloseData {
                    status_code: code,
                    reason: String::from_utf8_lossy(&message.payload).into_owned(),
                })),
                None => OwnedMessage::Close(None),
            },
            Type::Binary => OwnedMessage::Binary(message.payload.into_owned()),
            Type::Ping => OwnedMessage::Ping(message.payload.into_owned()),
            Type::Pong => OwnedMessage::Pong(message.payload.into_owned()),
        }
    }
}

impl<'m> From<OwnedMessage> for Message<'m> {
    fn from(message: OwnedMessage) -> Self {
        match message {
            OwnedMessage::Text(txt) => Message::text(txt),
            OwnedMessage::Binary(bin) => Message::binary(bin),
            OwnedMessage::Close(because) => match because {
                Some(c) => Message::close_because(c.status_code, c.reason),
                None => Message::close(),
            },
            OwnedMessage::Ping(data) => Message::ping(data),
            OwnedMessage::Pong(data) => Message::pong(data),
        }
    }
}

/// Represents data contained in a Close message
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct CloseData {
    /// The status-code of the CloseData
    pub status_code: u16,
    /// The reason-phrase of the CloseData
    pub reason: String,
}

impl CloseData {
    pub fn normal() -> Self {
        CloseData {
            status_code: CloseCode::Normal.into(),
            reason: String::new(),
        }
    }
    /// Create a new CloseData object
    pub fn new<U>(status_code: U, reason: String) -> CloseData
    where
        U: Into<u16>,
    {
        CloseData {
            status_code: status_code.into(),
            reason,
        }
    }
    /// Convert this into a vector of bytes
    pub fn into_bytes(self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.put_u16(self.status_code);
        for i in self.reason.as_bytes().iter() {
            buf.push(*i);
        }
        Ok(buf)
    }
}

/// Trait representing the ability to convert
/// self to a `Cow<'a, [u8]>`
pub trait IntoCowBytes<'a> {
    /// Consume `self` and produce a `Cow<'a, [u8]>`
    fn into(self) -> Cow<'a, [u8]>;
}

impl<'a> IntoCowBytes<'a> for Vec<u8> {
    fn into(self) -> Cow<'a, [u8]> {
        Cow::Owned(self)
    }
}

impl<'a> IntoCowBytes<'a> for &'a [u8] {
    fn into(self) -> Cow<'a, [u8]> {
        Cow::Borrowed(self)
    }
}

impl<'a> IntoCowBytes<'a> for Cow<'a, [u8]> {
    fn into(self) -> Cow<'a, [u8]> {
        self
    }
}

use self::CloseCode::*;
/// Status code used to indicate why an endpoint is closing the WebSocket connection.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CloseCode {
    /// Indicates a normal closure, meaning that the purpose for
    /// which the connection was established has been fulfilled.
    /// 表示一个正常的关闭，意味着连接建立的目标已经完成了。
    Normal,
    /// Indicates that an endpoint is "going away", such as a server
    /// going down or a browser having navigated away from a page.
    /// 表示终端已经“走开”，例如服务器停机了或者在浏览器中离开了这个页面。
    Away,
    /// Indicates that an endpoint is terminating the connection due
    /// to a protocol error.
    /// 表示终端由于协议错误中止了连接。
    Protocol,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received a type of data it cannot accept (e.g., an
    /// endpoint that understands only text data MAY send this if it
    /// receives a binary message).
    /// 表示终端由于收到了一个不支持的数据类型的数据（如终端只能怪理解文本数据，但是收到了一个二进制数据）从而关闭连接。
    Unsupported,
    /// Indicates that no status code was included in a closing frame. This
    /// close code makes it possible to use a single method, `on_close` to
    /// handle even cases where no close code was provided.
    /// 是一个保留值并且不能被终端当做一个关闭帧的状态码。这个状态码是为了给上层应用表示当前没有状态码。
    Status,
    /// Indicates an abnormal closure. If the abnormal closure was due to an
    /// error, this close code will not be used. Instead, the `on_error` method
    /// of the handler will be called with the error. However, if the connection
    /// is simply dropped, without an error, this close code will be sent to the
    /// handler.
    /// 是一个保留值并且不能被终端当做一个关闭帧的状态码。这个状态码是为了给上层应用表示
    /// 连接被异常关闭如没有发送或者接受一个关闭帧这种场景的使用而设计的。
    Abnormal,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received data within a message that was not
    /// consistent with the type of the message (e.g., non-UTF-8 [RFC3629]
    /// data within a text message).
    /// 表示终端因为收到了类型不连续的消息（如非 UTF-8 编码的文本消息）导致的连接关闭。
    Invalid,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received a message that violates its policy.  This
    /// is a generic status code that can be returned when there is no
    /// other more suitable status code (e.g., Unsupported or Size) or if there
    /// is a need to hide specific details about the policy.
    /// 表示终端是因为收到了一个违反政策的消息导致的连接关闭。这是一个通用的状态码，
    /// 可以在没有什么合适的状态码（如 1003 或者 1009）时或者可能需要隐藏关于政策的具体信息时返回。
    Policy,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received a message that is too big for it to
    /// process.
    /// 表示终端由于收到了一个太大的消息无法进行处理从而关闭连接。
    Size,
    /// Indicates that an endpoint (client) is terminating the
    /// connection because it has expected the server to negotiate one or
    /// more extension, but the server didn't return them in the response
    /// message of the WebSocket handshake.  The list of extensions that
    /// are needed should be given as the reason for closing.
    /// Note that this status code is not used by the server, because it
    /// can fail the WebSocket handshake instead.
    /// 表示终端（客户端）因为预期与服务端协商一个或者多个扩展，但是服务端在 WebSocket 握手中没有响应这个导致的关闭。
    /// 需要的扩展清单应该出现在关闭帧的原因（reason）字段中。
    Extension,
    /// Indicates that a server is terminating the connection because
    /// it encountered an unexpected condition that prevented it from
    /// fulfilling the request.
    /// 表示服务端因为遇到了一个意外的条件阻止它完成这个请求从而导致连接关闭。
    Error,
    /// Indicates that the server is restarting. A client may choose to reconnect,
    /// and if it does, it should use a randomized delay of 5-30 seconds between attempts.
    Restart,
    /// Indicates that the server is overloaded and the client should either connect
    /// to a different IP (when multiple targets exist), or reconnect to the same IP
    /// when a user has performed an action.
    Again,
    #[doc(hidden)]
    /// 这个状态码是用于上层应用来表示连接失败是因为 TLS 握手失败（如服务端证书没有被验证过）导致的关闭的。
    Tls,
    #[doc(hidden)]
    Empty,
    #[doc(hidden)]
    Other(u16),
}

impl Into<u16> for CloseCode {
    fn into(self) -> u16 {
        match self {
            Normal => 1000,
            Away => 1001,
            Protocol => 1002,
            Unsupported => 1003,
            Status => 1005,
            Abnormal => 1006,
            Invalid => 1007,
            Policy => 1008,
            Size => 1009,
            Extension => 1010,
            Error => 1011,
            Restart => 1012,
            Again => 1013,
            Tls => 1015,
            Empty => 0,
            Other(code) => code,
        }
    }
}

impl From<u16> for CloseCode {
    fn from(code: u16) -> CloseCode {
        match code {
            1000 => Normal,
            1001 => Away,
            1002 => Protocol,
            1003 => Unsupported,
            1005 => Status,
            1006 => Abnormal,
            1007 => Invalid,
            1008 => Policy,
            1009 => Size,
            1010 => Extension,
            1011 => Error,
            1012 => Restart,
            1013 => Again,
            1015 => Tls,
            0 => Empty,
            _ => Other(code),
        }
    }
}
