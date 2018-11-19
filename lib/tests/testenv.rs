// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Testing common components of the test environment

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use gong::options::*;
use common::get_base;

/// Checks defaults in test environment match those within lib (the constants are not part of the
/// public API).
#[test]
fn check_constants() {
    let test = gong_option_set!(vec![], vec![], common::MODE_DEFAULT, common::ABBR_SUP_DEFAULT);
    let cmp: Options = Default::default();
    assert_eq!(test, cmp);
}

/// Check common base "available options" set validity
///
/// Doing it once here allows avoiding inefficiently doing so in every test using it (without
/// modification).
mod base_set {
    use super::*;

    #[test]
    fn is_valid() {
        let opts = get_base();
        assert!(opts.is_valid());
    }

    /// Double check inserting a problem does break it
    #[test]
    #[should_panic]
    fn can_break() {
        let mut opts = get_base();
        opts.add_long("foo"); // Duplicate - should panic here in debug mode!
        assert!(opts.is_valid());
    }

    /// Check valid in `alt` mode
    #[test]
    fn is_valid_altmode() {
        let mut opts = get_base();
        opts.set_mode(OptionsMode::Alternate);
        assert!(opts.is_valid());
    }
}
