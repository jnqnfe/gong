// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Testing “available” option set construction/modification
//!
//! Note, construction with macros is tested separately

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use gong::options::*;

/// Check basic valid construction methods
#[test]
fn basic() {
    let mut opts = OptionSetEx::new();
    opts.add_short('h')
        .add_short_data('o')
        .add_existing_short(gong_shortopt!('a', false))
        .add_long("foo")
        .add_long_data("bar")
        .add_existing_long(gong_longopt!("foobar", false));

    let expected = OptionSetEx {
        long: vec![
            gong_longopt!("foo", false),
            gong_longopt!("bar", true),
            gong_longopt!("foobar", false),
        ],
        short: vec![
            gong_shortopt!('h', false),
            gong_shortopt!('o', true),
            gong_shortopt!('a', false),
        ],
    };

    assert_eq!(opts, expected);
    assert!(opts.validate().is_ok());
}

/// Check `is_empty`
#[test]
fn is_empty() {
    // Here, let's double-check that the derive of `Default` for fixed option sets is really an
    // empty set
    let opt_set = OptionSet::default();
    assert!(opt_set.is_empty());

    let opt_set = gong_option_set_fixed!();
    assert!(opt_set.is_empty());
    let opt_set = gong_option_set_fixed!([], []);
    assert!(opt_set.is_empty());

    let opt_set = gong_option_set_fixed!([ gong_longopt!("foo", false) ], []);
    assert!(!opt_set.is_empty());

    let opt_set = gong_option_set_fixed!([], [ gong_shortopt!('h', false) ]);
    assert!(!opt_set.is_empty());

    let opt_set = gong_option_set_fixed!(
        [ gong_longopt!("foo", false) ],
        [ gong_shortopt!('h', false) ]
    );
    assert!(!opt_set.is_empty());
}

/// Check set type (`OptionSet`/`OptionSetEx`) conversion and comparison
#[test]
fn set_types() {
    let opts_fixed = OptionSet {
        long: &[
            gong_longopt!("foo", false),
            gong_longopt!("bar", true),
            gong_longopt!("foobar", false),
        ],
        short: &[
            gong_shortopt!('h', false),
            gong_shortopt!('o', true),
            gong_shortopt!('a', false),
        ],
    };

    let opts_extendible = OptionSetEx {
        long: vec![
            gong_longopt!("foo", false),
            gong_longopt!("bar", true),
            gong_longopt!("foobar", false),
        ],
        short: vec![
            gong_shortopt!('h', false),
            gong_shortopt!('o', true),
            gong_shortopt!('a', false),
        ],
    };

    // Check the two types can be compared
    assert_eq!(true, opts_fixed.eq(&opts_extendible));
    assert_eq!(true, opts_extendible.eq(&opts_fixed));

    // Check conversions
    let fixed_from_extendible: OptionSet = opts_extendible.as_fixed();
    assert_eq!(true, opts_fixed.eq(&fixed_from_extendible));
    assert_eq!(true, opts_extendible.eq(&fixed_from_extendible));
    assert_eq!(true, fixed_from_extendible.eq(&opts_fixed));
    assert_eq!(true, fixed_from_extendible.eq(&opts_extendible));
    let extendible_from_fixed: OptionSetEx = opts_fixed.to_extendible();
    assert_eq!(true, opts_fixed.eq(&extendible_from_fixed));
    assert_eq!(true, opts_extendible.eq(&extendible_from_fixed));
    assert_eq!(true, extendible_from_fixed.eq(&opts_fixed));
    assert_eq!(true, extendible_from_fixed.eq(&opts_extendible));

    let opts_fixed_2 = OptionSet {
        long: &[
            gong_longopt!("blah", false),
        ],
        short: &[],
    };

    let opts_extendible_2 = OptionSetEx {
        long: vec![
            gong_longopt!("blah", false),
        ],
        short: vec![],
    };

    // Verify not equal
    assert!(opts_fixed != opts_fixed_2);
    assert!(opts_fixed != opts_extendible_2);
    assert!(opts_extendible != opts_fixed_2);
    assert!(opts_extendible != opts_extendible_2);
}
