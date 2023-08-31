use std::{fmt, mem};

use crate::{Http2Error, WebError, WebResult, MarkBuf, Buf, BufMut};

use super::{
    encode_u64, frame::FrameHeader, read_u64, ErrorCode, Flag, Kind, ParserSettings, SizeIncrement,
    StreamIdentifier,
};

const PRIORITY_BYTES: u32 = 5;
const PADDING_BYTES: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Payload<T>
where T: Buf + MarkBuf {
    Data {
        data: T,
    },
    Headers {
        priority: Option<Priority>,
        block: T,
    },
    Priority(Priority),
    Reset(ErrorCode),
    Settings(Vec<Setting>),
    PushPromise {
        promised: StreamIdentifier,
        block: T,
    },
    Ping(u64),
    GoAway {
        last: StreamIdentifier,
        error: ErrorCode,
        data: T,
    },
    WindowUpdate(SizeIncrement),
    Continuation(T),
    Unregistered(T),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Priority {
    exclusive: bool,
    dependency: StreamIdentifier,
    weight: u8,
}

impl Priority {
    #[inline]
    pub fn parse<T: Buf+MarkBuf>(present: bool, mut buffer: T) -> WebResult<(T, Option<Priority>)> {
        if present {        
            let first = buffer.peek().unwrap();
            let id = StreamIdentifier::parse(&mut buffer);
            let weight = buffer.get_u8();
            Ok((
                buffer.mark_clone_slice(),
                Some(Priority {
                    // Most significant bit.
                    exclusive: first & 0x7F != first,
                    dependency: id,
                    weight,
                }),
            ))
        } else {
            Ok((buffer, None))
        }
    }

    #[inline]
    pub fn encode<B: Buf + BufMut + MarkBuf>(&self, buf: &mut B) -> usize {
        let mut dependency = self.dependency;
        if self.exclusive {
            dependency.0 |= 1 << 31
        }

        dependency.encode(buf);
        buf.put_u8(self.weight);
        PRIORITY_BYTES as usize
    }
}

// Settings are (u16, u32) in memory.
#[repr(packed)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Setting {
    identifier: u16,
    value: u32,
}

impl fmt::Debug for Setting {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.identifier(), f)
    }
}

impl Setting {
    #[inline]
    pub fn new(identifier: SettingIdentifier, value: u32) -> Setting {
        Setting {
            identifier: identifier as u16,
            value: value,
        }
    }

    #[inline]
    pub fn identifier(&self) -> Option<SettingIdentifier> {
        match self.identifier {
            0x1 => Some(SettingIdentifier::HeaderTableSize),
            0x2 => Some(SettingIdentifier::EnablePush),
            0x3 => Some(SettingIdentifier::MaxConcurrentStreams),
            0x4 => Some(SettingIdentifier::InitialWindowSize),
            0x5 => Some(SettingIdentifier::MaxFrameSize),
            0x6 => Some(SettingIdentifier::MaxHeaderListSize),
            _ => None,
        }
    }

    #[inline]
    pub fn value(&self) -> u32 {
        self.value
    }

    #[inline]
    fn encode<B: Buf+MarkBuf+BufMut>(settings: &[Setting], buf: &mut B) -> usize {
        let mut size = 0;
        for setting in settings {
            buf.put_u16(setting.identifier);
            buf.put_u32(setting.value);
            size += 6;
        }
        size
    }

