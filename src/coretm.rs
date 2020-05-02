//! core type and macros
//!
//! this module is not exported by the crate as it contains
//! way to infringe on some type guarantees, and thus should
//! be use with care
//!

// transform a variable $name from type [u8] to $typ
macro_rules! cast_slice_u8_to_typed_slice {
    ($name: ident, $typ: ident) => {
        unsafe { &*($name as *const [u8] as *const $typ) }
    };
}

macro_rules! slice_reexport_asref {
    ($name: ident) => {
        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                &self.0.as_ref()
            }
        }
    };
}

macro_rules! slice_owned_mapping {
    ($ty: ident, $slice: ident) => {
        #[cfg(feature = "owned")]
        impl core::borrow::Borrow<$slice> for $ty {
            fn borrow(&self) -> &$slice {
                $slice::from_raw_slice(&self.0[..])
            }
        }
        #[cfg(feature = "owned")]
        impl alloc::borrow::ToOwned for $slice {
            type Owned = $ty;
            fn to_owned(&self) -> Self::Owned {
                $ty(self.0.to_owned())
            }
        }
        #[cfg(feature = "owned")]
        impl core::ops::Deref for $ty {
            type Target = $slice;

            fn deref(&self) -> &Self::Target {
                $slice::from_raw_slice(&self.0[..])
            }
        }
    };
}

// define type $name and $slice
macro_rules! define_typed_vec_and_slice {
    ($name: ident, $slice: ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[cfg(feature = "owned")]
        pub struct $name(Vec<u8>);

        #[derive(Debug, PartialEq, Eq, Hash)]
        pub struct $slice([u8]);
    };
}

macro_rules! method_reslice_cast {
    ($name: ident, $slice: ident) => {
        impl $name {
            /// unsafe method only available from internal module
            pub(crate) fn from_inner_slice<'a>(slice: &'a $slice) -> &'a $name {
                unsafe { &*(slice as *const $slice as *const $name) }
            }
        }
    };
}
