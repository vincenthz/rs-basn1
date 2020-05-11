//use crate::coretm::slice_reexport_asref;
use crate::intenc::{Integer8Bit, IntegerContBit7};

#[cfg(feature = "owned")]
use alloc::vec::Vec;

macro_rules! typed_vec_and_slice {
    ($name: ident, $slice: ident) => {
        define_typed_vec_and_slice!($name, $slice);
        slice_owned_mapping!($name, $slice);
        slice_reexport_asref!($slice);

        impl $slice {
            /// unsafe method only available from internal module
            pub(crate) fn from_raw_slice<'a>(slice: &'a [u8]) -> &'a $slice {
                cast_slice_u8_to_typed_slice!(slice, $slice)
            }
        }
    };
}

macro_rules! type_reslice {
    ($name: ident, $slice: ident) => {
        #[derive(Debug, PartialEq, Eq, Hash)]
        pub struct $name($slice);

        method_reslice_cast!($name, $slice);
        slice_reexport_asref!($name);
    };
}

macro_rules! type_slice_integer_method {
    ($name: ident) => {
        impl $name {
            pub fn to_u128(&self) -> Option<u128> {
                self.0.to_u128()
            }
            pub fn to_u64(&self) -> Option<u64> {
                self.0.to_u64()
            }
            pub fn to_u32(&self) -> Option<u32> {
                self.0.to_u32()
            }
            pub fn to_u16(&self) -> Option<u16> {
                self.0.to_u16()
            }
            pub fn to_u8(&self) -> Option<u8> {
                self.0.to_u8()
            }
        }
    };
}

typed_vec_and_slice!(BitStringOwned, BitString);
//typed_vec_and_slice!(IA5StringOwned, IA5String);
typed_vec_and_slice!(OIDOwned, OID);

type_reslice!(OIDComponent, IntegerContBit7);
type_slice_integer_method!(OIDComponent);

type_reslice!(Integer, Integer8Bit);
type_slice_integer_method!(Integer);

type_reslice!(Enumerated, Integer8Bit);
type_slice_integer_method!(Enumerated);

impl BitString {
    /// Return the total number of bits of the bitstring
    pub fn bits(&self) -> usize {
        let bits_unused = self.0[0];
        let bits = (self.0.len() - 1) * 8;
        bits - (bits_unused as usize)
    }

    /// Return the total number of unused bits on the bitstring
    pub fn bits_unused(&self) -> usize {
        self.0[0] as usize
    }

    /// Get the data associated with the bitstring as full bytes
    pub fn data_bytes(&self) -> &[u8] {
        &self.0[1..]
    }
}

#[derive(Debug, Clone)]
pub struct OIDComponents<'a> {
    slice: &'a OID,
    index: usize,
}

impl OID {
    pub fn value1(&self) -> u8 {
        self.0[0] / 40
    }

    pub fn value2(&self) -> u8 {
        self.0[0] % 40
    }

    /// Return all trailing components, except the first and second value
    pub fn components(&self) -> OIDComponents<'_> {
        OIDComponents {
            slice: self,
            index: 1,
        }
    }

    pub fn parse_from_slice<'a>(slice: &'a [u8]) -> Result<&'a Self, ()> {
        if slice.is_empty() {
            return Err(());
        }
        let f = slice[0];
        // only 0, 1, and 2 are allowed.
        if (f / 40) > 2 {
            return Err(());
        }
        let mut index = 1;
        while index < slice.len() {
            match IntegerContBit7::parse_from_slice(&slice[index..]) {
                Err(_) => return Err(()),
                Ok((_, adv)) => index += adv,
            }
        }
        // this really cannot happen, but check for extra safety
        if index > slice.len() {
            return Err(());
        }

        Ok(Self::from_raw_slice(slice))
    }
}

impl<'a> Iterator for OIDComponents<'a> {
    type Item = &'a OIDComponent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.slice.0.len() {
            None
        } else {
            match IntegerContBit7::parse_from_slice(&self.slice.0[self.index..]) {
                Err(_) => unreachable!(),
                Ok((r, adv)) => {
                    self.index += adv;
                    return Some(OIDComponent::from_inner_slice(r));
                }
            }
        }
    }
}
