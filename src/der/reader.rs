use crate::header::constants;
use crate::header::{Class, Identifier, Length, PC};
use crate::intenc::Integer8Bit;
use crate::objects::*;

#[derive(Clone)]
pub struct Reader<'a> {
    index: usize,
    slice: &'a [u8],
}

#[derive(Debug, Clone)]
pub enum Error {
    ExpectedCType { expected: PC, got: PC },
    ExpectedTag { expected: u32, got: u32 },
    ExpectedClass { expected: Class, got: Class },
    IndefiniteLengthDER,
    BoolLengthInvalid(usize),
    BoolEncodingInvalid(u8),
    BitStringEncodingEmpty,
    BitStringEncodingInvalidStart,
    BitStringEncodingInvalidEnd,
    IntegerNotCanonical,
    Utf8Invalid,
    NullEncodingInvalid,
    OIDInvalid,
    ReaderNotTerminated { index: usize, len: usize },
}

fn assume(header: &Identifier, pc: PC, tag: u32) -> Result<(), Error> {
    if header.class != Class::Universal {
        return Err(Error::ExpectedClass {
            expected: Class::Universal,
            got: header.class,
        });
    }
    if header.pc != pc {
        return Err(Error::ExpectedCType {
            expected: pc,
            got: header.pc,
        });
    }
    if header.tag.value() != tag {
        return Err(Error::ExpectedTag {
            expected: tag,
            got: header.tag.value(),
        });
    }
    Ok(())
}

/// Iterator to iterate over an element from a DER SET
#[derive(Clone)]
pub struct Set<'a, F>(Reader<'a>, F);

impl<'a, A, F> Iterator for Set<'a, F>
where
    F: Fn(&mut Reader<'a>) -> Result<A, Error>,
{
    type Item = Result<A, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.index < self.0.slice.len() {
            Some(self.1(&mut self.0))
        } else {
            None
        }
    }
}

impl<'a> Reader<'a> {
    /// Create a new DER Reader where the read buffer is given by the user
    pub fn new(slice: &'a [u8]) -> Self {
        Reader { slice, index: 0 }
    }

    fn next(&mut self) -> Result<(Identifier, Length), Error> {
        let (hdr, sz) = Identifier::decode(&self.slice[self.index..]).unwrap();
        self.index += sz;
        let (len, sz) = Length::decode(&self.slice[self.index..]).unwrap();
        self.index += sz;
        Ok((hdr, len))
    }

    fn next_assume(&mut self, pc: PC, tag: u32) -> Result<Length, Error> {
        let (hdr, len) = self.next()?;
        assume(&hdr, pc, tag)?;
        Ok(len)
    }