    #[inline]
    fn decode<T: Buf+MarkBuf>(bytes: &mut T) -> Vec<Setting> {
        let len = bytes.remaining() / mem::size_of::<Setting>();
        let mut result = vec![];
        for _ in 0..len {
            let identifier = bytes.get_u16();
            let value = bytes.get_u32();
            result.push(Setting {
                identifier,
                value
            })
        }
        result
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingIdentifier {
    //允许发送者以八位字节的形式通知远程端点用于解码头块的头压缩表的最大尺寸。编码器可以通过使用特定于头部块内头部压缩格式的信令来选择等于或小于此值的任何大小（请参见[压缩]）。初始值是4,096个八位字节。
    HeaderTableSize = 0x1,
    //此设置可用于禁用服务器推送（第8.2节）。如果一个端点接收到这个参数设置为0的值，它不应该发送一个PUSH_PROMISE帧。一个端点既将这个参数设置为0，并且确认它也必须将PUSH_PROMISE帧的接收视为连接错误（见5.4节）。 1）类型PROTOCOL_ERROR。初始值为1，表示允许服务器推送。除0或1以外的任何值必须视为PROTOCOL_ERROR类型的连接错误
    EnablePush = 0x2,
    //表示发件人允许的最大并发流数。这个限制是有方向性的：它适用于发送者允许接收者创建的数据流。最初，这个值没有限制。建议此值不小于100，以免不必要地限制并行性。值为0的SETTINGS_MAX_CONCURRENT_STREAMS不应被视为特殊的端点。零值确实会阻止创建新的流;然而，这也可能发生在活动流所耗尽的任何限制上。服务器应该只在短时间内设置一个零值;如果服务器不希望接受请求，关闭连接更合适。
    MaxConcurrentStreams = 0x3,
    //指示发送者的流级别流控制的初始窗口大小（以八位字节为单位）。初始值是2 ^ 16-1（65,535）个八位组。该设置会影响所有流的窗口大小（请参阅第6.9.2节）。高于最大流量控制窗口大小2 ^ 31-1的值必须视为FLOW_CONTROL_ERROR类型的连接错误;然而，这也可能发生在活动流所耗尽的任何限制上。服务器应该只在短时间内设置一个零值;如果服务器不希望接受请求，关闭连接更合适。
    InitialWindowSize = 0x4,
    //指示发送者愿意接收的最大帧有效载荷的大小，以八位字节为单位。初始值是2 ^ 14（16,384）个八位字节。端点通告的值必须在该初始值和最大允许帧大小之间（2 ^ 24-1或16,777,215个八位字节），包括在内。此范围之外的值务必视为PROTOCOL_ERROR类型的连接错误
    MaxFrameSize = 0x5,
    //此通报设置以八位字节的形式通知对等方发送方准备接受的标题列表的最大大小。该值基于头字段的未压缩大小，包括名称和八位字节的值的长度，以及每个头字段的开销32个字节。对于任何给定的请求，可能会强制实施一个比所宣传的更低的限制。
    MaxHeaderListSize = 0x6,
}

impl<T: Buf+MarkBuf> Payload<T> {
    #[inline]
    pub fn kind(&self) -> Kind {
        use self::Payload::*;

        match *self {
            Data { .. } => Kind::Data,
            Headers { .. } => Kind::Headers,
            Priority(..) => Kind::Priority,
            Reset(..) => Kind::Reset,
            Settings(..) => Kind::Settings,
            PushPromise { .. } => Kind::PushPromise,
            Ping(..) => Kind::Ping,
            GoAway { .. } => Kind::GoAway,
            WindowUpdate(_) => Kind::WindowUpdate,
            Continuation(_) => Kind::Continuation,
            Unregistered(_) => Kind::Unregistered,
        }
    }

    #[inline]
    pub fn parse(header: FrameHeader, buffer: &mut T) -> WebResult<Payload<T>> {
        let settings = ParserSettings {
            padding: header.flag.contains(Flag::padded()),
            priority: header.flag.contains(Flag::priority()),
        };

        if buffer.remaining() < header.length as usize {
            return Err(Http2Error::into(Http2Error::Short));
        }

        let min_payload_length = if settings.priority && settings.padding {
            PRIORITY_BYTES + PADDING_BYTES
        } else if settings.priority {
            PRIORITY_BYTES
        } else if settings.padding {
            PADDING_BYTES
        } else {
            0
        };

        if header.length < min_payload_length {
            return Err(Http2Error::into(Http2Error::PayloadLengthTooShort));
        }

        let buf = buffer.mark_clone_slice_range(..header.length as isize);
        buffer.advance(header.length as usize);

        match header.kind {
            Kind::Data => Payload::parse_data(header, buf, settings),
            Kind::Headers => Payload::parse_headers(header, buf, settings),
            Kind::Priority => {
                let (_, priority) = Priority::parse(true, buf)?;
                Ok(Payload::Priority(priority.unwrap()))
            }
            Kind::Reset => Payload::parse_reset(header, buf),
            Kind::Settings => Payload::parse_settings(header, buf),
            Kind::Ping => Payload::parse_ping(header, buf),
            Kind::GoAway => Payload::parse_goaway(header, buf),
            Kind::WindowUpdate => Payload::parse_window_update(header, buf),
            Kind::PushPromise => Payload::parse_push_promise(header, buf, settings),
            Kind::Continuation => Ok(Payload::Continuation(buf)),
            Kind::Unregistered => Ok(Payload::Unregistered(buf)),
        }
    }

    #[inline]
    pub fn encode<B: Buf + BufMut + MarkBuf>(&self, buf: &mut B) -> usize {
        match *self {
            Payload::Data { ref data } => encode_memory(data, buf),
            Payload::Headers {
                ref priority,
                ref block,
            } => {
                let priority_wrote = priority.map(|p| p.encode(buf)).unwrap_or(0);
                let block_wrote = encode_memory(block, buf);
                priority_wrote + block_wrote
            }
            Payload::Reset(ref err) => err.encode(buf),
            Payload::Settings(ref settings) => Setting::encode(&settings, buf),
            Payload::Ping(data) => {
                buf.put_u64(data);
                8
            },
            Payload::GoAway {
                ref data,
                ref last,
                ref error,
            } => {
                let last_wrote = last.encode(buf);
                let error_wrote = error.encode(buf);
                encode_memory(data, buf) + last_wrote + error_wrote
            }
            Payload::WindowUpdate(ref increment) => increment.encode(buf),
            Payload::PushPromise {
                ref promised,
                ref block,
            } => {
                promised.encode(buf);
                encode_memory(block, buf) + 4
            }
            Payload::Priority(ref priority) => priority.encode(buf),
            Payload::Continuation(ref block) => encode_memory(block, buf),
            Payload::Unregistered(ref block) => encode_memory(block, buf),
        }
    }

    #[inline]
    /// How many bytes this Payload would be encoded.
    pub fn encoded_len(&self) -> usize {
        use self::Payload::*;

        match *self {
            Data { ref data } => data.remaining(),
            Headers {
                ref priority,
                ref block,
            } => {
                let priority_len = if priority.is_some() { 5 } else { 0 };
                priority_len + block.remaining()
            }
            Reset(_) => 4,
            Settings(ref settings) => settings.len() * mem::size_of::<Setting>(),
            Ping(_) => 8,
            GoAway { ref data, .. } => 4 + 4 + data.remaining(),
            WindowUpdate(_) => 4,
            PushPromise { ref block, .. } => 4 + block.remaining(),
            Priority(_) => 5,
            Continuation(ref block) => block.remaining(),
            Unregistered(ref block) => block.remaining(),
        }
    }

    #[inline]
    pub fn padded(&self) -> Option<u32> {
        None
    }

    #[inline]
    pub fn priority(&self) -> Option<&Priority> {
        match *self {
            Payload::Priority(ref priority) => Some(priority),
            Payload::Headers { ref priority, .. } => priority.as_ref(),
            _ => None,
        }
    }

    #[inline]
    fn parse_data(
        header: FrameHeader,
        mut buf: T,
        settings: ParserSettings,
    ) -> WebResult<Payload<T>> {
        trim_padding(settings, header, &mut buf)?;
        Ok(Payload::Data {
            data: buf,
        })
    }

    #[inline]
    fn parse_headers(
        header: FrameHeader,
        mut buf: T,
        settings: ParserSettings,
    ) -> WebResult<Payload<T>> {
        trim_padding(settings, header, &mut buf)?;
        let (buf, priority) = Priority::parse(settings.priority, buf)?;
        Ok(Payload::Headers {
            priority: priority,
            block: buf,
        })
    }

    #[inline]
    fn parse_reset(header: FrameHeader, mut buf: T) -> WebResult<Payload<T>> {
        if header.length < 4 {
            return Err(Http2Error::into(Http2Error::PayloadLengthTooShort));
        }

        Ok(Payload::Reset(ErrorCode::parse(&mut buf)))
    }

    #[inline]
    fn parse_settings(header: FrameHeader, mut buf: T) -> WebResult<Payload<T>> {
        if header.length % mem::size_of::<Setting>() as u32 != 0 {
            return Err(Http2Error::into(Http2Error::PartialSettingLength));
        }

        Ok(Payload::Settings(Setting::decode(&mut buf)))
    }

    #[inline]
    fn parse_ping(header: FrameHeader, mut buf: T) -> WebResult<Payload<T>> {
        if header.length != 8 {
            return Err(Http2Error::into(Http2Error::InvalidPayloadLength));
        }

        let data = read_u64(&mut buf);
        Ok(Payload::Ping(data))
    }

    #[inline]
    fn parse_goaway(header: FrameHeader, mut buf: T) -> WebResult<Payload<T>> {
        if header.length < 8 {
            return Err(Http2Error::into(Http2Error::PayloadLengthTooShort));
        }

        let last = StreamIdentifier::parse(&mut buf);
        let error = ErrorCode::parse(&mut buf);

        Ok(Payload::GoAway {
            last,
            error,
            data: buf.mark_clone_slice(),
        })
    }

    #[inline]
    fn parse_window_update(header: FrameHeader, mut buf: T) -> WebResult<Payload<T>> {
        if header.length != 4 {
            return Err(Http2Error::into(Http2Error::InvalidPayloadLength));
        }

        Ok(Payload::WindowUpdate(SizeIncrement::parse(&mut buf)))
    }

    #[inline]
    fn parse_push_promise(
        header: FrameHeader,
        mut buf: T,
        settings: ParserSettings,
    ) -> WebResult<Payload<T>> {
        trim_padding(settings, header, &mut buf)?;

        if buf.remaining() < 4 {
            return Err(Http2Error::into(Http2Error::PayloadLengthTooShort));
        }

        let promised = StreamIdentifier::parse(&mut buf);
        let block = buf.mark_clone_slice();

        Ok(Payload::PushPromise {
            promised,
            block,
        })
    }
}

#[inline]
fn encode_memory<T: Buf + MarkBuf, B: Buf + BufMut + MarkBuf>(src: &T, mut dst: &mut B) -> usize {
    dst.put_slice(src.chunk())
}

#[inline]
fn trim_padding<T: Buf + MarkBuf>(settings: ParserSettings, header: FrameHeader, buf: &mut T) -> WebResult<()> {
    if settings.padding && buf.has_remaining() {
        let pad_length = buf.peek().unwrap();
        if pad_length as u32 > header.length {
            return Err(Http2Error::into(Http2Error::TooMuchPadding(pad_length)))
        } else {
            buf.advance(1);
            buf.mark_len(header.length as usize - pad_length as usize - 1);
        }
    }
    Ok(())
}
