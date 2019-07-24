// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Testing “available” option construction

extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use std::ffi::OsStr;
use gong::{option_set, longopt, shortopt};
use gong::analysis::*;
use gong::options::*;
use gong::parser::Parser;
use self::common::{Actual, Expected};

/// Dash (`-`) is an invalid short option (clashes with early terminator if it were given on its own
/// (`--`), and would be misinterpreted as a long option if given as the first in a short option set
/// (`--abc`)).
mod short_dash {
    use super::*;

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_short() {
        let mut opts = OptionSetEx::new();
        opts.add_short('-', OptionType::Flag); // Should panic here in debug mode!
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_short_data() {
        let mut opts = OptionSetEx::new();
        opts.add_short('-', OptionType::Data); // Should panic here in debug mode!
    }

    #[test]
    fn add_short_existing() {
        let mut opts = OptionSetEx::new();
        opts.add_existing_short(shortopt!(@flag '-')); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    fn invalid_set() {
        let opts = option_set!(@short [
            shortopt!(@flag 'a'),
            shortopt!(@flag '-'),
            shortopt!(@flag 'b'),
        ]);
        assert_eq!(false, opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![ OptionFlaw::ShortIsForbiddenChar('-') ]));
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
    fn bypassed_parsing() {
        // Using a custom **invalid** option set (short is '-')
        let opts = option_set!(@short [ shortopt!(@flag '-') ]);
        //assert!(opts.validate().is_ok()); DISABLED! WHAT HAPPENS NEXT? LET’S SEE...

        let args = arg_list!(
            "--abc",    // Can’t use as a shortopt like this, will be interpreted as long opt
            "-a-bc",    // Can use like this
            "--",       // Can’t use as a shortopt like this, will be interpreted as early terminator
        );
        let expected = expected!(
            problems: true,
            opt_set: &opts,
            [
                expected_item!(0, UnknownLong, "abc"),
                expected_item!(1, UnknownShort, 'a'),
                expected_item!(1, Short, '-'),
                expected_item!(1, UnknownShort, 'b'),
                expected_item!(1, UnknownShort, 'c'),
                expected_item!(2, EarlyTerminator),
            ]
        );

        let mut parser = Parser::new(&opts, None);
        parser.settings.set_stop_on_problem(false);
        check_result!(&Actual(parser.parse(&args)), &expected);
    }
}

/// The unicode replacement character (`\u{FFFD}`) is an invalid short option. If it were valid, it
/// would allow incorrect analysis with `OsStr` based parsing.
mod short_rep_char {
    use std::char::REPLACEMENT_CHARACTER;
    use super::*;

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_short() {
        let mut opts = OptionSetEx::new();
        opts.add_short(REPLACEMENT_CHARACTER, OptionType::Flag); // Should panic here in debug mode!
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_short_data() {
        let mut opts = OptionSetEx::new();
        opts.add_short(REPLACEMENT_CHARACTER, OptionType::Data); // Should panic here in debug mode!
    }

    #[test]
    fn add_short_existing() {
        let mut opts = OptionSetEx::new();
        opts.add_existing_short(shortopt!(@flag REPLACEMENT_CHARACTER)); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    fn invalid_set() {
        let opts = option_set!(@short [
            shortopt!(@flag 'a'),
            shortopt!(@flag REPLACEMENT_CHARACTER),
            shortopt!(@flag 'b'),
        ]);
        assert_eq!(false, opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![
            OptionFlaw::ShortIsForbiddenChar(REPLACEMENT_CHARACTER)
        ]));
    }
}

/// An empty string is not a valid long-option name property
mod long_no_name {
    use super::*;

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long() {
        let mut opts = OptionSetEx::new();
        opts.add_long("", OptionType::Flag); // Should panic here in debug mode!
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long_data() {
        let mut opts = OptionSetEx::new();
        opts.add_long("", OptionType::Data); // Should panic here in debug mode!
    }

    #[test]
    fn add_long_existing() {
        let mut opts = OptionSetEx::new();
        opts.add_existing_long(longopt!(@flag "")); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    fn invalid_set() {
        let opts = option_set!(@long [
            longopt!(@flag "foo"),
            longopt!(@flag ""),
            longopt!(@flag "bar"),
        ]);
        assert_eq!(false, opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![ OptionFlaw::LongEmptyName ]));
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
        let mut opts = OptionSetEx::new();
        opts.add_long("a=b", OptionType::Flag); // Should panic here in debug mode!
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long_data() {
        let mut opts = OptionSetEx::new();
        opts.add_long("a=b", OptionType::Data); // Should panic here in debug mode!
    }

    #[test]
    fn add_long_existing() {
        let mut opts = OptionSetEx::new();
        opts.add_existing_long(longopt!(@flag "=")); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    fn invalid_set() {
        let opts = option_set!(@long [
            longopt!(@flag "foo"),
            longopt!(@flag "a=b"),
            longopt!(@flag "bar"),
        ]);
        assert_eq!(false, opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![ OptionFlaw::LongNameHasForbiddenChar("a=b", '=') ]));
    }

    /// Check behaviour when validity check bypassed.
    ///
    /// This situation is an invalid use case, the user should always validate their option set;
    /// this test verifies that things behave though as we expect if the set is invalid due to a
    /// long with an equals (`=`).
    #[test]
    fn bypassed_parsing() {
        // Using a custom **invalid** option set (long name contains `=`)
        let opts = option_set!(@long [ longopt!(@flag "a=b") ]);
        //assert!(opts.validate().is_ok()); DISABLED! WHAT HAPPENS NEXT? LET’S SEE...

        let args = arg_list!(
            "--a",      // This should match against the “a=b” invalid option as an abbreviation
            "--a=b",    // Here, this is a long option with “in-arg” data, thus the name is “a”,
                        // which again therefore matches the invalid “a=b” option, as an
                        // abbreviation, but carrying “b” as data.
        );
        let expected = expected!(
            problems: true,
            opt_set: &opts,
            [
                expected_item!(0, Long, "a=b"),
                expected_item!(1, LongWithUnexpectedData, "a=b", "b"),
            ]
        );

        let mut parser = Parser::new(&opts, None);
        parser.settings.set_stop_on_problem(false);
        check_result!(&Actual(parser.parse(&args)), &expected);
    }
}

/// Long option names cannot contain the unicode replacement character (`\u{FFFD}`). If it were
/// allowed, it would allow incorrect analysis with `OsStr` based parsing.
mod long_rep_char {
    use std::char::REPLACEMENT_CHARACTER;
    use super::*;

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long() {
        let mut opts = OptionSetEx::new();
        opts.add_long("a\u{FFFD}b", OptionType::Flag); // Should panic here in debug mode!
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long_data() {
        let mut opts = OptionSetEx::new();
        opts.add_long("a\u{FFFD}b", OptionType::Data); // Should panic here in debug mode!
    }

    #[test]
    fn add_long_existing() {
        let mut opts = OptionSetEx::new();
        opts.add_existing_long(longopt!(@flag "\u{FFFD}")); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    fn invalid_set() {
        let opts = option_set!(@long [
            longopt!(@flag "foo"),
            longopt!(@flag "a\u{FFFD}b"),
            longopt!(@flag "bar"),
        ]);
        assert_eq!(false, opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![
            OptionFlaw::LongNameHasForbiddenChar("a\u{FFFD}b", REPLACEMENT_CHARACTER)
        ]));
    }
}

/// Check what happens with multiple flaws at a time. Naturally this does not apply to short options.
mod multi {
    use std::char::REPLACEMENT_CHARACTER;
    use super::*;

    /// Bypassing add methods, check validation fails
    #[test]
    fn invalid_set() {
        let opts = option_set!(@long [
            longopt!(@flag "foo"),
            longopt!(@flag "a\u{FFFD}b=c=d"), // More than one unique flaw, and duplicate flaws
            longopt!(@flag "w=x=y\u{FFFD}z"), // Same
            longopt!(@flag "foo\u{FFFD}bar"), // Single flaw, without the equals flaw
            longopt!(@flag "bar"),
        ]);
        assert_eq!(false, opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![
            /// Only the first flaw identified of each option is returned
            OptionFlaw::LongNameHasForbiddenChar("a\u{FFFD}b=c=d", '='),
            OptionFlaw::LongNameHasForbiddenChar("w=x=y\u{FFFD}z", '='),
            OptionFlaw::LongNameHasForbiddenChar("foo\u{FFFD}bar", REPLACEMENT_CHARACTER),
        ]));
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
    fn short() {
        let mut opts = OptionSetEx::with_capacity(0, 8);
        opts.add_short('a', OptionType::Flag)
            .add_short('b', OptionType::Flag)
            .add_short('c', OptionType::Flag)
            .add_short('c', OptionType::Flag)  // dup
            .add_short('d', OptionType::Data)
            .add_short('b', OptionType::Data)  // dup (ignore data indicator)
            .add_short('e', OptionType::Flag)
            .add_short('b', OptionType::Flag); // dup
        assert_eq!(false, opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![
            OptionFlaw::ShortDuplicated('b'),
            OptionFlaw::ShortDuplicated('c'),
        ]));
    }

    #[test]
    fn long() {
        let mut opts = OptionSetEx::with_capacity(8, 0);
        opts.add_long("aaa", OptionType::Flag)
            .add_long("bbb", OptionType::Flag)
            .add_long("ccc", OptionType::Flag)
            .add_long("ccc", OptionType::Flag)  // dup
            .add_long("ddd", OptionType::Data)
            .add_long("bbb", OptionType::Data)  // dup (ignore data indicator)
            .add_long("eee", OptionType::Flag)
            .add_long("bbb", OptionType::Flag); // dup
        assert_eq!(false, opts.is_valid());
        assert_eq!(opts.validate(), Err(vec![
            OptionFlaw::LongDuplicated("bbb"),
            OptionFlaw::LongDuplicated("ccc"),
        ]));
    }
}
