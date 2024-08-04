use crate::{ascending::Ascending, descending::Descending, inner::Inner};

/// A borrowed set of natural numbers.
#[derive(Clone, Copy)]
pub struct Borrowed<'data> {
	pub(crate) data: &'data [Inner],
	pub(crate) len: usize,
}

impl<'data> Borrowed<'data> {
	///  Returns a new instance of the borrowed set.
	///
	/// # Safety
	///
	/// The caller ensures that there are at least `len` bits
	/// set to 1 within the array `data`.
	#[inline]
	#[must_use]
	pub const unsafe fn new(data: &'data [Inner], len: usize) -> Self {
		Self { data, len }
	}

	/// Returns the largest value the set can store.
	#[inline]
	#[must_use]
	pub const fn maximum(self) -> usize {
		crate::inner::chunk_to_bits(self.data.len())
	}

	/// Returns the number of values in the set.
	#[inline]
	#[must_use]
	pub const fn len(self) -> usize {
		self.len
	}

	/// Returns whether the set contains any value.
	#[inline]
	#[must_use]
	pub const fn is_empty(self) -> bool {
		self.len() == 0
	}

	/// Returns whether the set contains the given value.
	#[inline]
	#[must_use]
	pub const fn contains(self, index: usize) -> bool {
		let offset = crate::inner::bits_to_chunk(index);

		offset < self.data.len() && crate::inner::get(self.data[offset], index)
	}

	/// Returns an ascending iterator over the stored values.
	#[inline]
	pub const fn ascending(self) -> Ascending<'data> {
		// SAFETY: `len` correctly tracks how many set bits exist.
		unsafe { Ascending::new(self.data, self.len) }
	}

	/// Returns a descending iterator over the stored values.
	#[inline]
	pub const fn descending(self) -> Descending<'data> {
		// SAFETY: `len` correctly tracks how many set bits exist.
		unsafe { Descending::new(self.data, self.len) }
	}
}

impl<'data> PartialEq for Borrowed<'data> {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		(self.len == other.len) && self.ascending().eq(other.ascending())
	}
}

impl<'data> Eq for Borrowed<'data> {}

impl<'data> PartialOrd for Borrowed<'data> {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'data> Ord for Borrowed<'data> {
	#[inline]
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.ascending().cmp(other.ascending())
	}
}

impl<'data> IntoIterator for Borrowed<'data> {
	type Item = usize;
	type IntoIter = Ascending<'data>;

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		self.ascending()
	}
}

impl<'data> core::fmt::Debug for Borrowed<'data> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		self.ascending().fmt(f)
	}
}
