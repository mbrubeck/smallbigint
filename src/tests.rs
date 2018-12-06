// Copyright 2017 Matt Brubeck. See the COPYRIGHT file at the top-level
// directory of this distribution and at http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::*;

#[test]
fn display_inline() {
    assert_eq!(format!("{}", BigUint::from(0)), "0");
    assert_eq!(format!("{}", BigUint::from(1)), "1");
}

#[test]
fn eq_inline() {
    assert_eq!(BigUint::from(0), BigUint::from(0));
    assert_eq!(BigUint::from(1), BigUint::from(1));
    assert_ne!(BigUint::from(0), BigUint::from(1));
    assert_ne!(BigUint::from(1), BigUint::from(0));
}

#[test]
fn strip_trailing_zeros() {
    assert_eq!(super::strip_trailing_zeros(&[]), &[]);
    assert_eq!(super::strip_trailing_zeros(&[1]), &[1]);
    assert_eq!(super::strip_trailing_zeros(&[1, 2, 3]), &[1, 2, 3]);
    assert_eq!(super::strip_trailing_zeros(&[1, 0]), &[1]);
    assert_eq!(super::strip_trailing_zeros(&[1, 0, 0]), &[1]);
    assert_eq!(super::strip_trailing_zeros(&[1, 0, 0, 0]), &[1]);
    assert_eq!(super::strip_trailing_zeros(&[1, 2, 0, 0]), &[1, 2]);
    assert_eq!(super::strip_trailing_zeros(&[0, 2, 0, 0]), &[0, 2]);
    assert_eq!(super::strip_trailing_zeros(&[0, 0, 0]), &[]);
}
