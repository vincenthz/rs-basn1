//! ASN.1 binary encoder and decoder (DER, possibly BER/CER in future)
//!
//! The general principle of this crate is to avoid transforming information
//! or re-allocating information. this should be compatible with no_std.
//!
//! In decoding, the user remain in control of the data allocation and instead
//! the parser give typed view into this data allocation with the data verified
//! for correctness.
//!
//! For example, when reading an integer from the stream, a typed slice `IntegerSlice` of the stream
//! is given back that the parser guaranteed to be correct, and that the user can keep as is.

#![no_std]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

mod header;

pub mod der;

#[macro_use]
mod coretm;
mod intenc;
mod objects;

pub use objects::*;
