// Copyright 2017 Matt Brubeck. See the COPYRIGHT file at the top-level
// directory of this distribution and at http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{
    cmp::max,
    mem::replace,
    ops::{Add, AddAssign, Mul},
};
use super::BigUint;

fn mul_with_carry(a: u64, b: u64) -> (u64, u64) {
    let c = a as u128 * b as u128;
    (c as u64, (c >> 64) as u64)
}

impl Add for BigUint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self.inline_val(), other.inline_val()) {
            (Some(a), Some(b)) => Self::from(a + b),
            (None, None) => {
                let mut a = self.into_storage().unwrap();
                let b = other.heap_value().unwrap();
                add_assign(&mut a, b);
                Self::from_storage(a)
            }
            (Some(a), None) => {
                let mut b = other.into_storage().unwrap();
                add_assign(&mut b, &[a]);
                Self::from_storage(b)
            }
            (None, Some(b)) => {
                let mut a = self.into_storage().unwrap();
                add_assign(&mut a, &[b]);
                Self::from_storage(a)
            }
        }
    }
}

impl AddAssign for BigUint {
    fn add_assign(&mut self, other: Self) {
        *self = replace(self, Self::default()) + other;
    }
}

fn add_assign(a: &mut Vec<u64>, b: &[u64]) {
    // Ensure there's space in `a` for all digits in `a` and `b` plus a carry digit.
    // TODO: Don't grow so eagerly?
    let cap = max(a.len(), b.len() + 1) + 1;
    a.resize(cap, 0);

    let mut carry = false;
    for (x, y) in a[1..].iter_mut().zip(b) {
        *x = x.wrapping_add(*y);
        if carry {
            *x = x.wrapping_add(1);
        }
        carry = *x < *y || (carry && *x == *y);
    }
    if carry {
        let x = &mut a[b.len() + 1];
        *x = x.wrapping_add(1);
    }
}

impl Mul for BigUint {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self.inline_val(), other.inline_val()) {
            (Some(a), Some(b)) => {
                let (sum, carry) = mul_with_carry(a, b);
                if carry == 0 {
                    Self::from(sum)
                } else {
                    let mut result = Self::with_capacity(2);
                    let buf = result.heap_value_mut().unwrap();
                    buf[0] = sum;
                    buf[1] = carry;
                    result
                }
            }
            _ => unimplemented!()
        }
    }
}
