/// ASN.1 Header Length has 3 differents encoding
///
/// * Short : 1 byte, for any raw length value less than < 0x80 bytes
/// * Indefinite : 1 byte, for encoding an unknown length value
/// * Long : 2 to 2+127 bytes, for encoding a length > 0x80 bytes
///
/// For long encoding, the maximum allowed length is 32 bits which result
/// in a maximum size for every given element of 4 gb.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Length {
    Short(u8),
    Long { nb_bytes: u8, value: u32 },
    Indefinite,
}

/// Length decoding error
#[derive(Clone, Debug)]
pub enum LengthDecodeError {
    EncodingIncomplete,
    /// Length encoded is bigger than the reasonable limit
    EncodingOverflow,
}

impl Length {
    pub fn value(&self) -> Option<u32> {
        match self {
            Length::Short(sz) => Some(*sz as u32),
            Length::Long { nb_bytes: _, value } => Some(*value),
            Length::Indefinite => None,
        }
    }

    pub fn new_smallest(v: usize) -> Self {
        if v < 0x80 {
            Self::Short(v as u8)
        } else {
            let usz = core::mem::size_of::<usize>();
            let bits_clear = v.leading_zeros();
            let nb_bytes = (usz as u32 - bits_clear / 8) as u8;
            Self::Long {
                nb_bytes,
                value: v as u32,
            }
        }
    }

    pub fn size_bytes(&self) -> usize {
        match self {
            Length::Indefinite => 1,
            Length::Short(_) => 1,
            Length::Long { nb_bytes, value: _ } => 1 + *nb_bytes as usize,
        }
    }

    pub fn encode(&self, out: &mut [u8]) {
        put_length(self, out)
    }

    pub fn decode(buf: &[u8]) -> Result<(Self, usize), LengthDecodeError> {
        get_length(buf)
    }
}

// length encoding is either 0x80 for indefinite, anything less is a short encoding,
// and anything above give the number of byte
fn get_length(slice: &[u8]) -> Result<(Length, usize), LengthDecodeError> {
    if slice.is_empty() {
        return Err(LengthDecodeError::EncodingIncomplete);
    }

    let f = slice[0];

    if f == 0b1000_0000 {
        Ok((Length::Indefinite, 1))
    } else if (f & 0b1000_0000) != 0 {
        let nb_bytes = f & 0b0111_1111;
        let mut acc = 0u32;

        let total_size = 1 + nb_bytes as usize;
        if slice.len() < total_size {
            return Err(LengthDecodeError::EncodingIncomplete);
        }

        let mut index = 1;
        for _ in 0..nb_bytes {
            let b: u8 = slice[index];
            index += 1;
            acc = acc
                .checked_shl(8)
                .ok_or(LengthDecodeError::EncodingOverflow)?
                .checked_add(u32::from(b))
                .ok_or(LengthDecodeError::EncodingOverflow)?;
        }
        let len = Length::Long {
            nb_bytes,
            value: acc,
        };
        Ok((len, total_size))
    } else {
        Ok((Length::Short(f), 1))
    }
}

fn put_length(len: &Length, out: &mut [u8]) {
    match len {
        Length::Indefinite => out[0] = 0x80,
        Length::Short(v) => {
            assert!(*v < 0x80);
            out[0] = *v;
        }
        Length::Long {
            mut nb_bytes,
            value,
        } => {
            assert_ne!(nb_bytes, 0);
            assert!(nb_bytes < 0x80);

            out[0] = 0b1000_0000 | nb_bytes;
            let mut index = 1;
            while nb_bytes > 4 {
                out[index] = 0;
                index += 1;
                nb_bytes -= 1;
            }

            let value_be = value.to_be_bytes();
            while nb_bytes > 0 {
                out[index] = value_be[4 - nb_bytes as usize];
                index += 1;
                nb_bytes -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    fn decode_encode(len: &Length) -> Result<Length, String> {
        let mut buf = [0u8; 32];
        //let sz = len.encode(&mut buf);
        let sz = len.size_bytes();
        len.encode(&mut buf);

        match Length::decode(&buf[0..sz]) {
            Err(e) => Err(format!("decoding error {:?} for {:?}", e, len)),
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
    fn decode_encode_length() {
        for v in &[
            1usize, 10, 32, 43, 46, 56, 80, 88, 92, 102, 140, 200, 340, 359, 469, 699, 999, 1001,
            1394, 2149214, 241421421,
        ] {
            let length = Length::new_smallest(*v);
            let new_length = decode_encode(&length).unwrap();
            assert_eq!(new_length, length)
        }
    }
}
