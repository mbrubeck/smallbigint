// Copyright 2017 Matt Brubeck. See the COPYRIGHT file at the top-level
// directory of this distribution and at http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(test)]
mod tests;

mod ops;

use std::{
    fmt,
    mem::{forget, replace},
    slice,
};

/// An arbitrary-precision integer that is 64 bits wide, and can store data either
/// inline or on the heap.  Optimized for the case where the value fits in less
/// than 64 bits.
#[derive(Default, Eq)]
pub struct BigUint {
    data: u64,
}

/// If the lowest bit of `data` is set, then the remaining bits of `data`
/// are a pointer to a heap allocation.
const HEAP_FLAG: u64 = 1;

/// The largest value that can be stored inline.
const INLINE_MAX: u64 = !0 >> 1;

impl BigUint {
    /// Creates a `BigUint` from the bits in the provided slice.
    ///
    /// The order of the slice is from least significant 64-bit word to most-significant.  For
    /// example, 2^128 is represented as `[0, 0, 1]`.
    pub fn from_slice(v: &[u64]) -> Self {
        let mut result = Self::with_capacity(v.len());
        result.heap_value_mut().unwrap().copy_from_slice(v);
        result
    }

    /// Create a `BigUint` with `n` words of storage pre-allocated on the heap.
    fn with_capacity(n: usize) -> Self {
        let mut vec = vec![0; n + 1];
        vec[0] = vec.capacity() as u64; // Capacity is stored in the first element.
        Self::from_storage(vec)
    }

    /// If the rightmost bit is set, then we treat it as inline storage.
    fn is_inline(&self) -> bool {
        self.data & HEAP_FLAG == 0
    }

    /// Otherwise, `data` is a pointer to a heap allocation.
    fn is_heap(&self) -> bool {
        !self.is_inline()
    }

    /// Raw pointer to the heap allocation.
    fn heap_ptr(&self) -> Option<*mut u64> {
        if self.is_heap() {
            Some((self.data & !HEAP_FLAG) as *mut u64)
        } else {
            None
        }
    }

    /// The entire heap buffer, including the length header and the value.
    fn heap_storage(&self) -> Option<&[u64]> {
        let ptr = self.heap_ptr()?;
        unsafe {
            let cap = *ptr as usize;
            Some(slice::from_raw_parts(ptr, cap))
        }
    }

    /// The entire heap buffer, including the length header and the value.
    fn heap_storage_mut(&mut self) -> Option<&mut [u64]> {
        let ptr = self.heap_ptr()?;
        unsafe {
            let cap = *ptr as usize;
            Some(slice::from_raw_parts_mut(ptr, cap))
        }
    }

    /// Just the value portion of the heap buffer.
    fn heap_value(&self) -> Option<&[u64]> {
        Some(&self.heap_storage()?[1..])
    }

    /// Just the value portion of the heap buffer.
    fn heap_value_mut(&mut self) -> Option<&mut [u64]> {
        Some(&mut self.heap_storage_mut()?[1..])
    }

    fn inline_val(&self) -> Option<u64> {
        if self.is_inline() {
            Some(self.data >> 1)
        } else {
            None
        }
    }

    /// Convert a heap-allocated `BigUint` into a `Vec` containing its internal storage format.
    fn into_storage(self) -> Option<Vec<u64>> {
        let heap_ptr = self.heap_ptr();
        forget(self);
        unsafe {
            let ptr = heap_ptr?;
            let cap = *ptr as usize;
            Some(Vec::from_raw_parts(ptr, cap, cap))
        }
    }

    /// Construct a `BigUint` from a `Vec` containing its internal storage format.
    fn from_storage(vec: Vec<u64>) -> Self {
        let data = vec.as_ptr() as u64 | HEAP_FLAG;
        forget(vec);
        Self { data }
    }
}

impl Drop for BigUint {
    fn drop(&mut self) {
        replace(self, Self::default()).into_storage();
    }
}

impl Clone for BigUint {
    fn clone(&self) -> Self {
        if let Some(storage) = self.heap_storage() {
            Self::from_storage(storage.to_vec())
        } else {
            Self { data: self.data }
        }
    }
}

impl fmt::Display for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(n) = self.inline_val() {
            write!(f, "{}", n)
        } else {
            unimplemented!()
        }
    }
}

impl fmt::Debug for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<u64> for BigUint {
    fn from(n: u64) -> Self {
        if n <= INLINE_MAX {
            Self { data: n << 1 }
        } else {
            let mut x = Self::with_capacity(1);
            x.heap_value_mut().unwrap()[0] = n;
            x
        }
    }
}

impl PartialEq for BigUint {
    fn eq(&self, other: &Self) -> bool {
        match (self.inline_val(), other.inline_val()) {
            (Some(a), Some(b)) => a == b,
            (Some(a), None) => eq(&[a], other.heap_value().unwrap()),
            (None, Some(b)) => eq(self.heap_value().unwrap(), &[b]),
            (None, None) => eq(self.heap_value().unwrap(), other.heap_value().unwrap()),
        }
    }
}

fn eq(a: &[u64], b: &[u64]) -> bool {
    if a.len() == b.len() {
        a == b
    } else {
        strip_trailing_zeros(a) == strip_trailing_zeros(b)
    }
}

fn strip_trailing_zeros(v: &[u64]) -> &[u64] {
    if let Some(i) = v.iter().rposition(|x| *x != 0) {
        &v[..i + 1]
    } else {
        &[]
    }
}
