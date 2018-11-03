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

use gong::analysis::*;
use gong::options::*;
use common::{Actual, Expected, check_result, MODE_DEFAULT, ABBR_SUP_DEFAULT};

/// Check basic valid construction methods
#[test]
fn basic() {
    let mut opts = OptionSetEx::new(3, 3);
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
        mode: MODE_DEFAULT,
        allow_abbreviations: ABBR_SUP_DEFAULT,
    };

    assert_eq!(opts, expected);
    assert!(opts.validate().is_ok());
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
        mode: MODE_DEFAULT,
        allow_abbreviations: ABBR_SUP_DEFAULT,
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
        mode: MODE_DEFAULT,
        allow_abbreviations: ABBR_SUP_DEFAULT,
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
        mode: MODE_DEFAULT,
        allow_abbreviations: ABBR_SUP_DEFAULT,
    };

    let opts_extendible_2 = OptionSetEx {
        long: vec![
            gong_longopt!("blah", false),
        ],
        short: vec![],
        mode: MODE_DEFAULT,
        allow_abbreviations: ABBR_SUP_DEFAULT,
    };

    // Verify not equal
    assert!(opts_fixed != opts_fixed_2);
    assert!(opts_fixed != opts_extendible_2);
    assert!(opts_extendible != opts_fixed_2);
    assert!(opts_extendible != opts_extendible_2);
}

