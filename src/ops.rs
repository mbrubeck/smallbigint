// Copyright 2017 Matt Brubeck. See the COPYRIGHT file at the top-level
// directory of this distribution and at http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::{Add, Mul};
use super::BigUint;

fn mul_with_carry(a: u64, b: u64) -> (u64, u64) {
    let c = a as u128 * b as u128;
    (c as u64, (c >> 64) as u64)
}

impl Add for BigUint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self.inline_val(), other.inline_val()) {
            (Some(a), Some(b)) => {
                // Can't overflow because INLINE_MAX < u64::MAX / 2.
                Self::from(a + b)
            }
            _ => unimplemented!()
        }
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
