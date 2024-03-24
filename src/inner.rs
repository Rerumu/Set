use core::iter::FusedIterator;

pub type Inner = u64;

#[allow(clippy::cast_possible_truncation)]
const BITS_U8: u8 = Inner::BITS as u8;
const BITS_USIZE: usize = Inner::BITS as usize;

#[inline]
pub const fn bits_to_chunk(index: usize) -> usize {
	index / BITS_USIZE
}

#[inline]
pub const fn chunk_to_bits(index: usize) -> usize {
	index * BITS_USIZE
}

#[inline]
pub const fn mask(bit: usize) -> Inner {
	1 << (bit % BITS_USIZE)
}

#[inline]
pub const fn get(inner: Inner, bit: usize) -> bool {
	inner & mask(bit) != 0
}

#[derive(Clone)]
pub struct Iter {
	inner: Inner,
	remaining: u8,
}

impl Iter {
	#[allow(clippy::cast_possible_truncation)]
	#[inline]
	pub const fn new(inner: Inner) -> Self {
		Self {
			inner,
			remaining: inner.count_ones() as u8,
		}
	}
}

impl Iterator for Iter {
	type Item = u8;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.remaining > BITS_U8 {
			// SAFETY: `count_ones` will not return more bits than the
			// size of the integer.
			unsafe { core::hint::unreachable_unchecked() };
		}

		self.remaining = self.remaining.checked_sub(1)?;

		let position = self.inner.trailing_zeros().try_into().unwrap();

		self.inner &= self.inner - 1;

		Some(position)
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.remaining as usize;

		(len, Some(len))
	}
}

impl DoubleEndedIterator for Iter {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.remaining > BITS_U8 {
			// SAFETY: `count_ones` will not return more bits than the
			// size of the integer.
			unsafe { core::hint::unreachable_unchecked() };
		}

		self.remaining = self.remaining.checked_sub(1)?;

		let position: u8 = self.inner.leading_zeros().try_into().unwrap();

		self.inner &= (Inner::MAX >> 1) >> u32::from(position);

		Some(BITS_U8 - position - 1)
	}
}

impl ExactSizeIterator for Iter {}

impl FusedIterator for Iter {}

#[cfg(test)]
mod test {
	use alloc::vec::Vec;

	use super::{Inner, Iter};

	#[test]
	fn ones_ordered() {
		let list_1: Vec<_> = Iter::new(Inner::MAX).collect();
		let list_2: Vec<_> = Iter::new(Inner::MAX).rev().collect();

		assert!(
			list_1.iter().eq(list_2.iter().rev()),
			"list should be equal from both sides"
		);
	}
}
