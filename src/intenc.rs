//! Integer encoding used in ASN.1
//!
//! * 7bit highest-continuation encoding
//! * 8bit variable encoding encoding slice

/// A simple encoded variable size integer where limbs
/// are 7 bits, and big endian, and continuation of
/// the encoding is indicated by having the highest bit set
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct IntegerContBit7([u8]);

macro_rules! to_primitive7 {
    ($type: ident, $name: ident) => {
        /// Try to convert to the primitive
        ///
        /// If there's an overflown then nothing is returned
        pub fn $name(&self) -> Option<$type> {
            // this function assume that the data has been checked properly
            // so that the first byte is not a long zero,
            // and that the continuation bit are correctly set
            // for each byte limb.
            let mut acc = (self.0[0] & 0b0111_1111) as $type;
            for c in &self.0[1..] {
                acc = acc
                    .checked_shl(7)?
                    .checked_add((c & 0b0111_1111) as $type)?
            }
            Some(acc)
        }
    };
}

impl IntegerContBit7 {
    /// transform a raw slice into a IntegerContBit7 slice,
    /// no verification is done by this call
    /// one should use parse_from_slice for safe parsing+verification
    pub(crate) fn unverified_from_slice(slice: &[u8]) -> &Self {
        cast_slice_u8_to_typed_slice!(slice, Self)
    }

    /// Try to parse from a slice
    pub fn parse_from_slice(slice: &[u8]) -> Result<(&Self, usize), ()> {
        if slice.is_empty() {
            return Err(());
        }
        if slice[0] == 0b1000_0000 {
            return Err(());
        }
        let mut i = 0;
        while (slice[i] & 0b1000_0000) != 0 {
            i += 1;
            if i == slice.len() {
                return Err(());
            }
        }
        let r = Self::unverified_from_slice(&slice[0..1 + i]);
        Ok((r, 1 + i))
    }

    to_primitive7!(u128, to_u128);
    to_primitive7!(u64, to_u64);
    to_primitive7!(u32, to_u32);
    to_primitive7!(u16, to_u16);
    to_primitive7!(u8, to_u8);

    /*
    pub fn as_be() -> BeIntegerBytes<'a> {
        todo!()
    }

    pub fn as_le() -> LeIntegerBytes<'a> {
        todo!()
    }
    */
}

slice_reexport_asref!(IntegerContBit7);

/// An encoded integer where each limbs is 8bits and in big endian
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct Integer8Bit([u8]);

macro_rules! to_primitive8 {
    ($type: ident, $name: ident) => {
        /// Try to convert to the primitive
        ///
        /// If there's an overflown then nothing is returned
        pub fn $name(&self) -> Option<$type> {
            // this function assume that the data has been checked properly
            // so that the first byte is not a long zero,
            // and that the continuation bit are correctly set
            // for each byte limb.
            let mut acc = self.0[0] as $type;
            for c in &self.0[1..] {
                acc = acc.checked_shl(8)?.checked_add(*c as $type)?
            }
            Some(acc)
        }
    };
}

impl Integer8Bit {
    /// transform a raw slice into a Integer8Bit slice,
    /// no verification is done by this call
    pub(crate) fn unverified_from_slice(slice: &[u8]) -> &Self {
        cast_slice_u8_to_typed_slice!(slice, Self)
    }

    /// Try to parse from a slice
    pub fn from_slice(slice: &[u8]) -> Result<&Self, ()> {
        if slice.is_empty() || slice[0] == 0 {
            return Err(());
        }
        Ok(Self::unverified_from_slice(slice))
    }

    to_primitive8!(u128, to_u128);
    to_primitive8!(u64, to_u64);
    to_primitive8!(u32, to_u32);
    to_primitive8!(u16, to_u16);
    to_primitive8!(u8, to_u8);

    /*
    pub fn as_be() -> BeIntegerBytes<'a> {}

    pub fn as_le() -> LeIntegerBytes<'a> {
        todo!()
    }
    */
}

slice_reexport_asref!(Integer8Bit);
