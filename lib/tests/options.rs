// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Testing "available" option set construction/modification
//!
//! Note, construction with macros is tested separately

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use gong::analysis::*;
use gong::options::*;
use common::{Actual, Expected, check_result};

/* Dash ('-') is an invalid short option (clashes with early terminator if it were given on its own
 * (`--`), and would be misinterpreted as a long option if given as the first in a short option set
 * (`--abc`)).
 */

/// Check `add_short` rejects '-'
#[test]
#[cfg_attr(debug_assertions, should_panic)]
fn add_short_dash() {
    let mut opts = Options::new(0, 1);
    opts.add_short('-'); // Should panic here in debug mode!
}

/// Check `add_short_data` rejects '-'
#[test]
#[cfg_attr(debug_assertions, should_panic)]
fn add_short_data_dash() {
    let mut opts = Options::new(0, 1);
    opts.add_short_data('-'); // Should panic here in debug mode!
}

/// Bypassing add methods, check `is_valid` rejects dash ('-') as short option
#[test]
#[should_panic]
fn is_valid_short_dash() {
    let opts = gong_option_set!(
        vec![],
        vec![
            gong_shortopt!('a'),
            gong_shortopt!('-'),
            gong_shortopt!('b'),
        ]
    );
    assert!(opts.is_valid());
}

/// Check behaviour when validity check bypassed.
///
/// This situation is an invalid use case, the user should always validate their option set; this
/// test verifies that things behave though as we expect if the set is invalid due to a short that
/// is a dash ('-').
///
/// The expected behaviour is this: If the first char in an argument is a dash, then as long as the
/// second char is not also a dash, then it will succeed in matching as a short option. If an
/// attempt is made to use a dash in a short-opt set as the first one in the set, thus the argument
/// starts with two dashes, it will then be taken to be either a long option or early terminator, as
/// approriate, giving no consideration to the possibility of it being a short option.
#[test]
fn short_dash_bypass() {
    let args = arg_list!(
        "--abc",    // Can't use as a shortopt like this, will be interpretted as long opt
        "-a-bc",    // Can use like this
        "--",       // Can't use as a shortopt like this, will be interpretted as early terminator
    );
    let expected = expected!(
        error: false,
        warn: true,
        vec![
            expected_item!(0, UnknownLong, "abc"),
            expected_item!(1, UnknownShort, 'a'),
            expected_item!(1, Short, '-'),
            expected_item!(1, UnknownShort, 'b'),
            expected_item!(1, UnknownShort, 'c'),
            expected_item!(2, EarlyTerminator),
        ]
    );

    // Using a custom **invalid** option set (short is '-')
    let opts = gong_option_set!(vec![], vec![ gong_shortopt!('-') ]);
    //assert!(opts.is_valid()); DISABLED! WHAT HAPPENS NEXT? LET'S SEE...

    check_result(&Actual(gong::process(&args, &opts)), &expected);
}

/// Check `add_long` rejects empty string
#[test]
#[cfg_attr(debug_assertions, should_panic)]
fn add_long_no_name() {
    let mut opts = Options::new(1, 0);
    opts.add_long(""); // Should panic here in debug mode!
}

/// Check `add_long_data` rejects empty string
#[test]
#[cfg_attr(debug_assertions, should_panic)]
fn add_long_data_no_name() {
    let mut opts = Options::new(1, 0);
    opts.add_long_data(""); // Should panic here in debug mode!
}

/// Bypassing add methods, check `is_valid` rejects empty name long option
#[test]
#[should_panic]
fn is_valid_long_no_name() {
    let opts = gong_option_set!(
        vec![
            gong_longopt!("foo"),
            gong_longopt!(""),
            gong_longopt!("bar"),
        ],
        vec![]
    );
    assert!(opts.is_valid());
}

/* Long option names cannot contain an '=' (used for declaring a data sub-argument in the same
 * argument; if names could contain an '=', as data can, we would not know where to do the split,
 * complicating matching.
 */

/// Check `add_long` rejects equals ('=') char
#[test]
#[cfg_attr(debug_assertions, should_panic)]
fn add_long_with_equals() {
    let mut opts = Options::new(1, 0);
    opts.add_long("a=b"); // Should panic here in debug mode!
}

/// Check `add_long_data` rejects equals ('=') char
#[test]
#[cfg_attr(debug_assertions, should_panic)]
fn add_long_data_with_equals() {
    let mut opts = Options::new(1, 0);
    opts.add_long_data("a=b"); // Should panic here in debug mode!
}

/// Bypassing add methods, check `is_valid` rejects long option with equals ('=')
#[test]
#[should_panic]
fn is_valid_long_with_equals() {
    let opts = gong_option_set!(
        vec![
            gong_longopt!("foo"),
            gong_longopt!("a=b"),
            gong_longopt!("bar"),
        ],
        vec![]
    );
    assert!(opts.is_valid());
}

/// Check behaviour when validity check bypassed.
///
/// This situation is an invalid use case, the user should always validate their option set; this
/// test verifies that things behave though as we expect if the set is invalid due to a long with an
/// equals ('=').
#[test]
fn long_with_equals_bypass() {
    let args = arg_list!(
        "--a",      // This should match against the "a=b" invalid option as an abbreviation
        "--a=b",    // Here, this is a long option with "in-arg" data, thus the name is "a", which
                    // again therefore matched the invalid "a=b" option, as an abbreviation, but
                    // carrying "b" as data.
    );
    let expected = expected!(
        error: false,
        warn: true,
        vec![
            expected_item!(0, Long, "a=b"),
            expected_item!(1, LongWithUnexpectedData, "a=b", "b"),
        ]
    );

    // Using a custom **invalid** option set (long name contains '=')
    let opts = gong_option_set!(vec![ gong_longopt!("a=b") ], vec![]);
    //assert!(opts.is_valid()); DISABLED! WHAT HAPPENS NEXT? LET'S SEE...

    check_result(&Actual(gong::process(&args, &opts)), &expected);
}

/* Option sets should not contain duplicates.
 *
 * Duplicates pose a potential problem due to potential for confusion over differing `expects_data`
 * attributes. They also can result from option name-clashing bugs with programs that dynamically
 * generate (large) option sets (rare? VLC media player is one example, which dynamically builds an
 * option set including options from plugins). An option set containing duplicates is thus
 * considered invalid.
 */

/// Short option duplicates
#[test]
#[should_panic]
fn short_dups() {
    let mut opts = Options::new(0, 8);
    opts.add_short('a')
        .add_short('b')
        .add_short('c')
        .add_short('c')       // dup
        .add_short_data('d')
        .add_short_data('b')  // dup (ignore data indicator)
        .add_short('e')
        .add_short('b');      // dup
    assert!(opts.is_valid());
}

/// Long option duplicates
#[test]
#[should_panic]
fn long_dups() {
    let mut opts = Options::new(3, 0);
    opts.add_long("aaa")
        .add_long("bbb")
        .add_long("ccc")
        .add_long("ccc")      // dup
        .add_long_data("ddd")
        .add_long_data("bbb") // dup (ignore data indicator)
        .add_long("eee")
        .add_long("bbb");     // dup
    assert!(opts.is_valid());
}
