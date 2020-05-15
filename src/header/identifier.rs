/// Class for encoding
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Universal,
    Application,
    Context,
    Private,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PC {
    Constructed,
    Primitive,
}

enum TagType {
    Short(u8), // u8 < 0x1f
    Long,      // u8 = 0x1f
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum TagEncoded {
    Short(u8),
    Long(u32),
}

impl TagEncoded {
    pub fn value(self) -> u32 {
        match self {
            TagEncoded::Short(u) => u as u32,
            TagEncoded::Long(l) => l,
        }
    }

    pub fn new_smallest(v: u32) -> Self {
        if v < 0x80 {
            TagEncoded::Short(v as u8)
        } else {
            TagEncoded::Long(v)
        }
    }
}

/// decode the first byte in the following format:
///
/// CL CL CON T T T T T
fn decode_first_byte(hdr: u8) -> (Class, PC, TagType) {
    let class = match hdr >> 6 {
        0b00 => Class::Universal,
        0b01 => Class::Application,
        0b10 => Class::Context,
        0b11 => Class::Private,
        _ => unreachable!(),
    };
    let constructed = if (hdr & 0b0010_0000) != 0 {
        PC::Constructed
    } else {
        PC::Primitive
    };
    let tag = hdr & 0b1_1111;
    let tagtype = if tag == 0b1_1111 {
        TagType::Long
    } else {
        TagType::Short(tag)
    };
    (class, constructed, tagtype)
}

fn encode_first_byte(class: Class, constructed: PC, tagtype: TagType) -> u8 {
    let e1 = match class {
        Class::Universal => 0b00,
        Class::Application => 0b01,
        Class::Context => 0b10,
        Class::Private => 0b11,
    };
    let e2 = if constructed == PC::Constructed {
        0b0010_0000
    } else {
        0
    };
    let e3 = match tagtype {
        TagType::Short(v) => v,
        TagType::Long => 0b1_1111,
    };
    e1 << 6 | e2 | e3
}

type Tag = u32;

/// ASN.1 BER/CER/DER Identifier
///
/// The header consist of the ASN.1 class,
/// the Primitive/Construction boolean, and the tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub class: Class,
    pub pc: PC,
    pub tag: TagEncoded,
}

#[derive(Debug)]
pub enum DecodeError {
    EmptyHeader,
    TagEncodingIncomplete,
    TagEncodingOverflow,
    TagEncodingNonCanonical,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputBufferTooSmall;

impl Identifier {
    pub fn decode(slice: &[u8]) -> Result<(Self, usize), DecodeError> {
        if slice.is_empty() {
            return Err(DecodeError::EmptyHeader);
        }

        let (class, pc, tagtype) = decode_first_byte(slice[0]);
        let mut index = 1;

        let tag = match tagtype {
            TagType::Short(tag) => TagEncoded::Short(tag),
            TagType::Long => TagEncoded::Long(get_taglong(slice, &mut index)?),
        };

        Ok((Identifier { class, pc, tag }, index))
    }

    pub fn encode(&self, out: &mut [u8]) -> usize {
        let tagtype = match self.tag {
            TagEncoded::Short(v) => TagType::Short(v),
            TagEncoded::Long(_) => TagType::Long,
        };
        let x = encode_first_byte(self.class, self.pc, tagtype);

        out[0] = x;
        let mut index = 1;

        match self.tag {
            TagEncoded::Short(_) => {} // already done
            TagEncoded::Long(b) => {
                let nb_bytes = size_7bit(b);
                for i in 0..nb_bytes {
                    let shifter = 7 * (nb_bytes - 1 - i);
                    let v = ((b >> shifter) & 0x7f) as u8;
                    if i == nb_bytes - 1 {
                        out[index] = v;
                    } else {
                        out[index] = v | 0x80;
                    }
                    index += 1;
                }
            }
        };
        index
    }

    pub fn size_bytes(&self) -> usize {
        match self.tag {
            TagEncoded::Long(v) => 1 + size_7bit(v),
            TagEncoded::Short(_) => 1,
        }
    }
}

fn get_taglong(slice: &[u8], index: &mut usize) -> Result<u32, DecodeError> {
    let mut acc: Tag = 0u32;
    let mut first_byte = true;
    loop {
        let byte: u8 = *slice
            .get(*index)
            .ok_or(DecodeError::TagEncodingIncomplete)?;
        *index += 1;
        if (byte & 0b1000_0000) != 0 {
            let cbyte = byte & 0b0111_1111;

            if first_byte && cbyte == 0 {
                break Err(DecodeError::TagEncodingNonCanonical);
            }
            first_byte = false;

            acc = acc
                .checked_shl(7)
                .ok_or(DecodeError::TagEncodingOverflow)?
                .checked_add(Tag::from(cbyte))
                .ok_or(DecodeError::TagEncodingOverflow)?;
        } else {
            acc = acc
                .checked_shl(7)
                .ok_or(DecodeError::TagEncodingOverflow)?
                .checked_add(Tag::from(byte))
                .ok_or(DecodeError::TagEncodingOverflow)?;
            break Ok(acc);
        }
    }
}

fn size_7bit(mut v: u32) -> usize {
    let mut nb_bytes = 1;
    while v >= 0x80 {
        v >>= 7;
        nb_bytes += 1;
    }
    nb_bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    fn decode_encode(header: &Identifier) -> Result<Identifier, String> {
        let mut buf = [0u8; 32];
        let sz = header.encode(&mut buf);
        match Identifier::decode(&buf[0..sz]) {
            Err(e) => Err(format!("decoding error {:?} for {:?}", e, header)),
            Ok((hdr, dsz)) => {
                if dsz == sz {
                    Ok(hdr)
                } else {
                    Err(format!(
                        "decoded size {} is different from encoded {}",
                        dsz, sz
                    ))
                }
            }
        }
    }

    #[test]
    fn decode_encode_headers() {
        let mut hdr = Identifier {
            class: Class::Universal,
            pc: PC::Primitive,
            tag: TagEncoded::Short(1),
        };

        for tag_short in 0..0x1f {
            hdr.tag = TagEncoded::Short(tag_short);
            let new_hdr = decode_encode(&hdr).unwrap();
            assert_eq!(new_hdr, hdr)
        }

        for tag_long in 1..0x3f {
            hdr.tag = TagEncoded::Long(tag_long);
            let new_hdr = decode_encode(&hdr).unwrap();
            assert_eq!(new_hdr, hdr)
        }
    }

    #[test]
    fn decode_encode_oddtag() {
        let hdr = Identifier {
            class: Class::Context,
            pc: PC::Primitive,
            tag: TagEncoded::Long(0x12482),
        };
        let new_hdr = decode_encode(&hdr).unwrap();
        assert_eq!(new_hdr, hdr)
    }
}