    fn subslice(&mut self, length: Length) -> Result<&'a [u8], Error> {
        let len = match length {
            Length::Indefinite => return Err(Error::IndefiniteLengthDER),
            Length::Short(v) => v as usize,
            Length::Long { nb_bytes: _, value } => value as usize,
        };
        let sub = &self.slice[self.index..self.index + len];
        self.index += len;
        Ok(sub)
    }

    fn subslice_reader(&mut self, length: Length) -> Result<Reader<'a>, Error> {
        let slice = self.subslice(length)?;
        Ok(Self::new(slice))
    }

    /*
    fn peek(&self) -> Result<Header, Error> {
        let (hdr, _) = Header::decode(&self.slice[self.index..]).unwrap();
        Ok(hdr)
    }
    */

    pub fn anything(&mut self) -> Result<(Identifier, Length, &'a [u8]), Error> {
        let (identifier, length) = self.next()?;
        let slice = self.subslice(length)?;
        Ok((identifier, length, slice))
    }

    pub fn bool(&mut self) -> Result<bool, Error> {
        let len = self.next_assume(PC::Primitive, constants::TAG_BOOLEAN)?;
        let sub = self.subslice(len)?;
        if sub.len() == 1 {
            match sub[0] {
                0 => Ok(false),
                0xff => Ok(true),
                v => Err(Error::BoolEncodingInvalid(v)),
            }
        } else {
            Err(Error::BoolLengthInvalid(sub.len()))
        }
    }

    pub fn integer(&mut self) -> Result<&'a Integer, Error> {
        let len = self.next_assume(PC::Primitive, constants::TAG_INTEGER)?;
        let sub = self.subslice(len)?;
        let i8 = Integer8Bit::from_slice(sub).map_err(|()| Error::IntegerNotCanonical)?;
        Ok(Integer::from_inner_slice(i8))
    }

    pub fn enumerated(&mut self) -> Result<&'a Enumerated, Error> {
        let len = self.next_assume(PC::Primitive, constants::TAG_ENUMERATED)?;
        let sub = self.subslice(len)?;
        let i8 = Integer8Bit::from_slice(sub).map_err(|()| Error::IntegerNotCanonical)?;
        Ok(Enumerated::from_inner_slice(i8))
    }

    pub fn bitstring(&mut self) -> Result<&'a BitString, Error> {
        let len = self.next_assume(PC::Primitive, constants::TAG_BIT_STRING)?;
        let sub = self.subslice(len)?;
        if sub.is_empty() {
            return Err(Error::BitStringEncodingEmpty);
        }
        let bit_unused = sub[0];
        if bit_unused > 7 {
            return Err(Error::BitStringEncodingInvalidStart);
        }
        if bit_unused > 0 {
            if sub.len() == 1 {
                return Err(Error::BitStringEncodingInvalidStart);
            }
            let last = sub[sub.len() - 1];
            let mask = (1 << bit_unused) - 1;
            if last & mask != 0 {
                return Err(Error::BitStringEncodingInvalidEnd);
            }
        }
        Ok(BitString::from_raw_slice(sub))
    }

    pub fn octetstring(&mut self) -> Result<&'a [u8], Error> {
        let len = self.next_assume(PC::Primitive, constants::TAG_OCTET_STRING)?;
        let sub = self.subslice(len)?;
        Ok(sub)
    }

    pub fn utf8_string(&mut self) -> Result<&'a str, Error> {
        let len = self.next_assume(PC::Primitive, constants::TAG_OCTET_STRING)?;
        let sub = self.subslice(len)?;
        core::str::from_utf8(sub).map_err(|_| Error::Utf8Invalid)
    }

    pub fn null(&mut self) -> Result<(), Error> {
        let len = self.next_assume(PC::Primitive, constants::TAG_NULL)?;
        let sub = self.subslice(len)?;
        if !sub.is_empty() {
            return Err(Error::NullEncodingInvalid);
        }
        Ok(())
    }

    pub fn oid(&mut self) -> Result<&'a OID, Error> {
        let len = self.next_assume(PC::Primitive, constants::TAG_OID)?;
        let sub = self.subslice(len)?;
        OID::parse_from_slice(sub).map_err(|_| Error::OIDInvalid)
    }

    pub fn sequence(&mut self) -> Result<Reader<'a>, Error> {
        let len = self.next_assume(PC::Constructed, constants::TAG_SEQUENCE)?;
        self.subslice_reader(len)
    }

    pub fn set<A, F>(&mut self, f: F) -> Result<Set<'a, F>, Error>
    where
        F: Fn(Reader<'a>) -> Result<A, Error>,
    {
        let len = self.next_assume(PC::Constructed, constants::TAG_SET)?;
        let subreader = self.subslice_reader(len)?;
        Ok(Set(subreader, f))
    }

    pub fn done(&self) -> Result<(), Error> {
        if self.index == self.slice.len() {
            Ok(())
        } else {
            Err(Error::ReaderNotTerminated {
                index: self.index,
                len: self.slice.len(),
            })
        }
    }

    pub fn current_position(&self) -> usize {
        self.index
    }

    pub fn remaining(&self) -> &'a [u8] {
        &self.slice[self.index..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    pub fn decode_simple() {
        let mut reader = Reader::new(&b"\x04\x08\x01\x23\x45\x67\x89\xab\xcd\xef"[..]);
        let octets = reader.octetstring().expect("octetstring");

        assert_eq!(octets.as_ref(), &b"\x01\x23\x45\x67\x89\xab\xcd\xef"[..]);
    }

    #[test]
    pub fn decode_key() {
        let key_bs = b"\x30\x59\x30\x13\x06\x07\x2A\x86\x48\xCE\x3D\x02\x01\x06\x08\x2A\x86\x48\xCE\x3D\x03\x01\x07\x03\x42\x00\x04\xA4\x39\xEC\xD3\xCE\xAD\xFD\xDB\x8E\x50\x34\xFD\x99\x72\x45\x8C\xDC\xEB\xA9\xD3\x4E\x09\xF3\x47\x31\x4A\x48\x6C\x3C\x4E\x3C\x00\x43\x3A\x1C\x0A\x6D\xBE\xE2\xEF\x6D\x00\x8A\x10\xC9\xE3\xBE\x0F\x07\xD3\x31\x8E\x77\x44\x20\x14\xE6\x63\xC2\xAF\x19\x14\x8B\xAC";
        let mut reader = Reader::new(key_bs);
        //println!("key_bs: {}", key_bs.len());
        let mut out_sequence = reader.sequence().expect("outer sequence");
        let mut inner_sequence = out_sequence.sequence().expect("inner sequence");

        let oid1 = inner_sequence.oid().expect("oid1");
        assert_eq!(oid1.value1(), 1, "OID1 compoment 1");
        assert_eq!(oid1.value2(), 2, "OID1 component 2");
        let trailing: Vec<u64> = oid1
            .components()
            .map(|comp| comp.to_u64().unwrap())
            .collect();
        assert_eq!(&trailing, &[840, 10045, 2, 1]);
        let oid2 = inner_sequence.oid().expect("oid2");
        assert_eq!(oid2.value1(), 1, "OID2 component 1");
        assert_eq!(oid2.value2(), 2, "OID2 component 2");
        let trailing: Vec<u64> = oid2
            .components()
            .map(|comp| comp.to_u64().unwrap())
            .collect();
        assert_eq!(&trailing, &[840, 10045, 3, 1, 7]);

        let bits = out_sequence.bitstring().expect("bitstring");
        assert_eq!(bits.bits(), 520);
    }
}
