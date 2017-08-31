// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A structure for holding a set of enum variants.
//!
//! This module defines a container which uses an efficient bit mask
//! representation to hold C-like enum variants.

use std::fmt;
use std::hash;
use std::marker::PhantomData;
use std::iter;
use std::ops;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// A specialized set implementation to use enum types.
pub struct EnumSet<E> {
    // We must maintain the invariant that no bits are set
    // for which no variant exists
    bits: u32,
    phantom: PhantomData<E>,
}

impl<E: CLike + fmt::Debug> fmt::Debug for EnumSet<E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self).finish()
    }
}

impl<E: CLike> hash::Hash for EnumSet<E> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.bits.hash(state);
    }
}

/// An interface for casting C-like enum to `u32` and back.
///
/// The returned value must be no more than 31: `EnumSet` does not support more cases than this.
///
/// A typical implementation can be seen below:
///
/// ```
/// extern crate enum_set;
/// #[macro_use]
/// extern crate enum_set_derive;
///
/// #[derive(Clone, Copy, CLike)]
/// enum Foo {
///     A, B, C
/// }
/// # fn main() {}
/// ```
pub trait CLike {
    /// Converts a C-like enum to a `u32`. The value must be `<= 31`.
    fn to_u32(&self) -> u32;

    /// Converts a `u32` to a C-like enum. This method only needs to be safe
    /// for possible return values of `to_u32` of this trait.
    unsafe fn from_u32(u32) -> Self;
}

fn bit<E: CLike>(e: &E) -> u32 {
    let value = e.to_u32();
    assert!(value < 32, "EnumSet only supports up to {} variants.", 31);
    1 << value
}

impl<E: CLike> EnumSet<E> {
    /// Returns an empty `EnumSet`.
    pub fn new() -> Self {
        Self::new_with_bits(0)
    }

    fn new_with_bits(bits: u32) -> Self {
        EnumSet { bits: bits, phantom: PhantomData }
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }

    /// Checks if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Removes all elements from the set.
    pub fn clear(&mut self) {
        self.bits = 0;
    }

    /// Returns `true` if the set has no elements in common with `other`.
    ///
    /// This is equivalent to checking for an empty intersection.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        (self.bits & other.bits) == 0
    }

    /// Returns `true` if the set is a superset of `other`.
    pub fn is_superset(&self, other: &Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Returns `true` if the set is a subset of `other`.
    pub fn is_subset(&self, other: &Self) -> bool {
        other.is_superset(self)
    }

    /// Returns the union of the set and `other`.
    pub fn union(&self, other: Self) -> Self {
        Self::new_with_bits(self.bits | other.bits)
    }

    /// Returns the intersection of the set and `other`.
    pub fn intersection(&self, other: Self) -> Self {
        Self::new_with_bits(self.bits & other.bits)
    }

    /// Returns the difference between the set and `other`.
    pub fn difference(&self, other: Self) -> Self {
        Self::new_with_bits(self.bits & !other.bits)
    }

    /// Returns the symmetric difference between the set and `other`.
    pub fn symmetric_difference(&self, other: Self) -> Self {
        Self::new_with_bits(self.bits ^ other.bits)
    }

    /// Adds the given value to the set.
    ///
    /// Returns `true` if the value was not already present in the set.
    pub fn insert(&mut self, value: E) -> bool {
        let result = !self.contains(&value);
        self.bits |= bit(&value);
        result
    }

    /// Removes a value from the set.
    ///
    /// Returns `true` if the value was present in the set.
    pub fn remove(&mut self, value: &E) -> bool {
        let result = self.contains(value);
        self.bits &= !bit(value);
        result
    }

    /// Returns `true` if the set contains the given value.
    pub fn contains(&self, value: &E) -> bool {
        (self.bits & bit(value)) != 0
    }

    /// Returns an iterator over the set's elements.
    pub fn iter(&self) -> Iter<E> {
        Iter { index: 0, bits: self.bits, phantom: PhantomData }
    }
}

impl<E: CLike> ops::Sub for EnumSet<E> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.difference(other)
    }
}

impl<E: CLike> ops::BitOr for EnumSet<E> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        self.union(other)
    }
}

impl<E: CLike> ops::BitAnd for EnumSet<E> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        self.intersection(other)
    }
}

impl<E: CLike> ops::BitXor for EnumSet<E> {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        self.symmetric_difference(other)
    }
}

#[derive(Clone)]
/// An iterator over an `EnumSet`.
pub struct Iter<E> {
    index: u32,
    bits: u32,
    phantom: PhantomData<*mut E>,
}

impl<E: CLike> Iterator for Iter<E> {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        if self.bits == 0 {
            return None;
        }

        while (self.bits & 1) == 0 {
            self.index += 1;
            self.bits >>= 1;
        }

        // Safe because of the invariant that only valid bits are set (see
        // comment on the `bit` member of this struct).
        let elem = unsafe { CLike::from_u32(self.index) };
        self.index += 1;
        self.bits >>= 1;
        Some(elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.bits.count_ones() as usize;
        (exact, Some(exact))
    }
}

impl<E: CLike> ExactSizeIterator for Iter<E> {}

impl<E: CLike> Default for EnumSet<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: CLike> iter::FromIterator<E> for EnumSet<E> {
    fn from_iter<I: IntoIterator<Item = E>>(iterator: I) -> Self {
        let mut ret = Self::new();
        ret.extend(iterator);
        ret
    }
}

impl<E: CLike> Extend<E> for EnumSet<E> {
    fn extend<I: IntoIterator<Item = E>>(&mut self, iter: I) {
        for element in iter {
            self.insert(element);
        }
    }
}

impl<'a, E: CLike> IntoIterator for &'a EnumSet<E> {
    type Item = E;
    type IntoIter = Iter<E>;
    fn into_iter(self) -> Iter<E> { self.iter() }
}
