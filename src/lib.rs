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

use std::{
    fmt,
    mem::forget,
    slice,
};

/// If the lowest bit of `data` is set, then the remaining bits of `data`
/// are a pointer to a heap allocation.
const HEAP_FLAG: usize = 1;

/// The largest value that can be stored inline.
const INLINE_MAX: usize = !0 >> 1;

#[derive(Default, Eq)]
pub struct BigUint {
    data: usize,
}

impl BigUint {
    /// Create a `BigUint` with `n` words of capacity pre-allocated on the heap.
    pub fn with_capacity(n: usize) -> Self {
        assert!(n < usize::max_value(), "capacity overflow");

        let mut v = vec![0; n + 1];
        v[0] = v.capacity(); // Capacity is stored in the first element.

        let data = v.as_ptr() as usize | HEAP_FLAG;
        forget(v);

        Self { data }
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
    fn heap_ptr(&self) -> Option<*mut usize> {
        if self.is_heap() {
            Some((self.data & !HEAP_FLAG) as *mut usize)
        } else {
            None
        }
    }

    /// The entire heap buffer, including the length header and the value.
    fn heap_storage(&self) -> Option<&[usize]> {
        let ptr = self.heap_ptr()?;
        unsafe {
            Some(slice::from_raw_parts(ptr, *ptr))
        }
    }

    /// Just the value portion of the heap buffer.
    fn heap_value(&self) -> Option<&[usize]> {
        let ptr = self.heap_ptr()?;
        unsafe {
            Some(slice::from_raw_parts(ptr.add(1), *ptr - 1))
        }
    }

    /// Just the value portion of the heap buffer.
    fn heap_value_mut(&mut self) -> Option<&mut [usize]> {
        let ptr = self.heap_ptr()?;
        unsafe {
            Some(slice::from_raw_parts_mut(ptr.add(1), *ptr - 1))
        }
    }

    fn from_inline_val(n: usize) -> Self {
        debug_assert!(n < INLINE_MAX);
        Self { data: n << 1 }
    }

    fn inline_val(&self) -> Option<usize> {
        if self.is_inline() {
            Some(self.data >> 1)
        } else {
            None
        }
    }
}

impl Drop for BigUint {
    fn drop(&mut self) {
        if let Some(ptr) = self.heap_ptr() {
            unsafe {
                drop(Vec::from_raw_parts(ptr, *ptr, *ptr));
            }
        }
    }
}

impl Clone for BigUint {
    fn clone(&self) -> Self {
        if let Some(storage) = self.heap_storage() {
            let v = storage.to_vec();
            let data = v.as_ptr() as usize | HEAP_FLAG;
            forget(v);

            Self { data }
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

impl From<usize> for BigUint {
    fn from(n: usize) -> Self {
        if n < INLINE_MAX {
            Self::from_inline_val(n)
        } else {
            let mut x = Self::with_capacity(1);
            x.heap_value_mut().unwrap()[1] = n;
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

fn eq(a: &[usize], b: &[usize]) -> bool {
    if a.len() == b.len() {
        a == b
    } else {
        strip_trailing_zeros(a) == strip_trailing_zeros(b)
    }
}

fn strip_trailing_zeros(v: &[usize]) -> &[usize] {
    if let Some(i) = v.iter().rposition(|x| *x != 0) {
        &v[..i + 1]
    } else {
        &[]
    }
}
