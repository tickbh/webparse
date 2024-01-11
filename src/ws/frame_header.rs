use bitflags::bitflags;

use crate::{Buf, BufMut, WebResult, ws::WsError};

bitflags! {
    /// Flags relevant to a WebSocket data frame.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct WsFrameFlags: u8 {
        /// Marks this dataframe as the last dataframe
        const FIN = 0x80;
        /// First reserved bit
        const RSV1 = 0x40;
        /// Second reserved bit
        const RSV2 = 0x20;
        /// Third reserved bit
        const RSV3 = 0x10;
    }
}

/// Represents a data frame header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WsFrameHeader {
    /// The bit flags for the first byte of the header.
    pub flags: WsFrameFlags,
    /// The opcode of the header - must be <= 16.
    pub opcode: u8,
    /// The masking key, if any.
    pub mask: Option<[u8; 4]>,
    /// The length of the payload.
    pub len: u64,
}

/// Writes a data frame header.
pub fn write_header(writer: &mut dyn BufMut, header: WsFrameHeader) -> WebResult<()> {
    if header.opcode > 0xF {
        return Err(WsError::DataFrameError("Invalid data frame opcode").into());
    }
    if header.opcode >= 8 && header.len >= 126 {
        return Err(WsError::DataFrameError("Control frame length too long").into());
    }

    // Write 'FIN', 'RSV1', 'RSV2', 'RSV3' and 'opcode'
    writer.put_u8((header.flags.bits()) | header.opcode);

    writer.put_u8(
        // Write the 'MASK'
        if header.mask.is_some() { 0x80 } else { 0x00 } |
		// Write the 'Payload len'
		if header.len <= 125 { header.len as u8 }
		else if header.len <= 65535 { 126 }
		else { 127 },
    );

    // Write 'Extended payload length'
    if header.len >= 126 && header.len <= 65535 {
        writer.put_u16(header.len as u16);
    } else if header.len > 65535 {
        writer.put_u64(header.len);
    }

    // Write 'Masking-key'
    if let Some(mask) = header.mask {
        writer.put_slice(&mask);
    }

    Ok(())
}

/// Reads a data frame header.
pub fn read_header<R>(reader: &mut R) -> WebResult<WsFrameHeader>
where
    R: Buf,
{
    let byte0 = reader.try_get_u8()?;
    let byte1 = reader.try_get_u8()?;

    let flags = WsFrameFlags::from_bits_truncate(byte0);
    let opcode = byte0 & 0x0F;

    let len = match byte1 & 0x7F {
        0..=125 => u64::from(byte1 & 0x7F),
        126 => {
            let len = u64::from(reader.try_get_u16()?);
            if len <= 125 {
                return Err(WsError::DataFrameError("Invalid data frame length").into());
            }
            len
        }
        127 => {
            let len = reader.try_get_u64()?;
            if len <= 65535 {
                return Err(WsError::DataFrameError("Invalid data frame length").into());
            }
            len
        }
        _ => unreachable!(),
    };

    if opcode >= 8 {
        if len >= 126 {
            return Err(WsError::DataFrameError("Control frame length too long").into());
        }
        if !flags.contains(WsFrameFlags::FIN) {
            return Err(WsError::ProtocolError("Illegal fragmented control frame").into());
        }
    }

    let mask = if byte1 & 0x80 == 0x80 {
        Some([
            reader.try_get_u8()?,
            reader.try_get_u8()?,
            reader.try_get_u8()?,
            reader.try_get_u8()?,
        ])
    } else {
        None
    };

    Ok(WsFrameHeader {
        flags,
        opcode,
        mask,
        len,
    })
}

mod tests {
    
    use test;

    #[test]
    fn test_read_header_simple() {
        let header = [0x81, 0x2B];
        let obtained = read_header(&mut &header[..]).unwrap();
        let expected = WsFrameHeader {
            flags: WsFrameFlags::FIN,
            opcode: 1,
            mask: None,
            len: 43,
        };
        assert_eq!(obtained, expected);
    }

    #[test]
    fn test_write_header_simple() {
        let header = WsFrameHeader {
            flags: WsFrameFlags::FIN,
            opcode: 1,
            mask: None,
            len: 43,
        };
        let expected = [0x81, 0x2B];
        let mut obtained = Vec::with_capacity(2);
        write_header(&mut obtained, header).unwrap();

        assert_eq!(&obtained[..], &expected[..]);
    }

    #[test]
    fn test_read_header_complex() {
        let header = [0x42, 0xFE, 0x02, 0x00, 0x02, 0x04, 0x08, 0x10];
        let obtained = read_header(&mut &header[..]).unwrap();
        let expected = WsFrameHeader {
            flags: WsFrameFlags::RSV1,
            opcode: 2,
            mask: Some([2, 4, 8, 16]),
            len: 512,
        };
        assert_eq!(obtained, expected);
    }

    #[test]
    fn test_write_header_complex() {
        let header = WsFrameHeader {
            flags: WsFrameFlags::RSV1,
            opcode: 2,
            mask: Some([2, 4, 8, 16]),
            len: 512,
        };
        let expected = [0x42, 0xFE, 0x02, 0x00, 0x02, 0x04, 0x08, 0x10];
        let mut obtained = Vec::with_capacity(8);
        write_header(&mut obtained, header).unwrap();

        assert_eq!(&obtained[..], &expected[..]);
    }

}