/// Dash (`-`) is an invalid short option (clashes with early terminator if it were given on its own
/// (`--`), and would be misinterpreted as a long option if given as the first in a short option set
/// (`--abc`)).
mod short_dash {
    use super::*;

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_short() {
        let mut opts = OptionSetEx::new(0, 1);
        opts.add_short('-'); // Should panic here in debug mode!
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_short_data() {
        let mut opts = OptionSetEx::new(0, 1);
        opts.add_short_data('-'); // Should panic here in debug mode!
    }

    #[test]
    fn add_short_existing() {
        let mut opts = OptionSetEx::new(0, 1);
        opts.add_existing_short(gong_shortopt!('-', false)); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    #[should_panic]
    fn invalid_set() {
        let opts = gong_option_set_fixed!(
            [], [
                gong_shortopt!('a'),
                gong_shortopt!('-'),
                gong_shortopt!('b'),
            ]
        );
        assert!(opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![ OptionFlaw::ShortDash ]));
    }

    /// Check behaviour when validity check bypassed.
    ///
    /// This situation is an invalid use case, the user should always validate their option set;
    /// this test verifies that things behave though as we expect if the set is invalid due to a
    /// short that is a dash (`-`).
    ///
    /// The expected behaviour is this: If the first char in an argument is a dash, then as long as
    /// the second char is not also a dash, then it will succeed in matching as a short option. If
    /// an attempt is made to use a dash in a short-opt set as the first one in the set, thus the
    /// argument starts with two dashes, it will then be taken to be either a long option or early
    /// terminator, as appropriate, giving no consideration to the possibility of it being a short
    /// option.
    #[test]
    fn bypassed_processing() {
        let args = arg_list!(
            "--abc",    // Can’t use as a shortopt like this, will be interpreted as long opt
            "-a-bc",    // Can use like this
            "--",       // Can’t use as a shortopt like this, will be interpreted as early terminator
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
        let opts = gong_option_set_fixed!([], [ gong_shortopt!('-') ]);
        //assert!(opts.validate().is_ok()); DISABLED! WHAT HAPPENS NEXT? LET’S SEE...

        check_result(&Actual(opts.process(&args)), &expected);
    }
}

/// An empty string is not a valid long-option name property
mod long_no_name {
    use super::*;

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long() {
        let mut opts = OptionSetEx::new(1, 0);
        opts.add_long(""); // Should panic here in debug mode!
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long_data() {
        let mut opts = OptionSetEx::new(1, 0);
        opts.add_long_data(""); // Should panic here in debug mode!
    }

    #[test]
    fn add_long_existing() {
        let mut opts = OptionSetEx::new(1, 0);
        opts.add_existing_long(gong_longopt!("", false)); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    #[should_panic]
    fn invalid_set() {
        let opts = gong_option_set_fixed!(
            [
                gong_longopt!("foo"),
                gong_longopt!(""),
                gong_longopt!("bar"),
            ], []
        );
        assert!(opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![ OptionFlaw::LongEmpty ]));
    }
}

/// Long option names cannot contain an `=` (used for declaring a data sub-argument in the same
/// argument; if names could contain an `=`, as data can, we would not know where to do the split,
/// complicating matching.
mod long_equals {
    use super::*;

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long() {
        let mut opts = OptionSetEx::new(1, 0);
        opts.add_long("a=b"); // Should panic here in debug mode!
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long_data() {
        let mut opts = OptionSetEx::new(1, 0);
        opts.add_long_data("a=b"); // Should panic here in debug mode!
    }

    #[test]
    fn add_long_existing() {
        let mut opts = OptionSetEx::new(1, 0);
        opts.add_existing_long(gong_longopt!("=", false)); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    #[should_panic]
    fn invalid_set() {
        let opts = gong_option_set_fixed!(
            [
                gong_longopt!("foo"),
                gong_longopt!("a=b"),
                gong_longopt!("bar"),
            ], []
        );
        assert!(opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![ OptionFlaw::LongIncludesEquals("a=b") ]));
    }

    /// Check behaviour when validity check bypassed.
    ///
    /// This situation is an invalid use case, the user should always validate their option set;
    /// this test verifies that things behave though as we expect if the set is invalid due to a
    /// long with an equals (`=`).
    #[test]
    fn bypassed_processing() {
        let args = arg_list!(
            "--a",      // This should match against the “a=b” invalid option as an abbreviation
            "--a=b",    // Here, this is a long option with “in-arg” data, thus the name is “a”,
                        // which again therefore matched the invalid “a=b” option, as an
                        // abbreviation, but carrying “b” as data.
        );
        let expected = expected!(
            error: false,
            warn: true,
            vec![
                expected_item!(0, Long, "a=b"),
                expected_item!(1, LongWithUnexpectedData, "a=b", "b"),
            ]
        );

        // Using a custom **invalid** option set (long name contains `=`)
        let opts = gong_option_set_fixed!([ gong_longopt!("a=b") ], []);
        //assert!(opts.validate().is_ok()); DISABLED! WHAT HAPPENS NEXT? LET’S SEE...

        check_result(&Actual(opts.process(&args)), &expected);
    }
}

/// Duplicates pose a potential problem due to potential for confusion over differing `expects_data`
/// attributes. They also can result from option name-clashing bugs with programs that dynamically
/// generate (large) option sets (rare? VLC media player is one example, which dynamically builds an
/// option set including options from plugins). An option set containing duplicates is thus
/// considered invalid.
mod duplicates {
    use super::*;

    #[test]
    #[should_panic]
    fn short() {
        let mut opts = OptionSetEx::new(0, 8);
        opts.add_short('a')
            .add_short('b')
            .add_short('c')
            .add_short('c')       // dup
            .add_short_data('d')
            .add_short_data('b')  // dup (ignore data indicator)
            .add_short('e')
            .add_short('b');      // dup
        assert!(opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![
            OptionFlaw::ShortDup('c'),
            OptionFlaw::ShortDup('b'),
            OptionFlaw::ShortDup('b'),
        ]));
    }

    #[test]
    #[should_panic]
    fn long() {
        let mut opts = OptionSetEx::new(8, 0);
        opts.add_long("aaa")
            .add_long("bbb")
            .add_long("ccc")
            .add_long("ccc")      // dup
            .add_long_data("ddd")
            .add_long_data("bbb") // dup (ignore data indicator)
            .add_long("eee")
            .add_long("bbb");     // dup
        assert!(opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![
            OptionFlaw::LongDup("ccc"),
            OptionFlaw::LongDup("bbb"),
            OptionFlaw::LongDup("bbb"),
        ]));
    }
}
