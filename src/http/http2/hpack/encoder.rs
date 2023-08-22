
// use super::
// use super::
use super::HeaderIndex;

pub struct Encoder {
    pub index: HeaderIndex,
}


impl Encoder {
    pub fn new() -> Encoder {
        Encoder {
            index: HeaderIndex::new(),
        }
    }


    // pub fn encode<'b, I>(&mut self, headers: I) -> Vec<u8>
    //         where I: IntoIterator<Item=(&'b [u8], &'b [u8])> {
    //     let mut encoded: Vec<u8> = Vec::new();
    //     self.encode_into(headers, &mut encoded).unwrap();
    //     encoded
    // }

    // pub fn encode_into<'b, I, W>(&mut self, headers: I, writer: &mut W) -> io::Result<()>
    //         where I: IntoIterator<Item=(&'b [u8], &'b [u8])>,
    //               W: io::Write {
    //     for header in headers {
    //         self.encode_header_into(header, writer)?;
    //     }
    //     Ok(())
    // }

}