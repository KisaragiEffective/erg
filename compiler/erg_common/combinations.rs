// Copied and modified from https://github.com/rust-itertools/itertools/blob/master/src/combinations.rs
// and https://github.com/rust-itertools/itertools/blob/master/src/impl_macros.rs
// MIT license | Apache 2.0 license
// License files are placed at the root.

use std::fmt;
use std::iter::FusedIterator;

use super::lazy_buffer::LazyBuffer;

pub struct TotalCombinations<I: Iterator> {
    combinations: Combinations<I>,
    len: usize,
}

/// ```
/// let it = total_combinations(0..2);
/// itertools::assert_equal(it, vec![
///     vec![0],
///     vec![1],
///     vec![2],
///     vec![0, 1],
///     vec![0, 2],
///     vec![1, 2],
///     vec![0, 1, 2],
/// ]);
///
pub fn total_combinations<I>(iter: I) -> TotalCombinations<I>
where
    I: Iterator + ExactSizeIterator,
    I::Item: Clone,
{
    TotalCombinations {
        len: iter.len(),
        combinations: combinations(iter, 1),
    }
}

impl<I> Iterator for TotalCombinations<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.combinations.next() {
            Some(i)
        } else {
            self.combinations.reset(self.combinations.k() + 1);
            self.len -= 1;
            if self.len == 0 {
                return None;
            }
            self.combinations.next()
        }
    }
}

impl<I> FusedIterator for TotalCombinations<I>
where
    I: Iterator,
    I::Item: Clone,
{
}

macro_rules! debug_fmt_fields {
    ($tyname:ident, $($($field:tt/*TODO ideally we would accept ident or tuple element here*/).+),*) => {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            f.debug_struct(stringify!($tyname))
                $(
              .field(stringify!($($field).+), &self.$($field).+)
              )*
              .finish()
        }
    }
}

macro_rules! clone_fields {
    ($($field:ident),*) => {
        fn clone(&self) -> Self {
            Self {
                $($field: self.$field.clone(),)*
            }
        }
    }
}

/// An iterator to iterate through all the `k`-length combinations in an iterator.
///
/// See [`.combinations()`](crate::Itertools::combinations) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Combinations<I: Iterator> {
    indices: Vec<usize>,
    pool: LazyBuffer<I>,
    first: bool,
}

impl<I> Clone for Combinations<I>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    clone_fields!(indices, pool, first);
}

impl<I> fmt::Debug for Combinations<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(Combinations, indices, pool, first);
}

/// Create a new `Combinations` from a clonable iterator.
pub fn combinations<I>(iter: I, k: usize) -> Combinations<I>
where
    I: Iterator,
{
    let mut pool = LazyBuffer::new(iter);
    pool.prefill(k);

    Combinations {
        indices: (0..k).collect(),
        pool,
        first: true,
    }
}

impl<I: Iterator> Combinations<I> {
    /// Returns the length of a combination produced by this iterator.
    #[inline]
    pub fn k(&self) -> usize {
        self.indices.len()
    }

    /// Returns the (current) length of the pool from which combination elements are
    /// selected. This value can change between invocations of [`next`](Combinations::next).
    #[inline]
    pub fn n(&self) -> usize {
        self.pool.len()
    }

    /// Returns a reference to the source iterator.
    #[inline]
    pub fn src(&self) -> &I {
        &self.pool.it
    }

    /// Resets this `Combinations` back to an initial state for combinations of length
    /// `k` over the same pool data source. If `k` is larger than the current length
    /// of the data pool an attempt is made to prefill the pool so that it holds `k`
    /// elements.
    pub fn reset(&mut self, k: usize) {
        self.first = true;

        if k < self.indices.len() {
            self.indices.truncate(k);
            for i in 0..k {
                self.indices[i] = i;
            }
        } else {
            for i in 0..self.indices.len() {
                self.indices[i] = i;
            }
            self.indices.extend(self.indices.len()..k);
            self.pool.prefill(k);
        }
    }
}

impl<I> Iterator for Combinations<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            if self.k() > self.n() {
                return None;
            }
            self.first = false;
        } else if self.indices.is_empty() {
            return None;
        } else {
            // Scan from the end, looking for an index to increment
            let mut i: usize = self.indices.len() - 1;

            // Check if we need to consume more from the iterator
            if self.indices[i] == self.pool.len() - 1 {
                self.pool.get_next(); // may change pool size
            }

            while self.indices[i] == i + self.pool.len() - self.indices.len() {
                if i > 0 {
                    i -= 1;
                } else {
                    // Reached the last combination
                    return None;
                }
            }

            // Increment index, and reset the ones to its right
            self.indices[i] += 1;
            for j in i + 1..self.indices.len() {
                self.indices[j] = self.indices[j - 1] + 1;
            }
        }

        // Create result vector based on the indices
        Some(self.indices.iter().map(|i| self.pool[*i].clone()).collect())
    }
}

impl<I> FusedIterator for Combinations<I>
where
    I: Iterator,
    I::Item: Clone,
{
}
