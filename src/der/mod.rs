//! Distinguised Encoding Rules (DER) Reader and Writer
//!
//! This encoding enforces one canonical representation of the encoding,
//! where the efficiency is biased towards the reader.
//!
//! This is the usual format of cryptographic material, although in few
//! cases, some cryptographic material need to use BER relaxed rules for
//! reading as their encoding wasn't done strictly.
pub mod reader;
pub mod writer;

pub use self::{reader::Reader, writer::Writer};

#[cfg(test)]
mod tests {
    use super::*;
    //use crate::objects::*;

    #[test]
    pub fn encode_double_sequence() {
        let mut buf = [0u8; 1024];
        let mut writer = Writer::new(&mut buf);

        let ostring = [2u8; 0x7c];

        writer
            .sequence(|writer| {
                writer.sequence(|writer| writer.octetstring(&ostring))?;
                writer.bool(true)
            })
            .expect("outer sequence");
        let slice = writer.finish();

        let mut reader = Reader::new(slice);
        let mut seqreader = reader.sequence().expect("outer sequence");
        let mut inreader = seqreader.sequence().expect("inner sequence");
        let b = seqreader.bool().expect("bool");
        assert_eq!(b, true);
        seqreader.done().expect("outer done");
        let ostring2 = inreader.octetstring().expect("octetstring");
        inreader.done().expect("inner done");
        assert_eq!(ostring2, &ostring[..]);
    }
}
