use crate::{
    http::http2::{frame::{Kind, StreamIdentifier}, DEFAULT_MAX_FRAME_SIZE, MAX_MAX_FRAME_SIZE, MAX_INITIAL_WINDOW_SIZE},
    Buf, BufMut, Http2Error, MarkBuf, WebResult,
};

use super::{frame::FrameHeader, Flag};

#[derive(Clone, Default, Eq, PartialEq, Debug, Hash)]
pub struct Settings {
    flags: Flag,
    // Fields
    header_table_size: Option<u32>,
    enable_push: Option<u32>,
    max_concurrent_streams: Option<u32>,
    initial_window_size: Option<u32>,
    max_frame_size: Option<u32>,
    max_header_list_size: Option<u32>,
    enable_connect_protocol: Option<u32>,
}

#[derive(Debug)]
pub enum Setting {
    HeaderTableSize(u32),
    EnablePush(u32),
    MaxConcurrentStreams(u32),
    InitialWindowSize(u32),
    MaxFrameSize(u32),
    MaxHeaderListSize(u32),
    EnableConnectProtocol(u32),
}

// ===== impl Setting =====

impl Setting {
    /// Creates a new `Setting` with the correct variant corresponding to the
    /// given setting id, based on the settings IDs defined in section
    /// 6.5.2.
    pub fn from_id(id: u16, val: u32) -> Option<Setting> {
        use self::Setting::*;

        match id {
            1 => Some(HeaderTableSize(val)),
            2 => Some(EnablePush(val)),
            3 => Some(MaxConcurrentStreams(val)),
            4 => Some(InitialWindowSize(val)),
            5 => Some(MaxFrameSize(val)),
            6 => Some(MaxHeaderListSize(val)),
            8 => Some(EnableConnectProtocol(val)),
            _ => None,
        }
    }

    fn parse<T: Buf + MarkBuf>(bytes: &mut T) -> Option<Setting> {
        let id: u16 = bytes.get_u16();
        let val: u32 = bytes.get_u32();

        Setting::from_id(id, val)
    }

    fn encode<B: Buf + MarkBuf + BufMut>(&self, dst: &mut B) -> WebResult<usize> {
        use self::Setting::*;

        let (kind, val) = match *self {
            HeaderTableSize(v) => (1, v),
            EnablePush(v) => (2, v),
            MaxConcurrentStreams(v) => (3, v),
            InitialWindowSize(v) => (4, v),
            MaxFrameSize(v) => (5, v),
            MaxHeaderListSize(v) => (6, v),
            EnableConnectProtocol(v) => (8, v),
        };

        dst.put_u16(kind);
        dst.put_u32(val);
        Ok(6)
    }
}

impl Settings {
    pub fn ack() -> Settings {
        Settings {
            flags: Flag::ack(),
            ..Settings::default()
        }
    }

    pub fn is_ack(&self) -> bool {
        self.flags.is_ack()
    }

    pub fn flags(&self) -> Flag {
        self.flags
    }

    pub fn initial_window_size(&self) -> Option<u32> {
        self.initial_window_size
    }

    pub fn set_initial_window_size(&mut self, size: Option<u32>) {
        self.initial_window_size = size;
    }

    pub fn max_concurrent_streams(&self) -> Option<u32> {
        self.max_concurrent_streams
    }

    pub fn set_max_concurrent_streams(&mut self, max: Option<u32>) {
        self.max_concurrent_streams = max;
    }

    pub fn max_frame_size(&self) -> Option<u32> {
        self.max_frame_size
    }

    pub fn set_max_frame_size(&mut self, size: Option<u32>) {
        if let Some(val) = size {
            assert!(DEFAULT_MAX_FRAME_SIZE <= val && val <= MAX_MAX_FRAME_SIZE);
        }
        self.max_frame_size = size;
    }

    pub fn max_header_list_size(&self) -> Option<u32> {
        self.max_header_list_size
    }

    pub fn set_max_header_list_size(&mut self, size: Option<u32>) {
        self.max_header_list_size = size;
    }

    pub fn is_push_enabled(&self) -> Option<bool> {
        self.enable_push.map(|val| val != 0)
    }

    pub fn set_enable_push(&mut self, enable: bool) {
        self.enable_push = Some(enable as u32);
    }

    pub fn is_extended_connect_protocol_enabled(&self) -> Option<bool> {
        self.enable_connect_protocol.map(|val| val != 0)
    }

