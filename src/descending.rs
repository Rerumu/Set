use core::{iter::FusedIterator, marker::PhantomData};

use crate::inner::{Inner, Iter};

/// A descending iterator over values in a set.
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Descending<'data> {
	start: *const Inner,
	end: usize,
	len: usize,
	cached: Iter,

	_phantom: PhantomData<&'data [Inner]>,
}

impl<'data> Descending<'data> {
	/// Returns a new iterator over the set values.
	///
	/// # Safety
	///
	/// The caller ensures that there are at least `len` bits
	/// set to 1 within the array `data`.
	#[inline]
	pub const unsafe fn new(data: &'data [Inner], remaining: usize) -> Self {
		Self {
			start: data.as_ptr(),
			end: data.len(),
			len: remaining,
			cached: Iter::new(0),
			_phantom: PhantomData,
		}
	}

	#[inline]
	unsafe fn find_non_zero(&mut self) {
		loop {
			self.end -= 1;

			let inner = unsafe { self.start.add(self.end).read() };

			if inner != 0 {
				self.cached = Iter::new(inner);

				break;
			}
		}
	}
}

impl<'data> Iterator for Descending<'data> {
	type Item = usize;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.len = self.len.checked_sub(1)?;

		if self.cached.len() == 0 {
			// SAFETY: We have at least 1 bit left and the current chunk does
			// not contain it.
			unsafe { self.find_non_zero() };
		}

		self.cached
			.next_back()
			.map(usize::from)
			.map(|index| crate::inner::chunk_to_bits(self.end) + index)
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.len, Some(self.len))
	}
}

impl<'data> ExactSizeIterator for Descending<'data> {}

impl<'data> FusedIterator for Descending<'data> {}

impl<'data> core::fmt::Debug for Descending<'data> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_list().entries(self.clone()).finish()
	}
}
