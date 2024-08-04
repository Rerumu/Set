use alloc::{boxed::Box, vec::Vec};

use crate::{ascending::Ascending, borrowed::Borrowed, descending::Descending, inner::Inner};

/// An owned set of natural numbers.
pub struct Owned {
	data: Box<[Inner]>,
	len: usize,
}

impl Owned {
	/// Returns a new instance of the set that accomodates no values.
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		const DATA: Box<[Inner; 0]> = unsafe {
			core::mem::transmute::<core::ptr::NonNull<[Inner; 0]>, _>(core::ptr::NonNull::dangling())
		};

		Self { data: DATA, len: 0 }
	}

	/// Returns a new instance of the set that accomodates values `0..maximum`.
	#[inline]
	#[must_use]
	pub fn with_maximum(maximum: usize) -> Self {
		let data = alloc::vec![0; maximum].into_boxed_slice();

		Self { data, len: 0 }
	}

	/// Returns a lightweight borrow of the set.
	#[inline]
	#[must_use]
	pub const fn as_slice(&self) -> Borrowed {
		// SAFETY: `len` correctly tracks how many set bits exist.
		unsafe { Borrowed::new(&self.data, self.len) }
	}

	/// Returns the largest value the set can store.
	#[inline]
	#[must_use]
	pub const fn maximum(&self) -> usize {
		self.as_slice().maximum()
	}

	/// Returns the number of values in the set.
	#[inline]
	#[must_use]
	pub const fn len(&self) -> usize {
		self.as_slice().len()
	}

	/// Returns whether the set contains any value.
	#[inline]
	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.as_slice().is_empty()
	}

	/// Returns whether the set contains the given value.
	#[inline]
	#[must_use]
	pub const fn contains(&self, value: usize) -> bool {
		self.as_slice().contains(value)
	}

	/// Returns an ascending iterator over the stored values.
	#[inline]
	pub const fn ascending(&self) -> Ascending {
		self.as_slice().ascending()
	}

	/// Returns a descending iterator over the stored values.
	#[inline]
	pub const fn descending(&self) -> Descending {
		self.as_slice().descending()
	}

	/// Removes all values from the set in bulk.
	#[inline]
	pub fn clear(&mut self) {
		self.len = 0;

		self.data.fill(0);
	}

	#[inline(never)]
	fn with_buffer<H: Fn(&mut Vec<Inner>)>(&mut self, handler: H) {
		let mut data = core::mem::take(&mut self.data).into_vec();

		handler(&mut data);

		self.data = data.into_boxed_slice();
	}

	/// Grows the set to accomodate at least the values `0..maximum`.
	#[inline]
	pub fn grow_maximum(&mut self, maximum: usize) {
		let offset = crate::inner::bits_to_chunk(maximum);

		if offset >= self.data.len() {
			self.with_buffer(move |data| {
				data.reserve(offset + 1 - data.capacity());
				data.resize(data.capacity(), 0);
			});
		}
	}

	/// Shrinks the set to accomodate at most the values it has.
	#[inline]
	pub fn shrink_to_fit(&mut self) {
		let maximum = self
			.data
			.iter()
			.rposition(|&value| value != 0)
			.map_or(0, |index| index + 1);

		self.with_buffer(move |data| data.truncate(maximum));
	}

	/// Inserts the given index into the set and returns the previous state.
	#[inline]
	pub fn insert(&mut self, value: usize) -> Option<bool> {
		let offset = crate::inner::bits_to_chunk(value);

		self.data.get_mut(offset).map(|inner| {
			if crate::inner::get(*inner, value) {
				true
			} else {
				*inner |= crate::inner::mask(value);

				self.len += 1;

				false
			}
		})
	}

	/// Removes the given index from the set and returns the previous state.
	#[inline]
	pub fn remove(&mut self, value: usize) -> Option<bool> {
		let offset = crate::inner::bits_to_chunk(value);

		self.data.get_mut(offset).map(|inner| {
			if crate::inner::get(*inner, value) {
				*inner &= !crate::inner::mask(value);

				self.len -= 1;

				true
			} else {
				false
			}
		})
	}

	/// Inserts the given index into the set and returns the previous state.
	#[inline]
	pub fn grow_insert(&mut self, value: usize) -> bool {
		self.grow_maximum(value + 1);

		unsafe { self.insert(value).unwrap_unchecked() }
	}

	/// Clones the data from `source` without allocating if possible.
	#[inline]
	pub fn clone_from_slice(&mut self, source: Borrowed) {
		self.with_buffer(move |data| {
			data.clear();
			data.extend_from_slice(source.data);
		});

		self.len = source.len;
	}
}

impl Default for Owned {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for Owned {
	#[inline]
	fn clone(&self) -> Self {
		Self {
			data: self.data.clone(),
			len: self.len,
		}
	}

	#[inline]
	fn clone_from(&mut self, source: &Self) {
		let source = source.as_slice();

		self.clone_from_slice(source);
	}
}

impl PartialEq for Owned {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		self.as_slice().eq(&other.as_slice())
	}
}

impl Eq for Owned {}

impl PartialOrd for Owned {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Owned {
	#[inline]
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.as_slice().cmp(&other.as_slice())
	}
}

impl Extend<usize> for Owned {
	#[inline]
	fn extend<T: IntoIterator<Item = usize>>(&mut self, iter: T) {
		iter.into_iter().for_each(|index| {
			self.grow_insert(index);
		});
	}
}

impl FromIterator<usize> for Owned {
	#[inline]
	fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
		let mut set = Self::new();

		set.extend(iter);

		set
	}
}

impl core::fmt::Debug for Owned {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		self.as_slice().fmt(f)
	}
}

#[cfg(test)]
mod test {
	use alloc::vec::Vec;

	use super::Owned;

	#[test]
	fn values_iteration() {
		let mut set = Owned::new();
		let mut list = Vec::new();

		for index in 0..10 {
			let value = 2_usize.pow(index);

			set.grow_insert(value);
			list.push(value);
		}

		assert!(
			set.ascending().eq(list.iter().copied()),
			"ascending list should be ordered"
		);

		assert!(
			set.descending().eq(list.iter().copied().rev()),
			"descending list should be ordered"
		);
	}
}