    pub fn set_enable_connect_protocol(&mut self, val: Option<u32>) {
        self.enable_connect_protocol = val;
    }

    pub fn header_table_size(&self) -> Option<u32> {
        self.header_table_size
    }

    /*
    pub fn set_header_table_size(&mut self, size: Option<u32>) {
        self.header_table_size = size;
    }
    */

    pub fn parse<T: Buf + MarkBuf>(head: FrameHeader, payload: &mut T) -> WebResult<Settings> {
        use self::Setting::*;

        debug_assert_eq!(head.kind(), &Kind::Settings);

        if !head.stream_id().is_zero() {
            return Err(Http2Error::into(Http2Error::InvalidStreamId));
        }

        // Load the flag
        let flag = head.flag();

        if flag.is_ack() {
            // Ensure that the payload is empty
            if payload.has_remaining() {
                return Err(Http2Error::into(Http2Error::InvalidPayloadLength));
            }

            // Return the ACK frame
            return Ok(Settings::ack());
        }

        // Ensure the payload length is correct, each setting is 6 bytes long.
        if payload.remaining() % 6 != 0 {
            return Err(Http2Error::into(Http2Error::InvalidPayloadLength));
        }

        let mut settings = Settings::default();
        debug_assert!(!settings.flags.is_ack());

        let len = payload.remaining() / 6;
        for _ in 0..len {
            match Setting::parse(payload) {
                Some(HeaderTableSize(val)) => {
                    settings.header_table_size = Some(val);
                }
                Some(EnablePush(val)) => match val {
                    0 | 1 => {
                        settings.enable_push = Some(val);
                    }
                    _ => {
                        return Err(Http2Error::InvalidSettingValue.into());
                    }
                },
                Some(MaxConcurrentStreams(val)) => {
                    settings.max_concurrent_streams = Some(val);
                }
                Some(InitialWindowSize(val)) => {
                    if val as usize > MAX_INITIAL_WINDOW_SIZE {
                        return Err(Http2Error::InvalidSettingValue.into());
                    } else {
                        settings.initial_window_size = Some(val);
                    }
                }
                Some(MaxFrameSize(val)) => {
                    if DEFAULT_MAX_FRAME_SIZE <= val && val <= MAX_MAX_FRAME_SIZE {
                        settings.max_frame_size = Some(val);
                    } else {
                        return Err(Http2Error::InvalidSettingValue.into());
                    }
                }
                Some(MaxHeaderListSize(val)) => {
                    settings.max_header_list_size = Some(val);
                }
                Some(EnableConnectProtocol(val)) => match val {
                    0 | 1 => {
                        settings.enable_connect_protocol = Some(val);
                    }
                    _ => {
                        return Err(Http2Error::InvalidSettingValue.into());
                    }
                },
                None => {}
            }
        }
        Ok(settings)
    }

    pub fn payload_len(&self) -> usize {
        let mut len = 0;
        self.for_each(|_| len += 6);
        len
    }

    pub fn encode<B: Buf + MarkBuf + BufMut>(&self, dst: &mut B) -> WebResult<usize> {
        // Create & encode an appropriate frame head
        let mut head =
            FrameHeader::new(Kind::Settings, self.flags.into(), StreamIdentifier::zero());
        head.length = self.payload_len() as u32;

        println!("encoding SETTINGS; len={}", head.length);
        let mut size = 0;
        size += head.encode(dst)?;

        // Encode the settings
        self.for_each(|setting| {
            log::trace!("encoding setting; val={:?}", setting);
            size += setting.encode(dst).unwrap()
        });
        Ok(size)
    }

    fn for_each<F: FnMut(Setting)>(&self, mut f: F) {
        use self::Setting::*;

        if let Some(v) = self.header_table_size {
            f(HeaderTableSize(v));
        }

        if let Some(v) = self.enable_push {
            f(EnablePush(v));
        }

        if let Some(v) = self.max_concurrent_streams {
            f(MaxConcurrentStreams(v));
        }

        if let Some(v) = self.initial_window_size {
            f(InitialWindowSize(v));
        }

        if let Some(v) = self.max_frame_size {
            f(MaxFrameSize(v));
        }

        if let Some(v) = self.max_header_list_size {
            f(MaxHeaderListSize(v));
        }

        if let Some(v) = self.enable_connect_protocol {
            f(EnableConnectProtocol(v));
        }
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
