#![no_std]

extern crate alloc;

mod borrowed;
mod inner;
mod owned;

pub mod ascending;
pub mod descending;

pub use borrowed::Borrowed as Slice;
pub use owned::Owned as Set;
