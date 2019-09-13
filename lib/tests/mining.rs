// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Testing data-mining

extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use std::ffi::OsStr;
use gong::{shortopt, longopt, findopt, foundopt, optpair};
use gong::analysis::*;
use gong::positionals::Policy as PositionalsPolicy;
use self::common::{get_parser, Actual, Expected};

/// Some of the tests here expect certain options to exist in the common options set, where such
/// options are **not** being used in the test arguments, so we need to assert that they definitely
/// are there.
#[test]
fn env() {
    let parser = get_parser();
    assert!(parser.options.long.contains(&longopt!(@flag "foo")));
    assert!(parser.options.short.contains(&shortopt!(@flag 'x')));
}

mod findopt {
    use super::*;

    /// Test various forms of specifying search parameters
    #[test]
    fn search_params() {
        assert_eq!(findopt!(@long "help"), FindOption::Long("help"));
        assert_eq!(findopt!(@short 'h'), FindOption::Short('h'));
        assert_eq!(findopt!(@pair 'h', "help"), FindOption::Pair('h', "help"));

        // Conversion from option descriptors
        assert_eq!(FindOption::from(longopt!(@flag "help")), findopt!(@long "help"));
        assert_eq!(FindOption::from(shortopt!(@flag 'h')), findopt!(@short 'h'));
    }

    /// Test invalid forms of specifying search parameters
    #[test]
    #[cfg(compile_fail)]
    fn search_params_fail() {
        let _ = findopt!(@pair "help", 'h'); // Wrong order
    }

    #[test]
    fn from_optpair() {
        assert_eq!(findopt!(@pair 'h', "help"), optpair!(@flag 'h', "help").as_findopt());
    }
}

mod foundopt {
    use super::*;

    /// Test various forms of search results
    #[test]
    fn search_results() {
        // Single
        assert_eq!(foundopt!(@long "help"), FoundOption::Long("help"));
        assert_eq!(foundopt!(@short 'h'), FoundOption::Short('h'));
    }

    /// Test invalid forms of search results
    #[test]
    #[cfg(compile_fail)]
    fn search_results_fail() {
        let _ = foundopt!(@pair 'h', "help"); // `FoundOption` does not capture a pair
    }

    /// Test creation from options
    #[test]
    fn opt_conv() {
        assert_eq!(foundopt!(@long "help"), FoundOption::from(longopt!(@flag "help")));
        assert_eq!(foundopt!(@short 'h'), FoundOption::from(shortopt!(@flag 'h')));
    }

    #[test]
    fn opt_equality() {
        // foundopt to opt
        assert!(foundopt!(@long "help") == longopt!(@flag "help"));
        assert!(foundopt!(@long "help") != longopt!(@flag "version"));
        assert!(foundopt!(@long "help") == optpair!(@flag 'h', "help"));
        assert!(foundopt!(@long "help") != optpair!(@flag 'V', "version"));
        assert!(foundopt!(@short 'h')   == shortopt!(@flag 'h'));
        assert!(foundopt!(@short 'h')   != shortopt!(@flag 'V'));
        assert!(foundopt!(@short 'h')   == optpair!(@flag 'h', "help"));
        assert!(foundopt!(@short 'h')   != optpair!(@flag 'V', "version"));

        // opt to foundopt
        assert!(longopt!(@flag "help")         == foundopt!(@long "help"));
        assert!(longopt!(@flag "version")      != foundopt!(@long "help"));
        assert!(optpair!(@flag 'h', "help")    == foundopt!(@long "help"));
        assert!(optpair!(@flag 'V', "version") != foundopt!(@long "help"));
        assert!(shortopt!(@flag 'h')           == foundopt!(@short 'h'));
        assert!(shortopt!(@flag 'V')           != foundopt!(@short 'h'));
        assert!(optpair!(@flag 'h', "help")    == foundopt!(@short 'h'));
        assert!(optpair!(@flag 'V', "version") != foundopt!(@short 'h'));
    }

    #[test]
    fn find_equality() {
        // foundopt to findopt
        assert!(foundopt!(@long "help") == findopt!(@long "help"));
        assert!(foundopt!(@long "help") != findopt!(@long "version"));
        assert!(foundopt!(@long "help") == findopt!(@pair 'h', "help"));
        assert!(foundopt!(@long "help") != findopt!(@pair 'V', "version"));
        assert!(foundopt!(@long "help") != findopt!(@short 'h'));
        assert!(foundopt!(@short 'h')   != findopt!(@long "help"));
        assert!(foundopt!(@short 'h')   == findopt!(@pair 'h', "help"));
        assert!(foundopt!(@short 'h')   != findopt!(@pair 'V', "version"));
        assert!(foundopt!(@short 'h')   == findopt!(@short 'h'));
        assert!(foundopt!(@short 'h')   != findopt!(@short 'V'));

        // findopt to foundopt
        assert!(findopt!(@long "help")         == foundopt!(@long "help"));
        assert!(findopt!(@long "version")      != foundopt!(@long "help"));
        assert!(findopt!(@pair 'h', "help")    == foundopt!(@long "help"));
        assert!(findopt!(@pair 'V', "version") != foundopt!(@long "help"));
        assert!(findopt!(@short 'h')           != foundopt!(@long "help"));
        assert!(findopt!(@long "help")         != foundopt!(@short 'h'));
        assert!(findopt!(@pair 'h', "help")    == foundopt!(@short 'h'));
        assert!(findopt!(@pair 'V', "version") != foundopt!(@short 'h'));
        assert!(findopt!(@short 'h')           == foundopt!(@short 'h'));
        assert!(findopt!(@short 'V')           != foundopt!(@short 'h'));
    }
}

/// Test that checking option use works
#[test]
fn used() {
    let args = arg_list!(
        "--help",       // Known option
        "--ooo",        // Unknown option
        "--hah=123",    // Known option with data
        "-h",           // Known option
        "-d",           // Unknown option
        "-o321",        // Known option with data
        "--version=a",  // Known option, with unexpected data
    );
    let parser = get_parser();

    let expected = expected!([
        indexed_item!(0, Long, "help"),
        indexed_item!(1, UnknownLong, "ooo"),
        indexed_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
        indexed_item!(3, Short, 'h'),
        indexed_item!(4, UnknownShort, 'd'),
        indexed_item!(5, ShortWithData, 'o', "321", DataLocation::SameArg),
        indexed_item!(6, LongWithUnexpectedData, "version", "a"),
    ]);
    check_iter_result!(parser, args, expected);

    let expected = dm_expected!(
        problems: true,
        [
            item!(Long, "help"),
            item!(UnknownLong, "ooo"),
            item!(LongWithData, "hah", "123"),
            item!(Short, 'h'),
            item!(UnknownShort, 'd'),
            item!(ShortWithData, 'o', "321"),
            item!(LongWithUnexpectedData, "version", "a"),
        ]
    );
    let item_set = parser.parse(&args);
    check_result!(&Actual(item_set.clone()), &expected);

    // Check known and used
    assert_eq!(true, item_set.option_used(FindOption::Long("help")));
    assert_eq!(true, item_set.option_used(FindOption::Long("hah")));
    assert_eq!(true, item_set.option_used(FindOption::Short('h')));
    assert_eq!(true, item_set.option_used(FindOption::Short('o')));
    assert_eq!(true, item_set.option_used(FindOption::Pair('h', "help")));
    assert_eq!(false, item_set.option_used(FindOption::Long("version"))); //Negative, being strict with unexpected data
    // Check known and unused
    assert_eq!(false, item_set.option_used(FindOption::Long("foo")));
    assert_eq!(false, item_set.option_used(FindOption::Short('x')));
    assert_eq!(false, item_set.option_used(FindOption::Pair('x', "foo")));
    // Check unknown but used
    assert_eq!(false, item_set.option_used(FindOption::Long("ooo")));
    assert_eq!(false, item_set.option_used(FindOption::Short('d')));
    assert_eq!(false, item_set.option_used(FindOption::Pair('d', "ooo")));
    // Check unknown and unused
    assert_eq!(false, item_set.option_used(FindOption::Long("aaa")));
    assert_eq!(false, item_set.option_used(FindOption::Short('w')));
    assert_eq!(false, item_set.option_used(FindOption::Pair('w', "aaa")));
}

/// Test that checking option use works
#[test]
fn count() {
    let args = arg_list!(
        "--help",       // Known option
        "--ooo",        // Unknown option
        "--hah=123",    // Known option with data
        "-vhv", "-v",   // Known options
        "-d",           // Unknown option
        "--hah=456",    // Known option with data
        "-o321",        // Known option with data
        "--help",       // Known option
        "-o654",        // Known option with data
        "--version=a",  // Known option, with unexpected data
    );
    let parser = get_parser();

    let expected = expected!([
        indexed_item!(0, Long, "help"),
        indexed_item!(1, UnknownLong, "ooo"),
        indexed_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
        indexed_item!(3, Short, 'v'),
        indexed_item!(3, Short, 'h'),
        indexed_item!(3, Short, 'v'),
        indexed_item!(4, Short, 'v'),
        indexed_item!(5, UnknownShort, 'd'),
        indexed_item!(6, LongWithData, "hah", "456", DataLocation::SameArg),
        indexed_item!(7, ShortWithData, 'o', "321", DataLocation::SameArg),
        indexed_item!(8, Long, "help"),
        indexed_item!(9, ShortWithData, 'o', "654", DataLocation::SameArg),
        indexed_item!(10, LongWithUnexpectedData, "version", "a"),
    ]);
    check_iter_result!(parser, args, expected);

    let expected = dm_expected!(
        problems: true,
        [
            item!(Long, "help"),
            item!(UnknownLong, "ooo"),
            item!(LongWithData, "hah", "123"),
            item!(Short, 'v'),
            item!(Short, 'h'),
            item!(Short, 'v'),
            item!(Short, 'v'),
            item!(UnknownShort, 'd'),
            item!(LongWithData, "hah", "456"),
            item!(ShortWithData, 'o', "321"),
            item!(Long, "help"),
            item!(ShortWithData, 'o', "654"),
            item!(LongWithUnexpectedData, "version", "a"),
        ]
    );
    let item_set = parser.parse(&args);
    check_result!(&Actual(item_set.clone()), &expected);

    // Check known and used
    assert_eq!(2, item_set.count_instances(FindOption::Long("help")));
    assert_eq!(2, item_set.count_instances(FindOption::Long("hah")));
    assert_eq!(1, item_set.count_instances(FindOption::Short('h')));
    assert_eq!(3, item_set.count_instances(FindOption::Short('v')));
    assert_eq!(2, item_set.count_instances(FindOption::Short('o')));
    assert_eq!(3, item_set.count_instances(FindOption::Pair('h', "help")));
    assert_eq!(0, item_set.count_instances(FindOption::Long("version"))); //Being strict about the problem
    // Check known and unused
    assert_eq!(0, item_set.count_instances(FindOption::Long("foo")));
    assert_eq!(0, item_set.count_instances(FindOption::Short('x')));
    assert_eq!(0, item_set.count_instances(FindOption::Pair('x', "foo")));
    // Check unknown but used
    assert_eq!(0, item_set.count_instances(FindOption::Long("ooo")));
    assert_eq!(0, item_set.count_instances(FindOption::Short('d')));
    assert_eq!(0, item_set.count_instances(FindOption::Pair('d', "ooo")));
    // Check unknown and unused
    assert_eq!(0, item_set.count_instances(FindOption::Long("aaa")));
    assert_eq!(0, item_set.count_instances(FindOption::Short('w')));
    assert_eq!(0, item_set.count_instances(FindOption::Pair('w', "aaa")));

    // Check capped count
    assert_eq!(0, item_set.count_instances_capped(FindOption::Long("help"), 0));
    assert_eq!(1, item_set.count_instances_capped(FindOption::Long("help"), 1));
    assert_eq!(2, item_set.count_instances_capped(FindOption::Long("help"), 2));
    assert_eq!(2, item_set.count_instances_capped(FindOption::Long("help"), 3));
    assert_eq!(2, item_set.count_instances_capped(FindOption::Long("help"), 100));
    assert_eq!(0, item_set.count_instances_capped(FindOption::Short('h'), 0));
    assert_eq!(1, item_set.count_instances_capped(FindOption::Short('h'), 1));
    assert_eq!(1, item_set.count_instances_capped(FindOption::Short('h'), 2));
    assert_eq!(1, item_set.count_instances_capped(FindOption::Short('h'), 3));
    assert_eq!(0, item_set.count_instances_capped(FindOption::Pair('h', "help"), 0));
    assert_eq!(1, item_set.count_instances_capped(FindOption::Pair('h', "help"), 1));
    assert_eq!(2, item_set.count_instances_capped(FindOption::Pair('h', "help"), 2));
    assert_eq!(3, item_set.count_instances_capped(FindOption::Pair('h', "help"), 3));
    assert_eq!(3, item_set.count_instances_capped(FindOption::Pair('h', "help"), 4));
    assert_eq!(3, item_set.count_instances_capped(FindOption::Pair('h', "help"), 100));
}

mod missing_data {
    use super::*;

    /// Test that checking option use/count works with a long option missing data
    #[test]
    fn long() {
        let args = arg_list!(
            "--hah",    // Known option with missing data
        );
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, LongMissingData, "hah"),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(LongMissingData, "hah"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        assert_eq!(false, item_set.option_used(FindOption::Long("hah")));
    }

    /// Test that checking option use/count works with a short option missing data
    #[test]
    fn short() {
        let args = arg_list!(
            "-o",    // Known option with missing data
        );
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, ShortMissingData, 'o'),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(ShortMissingData, 'o'),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        assert_eq!(false, item_set.option_used(FindOption::Short('o')));
    }
}

/// Test that fetching the first problem item works
#[test]
fn first_problem() {
    let args = arg_list!(
        "--why",       // Unknown option
        "--fo",        // Ambiguous option
    );
    let parser = get_parser();

    let expected = expected!([
        indexed_item!(0, UnknownLong, "why"),
        indexed_item!(1, AmbiguousLong, "fo"),
    ]);
    check_iter_result!(parser, args, expected);

    let expected = dm_expected!(
        problems: true,
        [
            item!(UnknownLong, "why"),
            item!(AmbiguousLong, "fo"),
        ]
    );
    let item_set = parser.parse(&args);
    check_result!(&Actual(item_set.clone()), &expected);

    assert_eq!(2, item_set.get_problem_items().count());

    assert_eq!(item_set.get_first_problem(), Some(&ProblemItem::UnknownLong(OsStr::new("why"), None)));
}

/// Testing iterators over collections of item types
mod iter {
    use super::*;

    /// Test that iterating over item types works (all + good + problems)
    #[test]
    fn types() {
        let args = arg_list!(
            "abc",         // Positional
            "--why",       // Unknown option
            "--fo",        // Ambiguous option
            "--foo",       // Known option
            "--help=blah", // Known option with unexpected data
        );
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, Positional, "abc"),
            indexed_item!(1, UnknownLong, "why"),
            indexed_item!(2, AmbiguousLong, "fo"),
            indexed_item!(3, Long, "foo"),
            indexed_item!(4, LongWithUnexpectedData, "help", "blah"),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(Positional, "abc"),
                item!(UnknownLong, "why"),
                item!(AmbiguousLong, "fo"),
                item!(Long, "foo"),
                item!(LongWithUnexpectedData, "help", "blah"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        // All items
        let mut iter = item_set.get_items();
        assert_eq!(iter.next(), Some(&item!(Positional, "abc")));
        assert_eq!(iter.next(), Some(&item!(UnknownLong, "why")));
        assert_eq!(iter.next(), Some(&item!(AmbiguousLong, "fo")));
        assert_eq!(iter.next(), Some(&item!(Long, "foo")));
        assert_eq!(iter.next(), Some(&item!(LongWithUnexpectedData, "help", "blah")));
        assert_eq!(iter.next(), None);

        // Good items
        let mut iter = item_set.get_good_items();
        assert_eq!(iter.next(), Some(&Item::Positional(OsStr::new("abc"))));
        assert_eq!(iter.next(), Some(&Item::Long("foo", None)));
        assert_eq!(iter.next(), None);

        // Problem items
        let mut iter = item_set.get_problem_items();
        assert_eq!(iter.next(), Some(&ProblemItem::UnknownLong(OsStr::new("why"), None)));
        assert_eq!(iter.next(), Some(&ProblemItem::AmbiguousLong(OsStr::new("fo"))));
        assert_eq!(iter.next(), Some(&ProblemItem::LongWithUnexpectedData("help", OsStr::new("blah"))));
        assert_eq!(iter.next(), None);
    }

    /// Test that iterating over positionals works
    #[test]
    fn positionals() {
        let args = arg_list!(
            "abc",          // Positional
            "--help",       // Known option
            "def",          // Positional
            "hij",          // Positional
            "--jjj",        // Unknown option
            "klm",          // Positional
            "--",           // Early terminator
            "nop",          // Positional
            "--help",       // Positional
        );
        let mut parser = get_parser();
        parser.set_positionals_policy(PositionalsPolicy::Max(2));

        let expected = expected!([
            indexed_item!(0, Positional, "abc"),
            indexed_item!(1, Long, "help"),
            indexed_item!(2, Positional, "def"),
            indexed_item!(3, UnexpectedPositional, "hij"),
            indexed_item!(4, UnknownLong, "jjj"),
            indexed_item!(5, UnexpectedPositional, "klm"),
            // <early term item not served per setting>
            indexed_item!(7, UnexpectedPositional, "nop"),
            indexed_item!(8, UnexpectedPositional, "--help"),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(Positional, "abc"),
                item!(Long, "help"),
                item!(Positional, "def"),
                item!(UnexpectedPositional, "hij"),
                item!(UnknownLong, "jjj"),
                item!(UnexpectedPositional, "klm"),
                // <early term item not served per setting>
                item!(UnexpectedPositional, "nop"),
                item!(UnexpectedPositional, "--help"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        assert_eq!(2, item_set.get_positionals().count());

        let mut iter = item_set.get_positionals();
        assert_eq!(iter.next(), Some(OsStr::new("abc")));
        assert_eq!(iter.next(), Some(OsStr::new("def")));
        assert_eq!(iter.next(), None);

        assert_eq!(4, item_set.get_unexpected_positionals().count());

        let mut iter = item_set.get_unexpected_positionals();
        assert_eq!(iter.next(), Some(OsStr::new("hij")));
        assert_eq!(iter.next(), Some(OsStr::new("klm")));
        assert_eq!(iter.next(), Some(OsStr::new("nop")));
        assert_eq!(iter.next(), Some(OsStr::new("--help")));
        assert_eq!(iter.next(), None);
    }
}

/// Test that fetching specific positionals by index works
#[test]
fn positional_by_index() {
    let args = arg_list!(
        "abc",          // Positional
        "--help",       // Known option
        "def",          // Positional
        "hij",          // Positional
        "--jjj",        // Unknown option
        "klm",          // Positional
        "--",           // Early terminator
        "nop",          // Positional
        "--help",       // Positional
    );
    let parser = get_parser();

    let expected = expected!([
        indexed_item!(0, Positional, "abc"),
        indexed_item!(1, Long, "help"),
        indexed_item!(2, Positional, "def"),
        indexed_item!(3, Positional, "hij"),
        indexed_item!(4, UnknownLong, "jjj"),
        indexed_item!(5, Positional, "klm"),
        // <early term item not served per setting>
        indexed_item!(7, Positional, "nop"),
        indexed_item!(8, Positional, "--help"),
    ]);
    check_iter_result!(parser, args, expected);

    let expected = dm_expected!(
        problems: true,
        [
            item!(Positional, "abc"),
            item!(Long, "help"),
            item!(Positional, "def"),
            item!(Positional, "hij"),
            item!(UnknownLong, "jjj"),
            item!(Positional, "klm"),
            // <early term item not served per setting>
            item!(Positional, "nop"),
            item!(Positional, "--help"),
        ]
    );
    let item_set = parser.parse(&args);
    check_result!(&Actual(item_set.clone()), &expected);

    assert_eq!(Some(OsStr::new("abc")), item_set.get_positional(0));
    assert_eq!(Some(OsStr::new("klm")), item_set.get_positional(3));
    assert_eq!(None, item_set.get_positional(9));
    assert_eq!(Some(OsStr::new("--help")), item_set.get_positional(5));
    assert_eq!(Some(OsStr::new("hij")), item_set.get_positional(2));
}

/// Test retrieving last value for an option
#[test]
fn last_value() {
    let args = arg_list!(
        "--help",       // Known option
        "--ooo",        // Unknown option
        "--hah=123",    // Known option with data
        "-vhv", "-v",   // Known options
        "-d",           // Unknown option
        "--hah=456",    // Known option with data
        "-o321",        // Known option with data
        "--help",       // Known option
        "-o654",        // Known option with data
    );
    let parser = get_parser();

    let expected = expected!([
        indexed_item!(0, Long, "help"),
        indexed_item!(1, UnknownLong, "ooo"),
        indexed_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
        indexed_item!(3, Short, 'v'),
        indexed_item!(3, Short, 'h'),
        indexed_item!(3, Short, 'v'),
        indexed_item!(4, Short, 'v'),
        indexed_item!(5, UnknownShort, 'd'),
        indexed_item!(6, LongWithData, "hah", "456", DataLocation::SameArg),
        indexed_item!(7, ShortWithData, 'o', "321", DataLocation::SameArg),
        indexed_item!(8, Long, "help"),
        indexed_item!(9, ShortWithData, 'o', "654", DataLocation::SameArg),
    ]);
    check_iter_result!(parser, args, expected);

    let expected = dm_expected!(
        problems: true,
        [
            item!(Long, "help"),
            item!(UnknownLong, "ooo"),
            item!(LongWithData, "hah", "123"),
            item!(Short, 'v'),
            item!(Short, 'h'),
            item!(Short, 'v'),
            item!(Short, 'v'),
            item!(UnknownShort, 'd'),
            item!(LongWithData, "hah", "456"),
            item!(ShortWithData, 'o', "321"),
            item!(Long, "help"),
            item!(ShortWithData, 'o', "654"),
        ]
    );
    let item_set = parser.parse(&args);
    check_result!(&Actual(item_set.clone()), &expected);

    // Check known and used
    assert_eq!(None, item_set.get_last_value(FindOption::Long("help")));
    assert_eq!(None, item_set.get_last_value(FindOption::Short('h')));
    assert_eq!(None, item_set.get_last_value(FindOption::Pair('h', "help")));
    assert_eq!(Some(OsStr::new("456")), item_set.get_last_value(FindOption::Long("hah")));
    assert_eq!(Some(OsStr::new("654")), item_set.get_last_value(FindOption::Short('o')));
    assert_eq!(Some(OsStr::new("654")), item_set.get_last_value(FindOption::Pair('o', "hah")));
    // Check known and unused
    assert_eq!(None, item_set.get_last_value(FindOption::Long("foo")));
    assert_eq!(None, item_set.get_last_value(FindOption::Short('x')));
    assert_eq!(None, item_set.get_last_value(FindOption::Pair('x', "foo")));
    // Check unknown but used
    assert_eq!(None, item_set.get_last_value(FindOption::Long("ooo")));
    assert_eq!(None, item_set.get_last_value(FindOption::Short('d')));
    assert_eq!(None, item_set.get_last_value(FindOption::Pair('d', "ooo")));
    // Check unknown and unused
    assert_eq!(None, item_set.get_last_value(FindOption::Long("aaa")));
    assert_eq!(None, item_set.get_last_value(FindOption::Short('w')));
    assert_eq!(None, item_set.get_last_value(FindOption::Pair('w', "aaa")));
}

/// Test retrieving last value for an option with a mixed (optional data) type option with no data
#[test]
fn last_value_mixed() {
    let args = arg_list!(
        "--delay=",         // Used with a value (empty string)
        "--delay=bar",      // Used with a value
        "--delay",          // Used without a value
        "--ǝƃ=abc",         // Alternative, used with a value
    );
    let parser = get_parser();

    let expected = expected!([
        indexed_item!(0, LongWithData, "delay", "", DataLocation::SameArg),
        indexed_item!(1, LongWithData, "delay", "bar", DataLocation::SameArg),
        indexed_item!(2, LongWithoutData, "delay"),
        indexed_item!(3, LongWithData, "ǝƃ", "abc", DataLocation::SameArg),
    ]);
    check_iter_result!(parser, args, expected);

    let expected = dm_expected!(
        problems: false,
        [
            item!(LongWithData, "delay", ""),
            item!(LongWithData, "delay", "bar"),
            item!(LongWithoutData, "delay"),
            item!(LongWithData, "ǝƃ", "abc"),
        ]
    );
    let item_set = parser.parse(&args);
    check_result!(&Actual(item_set.clone()), &expected);

    // The normal method should ignore the non-value instance
    assert_eq!(None, item_set.get_last_value(FindOption::Long("help")));
    assert_eq!(Some(OsStr::new("bar")), item_set.get_last_value(FindOption::Long("delay")));
    assert_eq!(Some(OsStr::new("abc")), item_set.get_last_value(FindOption::Long("ǝƃ")));

    // The alternate method should not ignore it
    assert_eq!(None, item_set.get_last_value_mixed(FindOption::Long("help")));
    assert_eq!(Some(None), item_set.get_last_value_mixed(FindOption::Long("delay")));
    assert_eq!(Some(Some(OsStr::new("abc"))), item_set.get_last_value_mixed(FindOption::Long("ǝƃ")));
}

/// Test retrieving all values for an option
#[test]
fn all_values() {
    let args = arg_list!(
        "--help",       // Known option
        "--ooo",        // Unknown option
        "--hah=123",    // Known option with data
        "-vhv", "-v",   // Known options
        "-d",           // Unknown option
        "-o321",        // Known option with data
        "-o654",        // Known option with data
        "--hah=456",    // Known option with data
        "--help",       // Known option
        "-o987",        // Known option with data
    );
    let parser = get_parser();

    let expected = expected!([
        indexed_item!(0, Long, "help"),
        indexed_item!(1, UnknownLong, "ooo"),
        indexed_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
        indexed_item!(3, Short, 'v'),
        indexed_item!(3, Short, 'h'),
        indexed_item!(3, Short, 'v'),
        indexed_item!(4, Short, 'v'),
        indexed_item!(5, UnknownShort, 'd'),
        indexed_item!(6, ShortWithData, 'o', "321", DataLocation::SameArg),
        indexed_item!(7, ShortWithData, 'o', "654", DataLocation::SameArg),
        indexed_item!(8, LongWithData, "hah", "456", DataLocation::SameArg),
        indexed_item!(9, Long, "help"),
        indexed_item!(10, ShortWithData, 'o', "987", DataLocation::SameArg),
    ]);
    check_iter_result!(parser, args, expected);

    let expected = dm_expected!(
        problems: true,
        [
            item!(Long, "help"),
            item!(UnknownLong, "ooo"),
            item!(LongWithData, "hah", "123"),
            item!(Short, 'v'),
            item!(Short, 'h'),
            item!(Short, 'v'),
            item!(Short, 'v'),
            item!(UnknownShort, 'd'),
            item!(ShortWithData, 'o', "321"),
            item!(ShortWithData, 'o', "654"),
            item!(LongWithData, "hah", "456"),
            item!(Long, "help"),
            item!(ShortWithData, 'o', "987"),
        ]
    );
    let item_set = parser.parse(&args);
    check_result!(&Actual(item_set.clone()), &expected);

    let empty_vec: Vec<&OsStr> = Vec::new();

    // Check known and used
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Long("help")).collect::<Vec<&OsStr>>());
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Short('h')).collect::<Vec<&OsStr>>());
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Pair('h', "help")).collect::<Vec<&OsStr>>());
    assert_eq!(vec!["123","456"], item_set.get_all_values(FindOption::Long("hah")).collect::<Vec<&OsStr>>());
    assert_eq!(vec!["321","654","987"], item_set.get_all_values(FindOption::Short('o')).collect::<Vec<&OsStr>>());
    assert_eq!(vec!["123","321","654","456","987"],
               item_set.get_all_values(FindOption::Pair('o', "hah")).collect::<Vec<&OsStr>>());
    // Check known and unused
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Long("foo")).collect::<Vec<&OsStr>>());
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Short('x')).collect::<Vec<&OsStr>>());
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Pair('x', "foo")).collect::<Vec<&OsStr>>());
    // Check unknown but used
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Long("ooo")).collect::<Vec<&OsStr>>());
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Short('d')).collect::<Vec<&OsStr>>());
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Pair('d', "ooo")).collect::<Vec<&OsStr>>());
    // Check unknown and unused
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Long("aaa")).collect::<Vec<&OsStr>>());
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Short('w')).collect::<Vec<&OsStr>>());
    assert_eq!(empty_vec, item_set.get_all_values(FindOption::Pair('w', "aaa")).collect::<Vec<&OsStr>>());
}

/// Test retrieving all values for an option with a mixed (optional data) type option with no data
#[test]
fn all_values_mixed() {
    let args = arg_list!(
        "--delay=",         // Used with a value (empty string)
        "--delay=foo",      // Used with a value
        "--delay",          // Used without a value
        "--delay=bar",      // Used with a value
    );
    let parser = get_parser();

    let expected = expected!([
        indexed_item!(0, LongWithData, "delay", "", DataLocation::SameArg),
        indexed_item!(1, LongWithData, "delay", "foo", DataLocation::SameArg),
        indexed_item!(2, LongWithoutData, "delay"),
        indexed_item!(3, LongWithData, "delay", "bar", DataLocation::SameArg),
    ]);
    check_iter_result!(parser, args, expected);

    let expected = dm_expected!(
        problems: false,
        [
            item!(LongWithData, "delay", ""),
            item!(LongWithData, "delay", "foo"),
            item!(LongWithoutData, "delay"),
            item!(LongWithData, "delay", "bar"),
        ]
    );
    let item_set = parser.parse(&args);
    check_result!(&Actual(item_set.clone()), &expected);

    // The normal method should ignore the non-value instance
    let mut iter = item_set.get_all_values(FindOption::Long("help"));
    assert_eq!(None, iter.next());
    let mut iter = item_set.get_all_values(FindOption::Long("delay"));
    assert_eq!(Some(OsStr::new("")), iter.next());
    assert_eq!(Some(OsStr::new("foo")), iter.next());
    assert_eq!(Some(OsStr::new("bar")), iter.next());
    assert_eq!(None, iter.next());

    // The alternate method should not ignore it
    let mut iter = item_set.get_all_values_mixed(FindOption::Long("help"));
    assert_eq!(None, iter.next());
    let mut iter = item_set.get_all_values_mixed(FindOption::Long("delay"));
    assert_eq!(Some(Some(OsStr::new(""))), iter.next());
    assert_eq!(Some(Some(OsStr::new("foo"))), iter.next());
    assert_eq!(Some(None), iter.next());
    assert_eq!(Some(Some(OsStr::new("bar"))), iter.next());
    assert_eq!(None, iter.next());
}

/// Test that checking last option used in list works, as well as getting boolean state
mod last_used {
    use super::*;

    /// This tests basic handling within other options and with multiple instances, and with a short
    /// being the last
    #[test]
    fn basic_and_short_is_last() {
        let args = arg_list!(
            "--color",
            "--help",
            "-C",
            "--ooo",
            "--no-color",
            "-Ch",
            "--no-color",
            "--color",
            "-dC",
            "--version=a",
        );
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, Long, "color"),
            indexed_item!(1, Long, "help"),
            indexed_item!(2, Short, 'C'),
            indexed_item!(3, UnknownLong, "ooo"),
            indexed_item!(4, Long, "no-color"),
            indexed_item!(5, Short, 'C'),
            indexed_item!(5, Short, 'h'),
            indexed_item!(6, Long, "no-color"),
            indexed_item!(7, Long, "color"),
            indexed_item!(8, UnknownShort, 'd'),
            indexed_item!(8, Short, 'C'),
            indexed_item!(9, LongWithUnexpectedData, "version", "a"),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(Long, "color"),
                item!(Long, "help"),
                item!(Short, 'C'),
                item!(UnknownLong, "ooo"),
                item!(Long, "no-color"),
                item!(Short, 'C'),
                item!(Short, 'h'),
                item!(Long, "no-color"),
                item!(Long, "color"),
                item!(UnknownShort, 'd'),
                item!(Short, 'C'),
                item!(LongWithUnexpectedData, "version", "a"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Long("color"),
            FindOption::Long("no-color"),
            FindOption::Short('C'),
        ];
        let find2 = [
            FindOption::Pair('C', "color"),
            FindOption::Long("no-color"),
        ];
        let find_pos = FindOption::Pair('C', "color");
        let find_neg = FindOption::Long("no-color");
        let find_pos_list = [ FindOption::Long("color"), FindOption::Short('C') ];
        let find_neg_list = [ FindOption::Long("no-color") ];

        assert_eq!(Some(FoundOption::Short('C')), item_set.get_last_used(&find));
        assert_eq!(Some(FoundOption::Short('C')), item_set.get_last_used(&find2));
        assert_eq!(Some(true), item_set.get_bool_flag_state(find_pos, find_neg));
        assert_eq!(Some(true), item_set.get_bool_flag_state_multi(&find_pos_list, &find_neg_list));
    }

    /// Long with positive value is last
    #[test]
    fn long_pos_is_last() {
        let args = arg_list!("--help", "-C", "--no-color", "--color", "-d");
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, Long, "help"),
            indexed_item!(1, Short, 'C'),
            indexed_item!(2, Long, "no-color"),
            indexed_item!(3, Long, "color"),
            indexed_item!(4, UnknownShort, 'd'),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(Long, "help"),
                item!(Short, 'C'),
                item!(Long, "no-color"),
                item!(Long, "color"),
                item!(UnknownShort, 'd'),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Long("color"),
            FindOption::Long("no-color"),
            FindOption::Short('C'),
        ];
        let find2 = [
            FindOption::Pair('C', "color"),
            FindOption::Long("no-color"),
        ];
        let find_pos = FindOption::Pair('C', "color");
        let find_neg = FindOption::Long("no-color");
        let find_pos_list = [ FindOption::Long("color"), FindOption::Short('C') ];
        let find_neg_list = [ FindOption::Long("no-color") ];

        assert_eq!(Some(FoundOption::Long("color")), item_set.get_last_used(&find));
        assert_eq!(Some(FoundOption::Long("color")), item_set.get_last_used(&find2));
        assert_eq!(Some(true), item_set.get_bool_flag_state(find_pos, find_neg));
        assert_eq!(Some(true), item_set.get_bool_flag_state_multi(&find_pos_list, &find_neg_list));
    }

    /// Long with negative value is last
    #[test]
    fn long_neg_is_last() {
        let args = arg_list!("--help", "-C", "--color", "--no-color", "-d");
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, Long, "help"),
            indexed_item!(1, Short, 'C'),
            indexed_item!(2, Long, "color"),
            indexed_item!(3, Long, "no-color"),
            indexed_item!(4, UnknownShort, 'd'),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(Long, "help"),
                item!(Short, 'C'),
                item!(Long, "color"),
                item!(Long, "no-color"),
                item!(UnknownShort, 'd'),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Long("color"),
            FindOption::Long("no-color"),
            FindOption::Short('C'),
        ];
        let find_pos = FindOption::Pair('C', "color");
        let find_neg = FindOption::Long("no-color");
        let find_pos_list = [ FindOption::Long("color"), FindOption::Short('C') ];
        let find_neg_list = [ FindOption::Long("no-color") ];

        assert_eq!(Some(FoundOption::Long("no-color")), item_set.get_last_used(&find));
        assert_eq!(Some(false), item_set.get_bool_flag_state(find_pos, find_neg));
        assert_eq!(Some(false), item_set.get_bool_flag_state_multi(&find_pos_list, &find_neg_list));
    }

    /// Testing that a long with unexpected data is not considered
    #[test]
    fn long_with_unexpected_data_is_not_last() {
        let args = arg_list!("--help", "-C", "--no-color", "--color=data", "-d");
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, Long, "help"),
            indexed_item!(1, Short, 'C'),
            indexed_item!(2, Long, "no-color"),
            indexed_item!(3, LongWithUnexpectedData, "color", "data"),
            indexed_item!(4, UnknownShort, 'd'),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(Long, "help"),
                item!(Short, 'C'),
                item!(Long, "no-color"),
                item!(LongWithUnexpectedData, "color", "data"),
                item!(UnknownShort, 'd'),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Long("color"),
            FindOption::Long("no-color"),
            FindOption::Short('C'),
        ];
        let find_pos = FindOption::Pair('C', "color");
        let find_neg = FindOption::Long("no-color");
        let find_pos_list = [ FindOption::Long("color"), FindOption::Short('C') ];
        let find_neg_list = [ FindOption::Long("no-color") ];

        assert_eq!(Some(FoundOption::Long("no-color")), item_set.get_last_used(&find));
        assert_eq!(Some(false), item_set.get_bool_flag_state(find_pos, find_neg));
        assert_eq!(Some(false), item_set.get_bool_flag_state_multi(&find_pos_list, &find_neg_list));
    }

    /// No searched for items given
    #[test]
    fn not_present() {
        let args = arg_list!("--help");
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, Long, "help"),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: false,
            [
                item!(Long, "help"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Long("color"),
            FindOption::Long("no-color"),
            FindOption::Short('C'),
        ];
        let find_pos = FindOption::Pair('C', "color");
        let find_neg = FindOption::Long("no-color");
        let find_pos_list = [ FindOption::Long("color"), FindOption::Short('C') ];
        let find_neg_list = [ FindOption::Long("no-color") ];

        assert_eq!(None, item_set.get_last_used(&find));
        assert_eq!(None, item_set.get_bool_flag_state(find_pos, find_neg));
        assert_eq!(None, item_set.get_bool_flag_state_multi(&find_pos_list, &find_neg_list));
    }

    /// Empty argument list
    #[test]
    fn no_args() {
        let args: Vec<&OsStr> = Vec::new();
        let parser = get_parser();

        let expected = expected!([]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: false,
            []
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Long("color"),
            FindOption::Long("no-color"),
            FindOption::Short('C'),
        ];
        let find_pos = FindOption::Pair('C', "color");
        let find_neg = FindOption::Long("no-color");
        let find_pos_list = [ FindOption::Long("color"), FindOption::Short('C') ];
        let find_neg_list = [ FindOption::Long("no-color") ];

        assert_eq!(None, item_set.get_last_used(&find));
        assert_eq!(None, item_set.get_bool_flag_state(find_pos, find_neg));
        assert_eq!(None, item_set.get_bool_flag_state_multi(&find_pos_list, &find_neg_list));
    }
}

/// Test that checking first option used in list works
mod first_used {
    use super::*;

    /// This tests basic handling within other options and with multiple instances, and with a long
    /// being the first
    #[test]
    fn basic_and_long_is_first() {
        let args = arg_list!(
            "--color",
            "--help",
            "-C",
            "--ooo",
            "--no-color",
            "-Ch",
            "--version",
            "--color",
            "-dVC",
            "--version=a",
        );
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, Long, "color"),
            indexed_item!(1, Long, "help"),
            indexed_item!(2, Short, 'C'),
            indexed_item!(3, UnknownLong, "ooo"),
            indexed_item!(4, Long, "no-color"),
            indexed_item!(5, Short, 'C'),
            indexed_item!(5, Short, 'h'),
            indexed_item!(6, Long, "version"),
            indexed_item!(7, Long, "color"),
            indexed_item!(8, UnknownShort, 'd'),
            indexed_item!(8, Short, 'V'),
            indexed_item!(8, Short, 'C'),
            indexed_item!(9, LongWithUnexpectedData, "version", "a"),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(Long, "color"),
                item!(Long, "help"),
                item!(Short, 'C'),
                item!(UnknownLong, "ooo"),
                item!(Long, "no-color"),
                item!(Short, 'C'),
                item!(Short, 'h'),
                item!(Long, "version"),
                item!(Long, "color"),
                item!(UnknownShort, 'd'),
                item!(Short, 'V'),
                item!(Short, 'C'),
                item!(LongWithUnexpectedData, "version", "a"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Long("help"),
            FindOption::Long("version"),
            FindOption::Short('h'),
            FindOption::Short('V'),
        ];
        let find2 = [
            FindOption::Pair('h', "help"),
            FindOption::Pair('V', "version"),
        ];

        assert_eq!(Some(FoundOption::Long("help")), item_set.get_first_used(&find));
        assert_eq!(Some(FoundOption::Long("help")), item_set.get_first_used(&find2));
    }

    /// Short is first
    #[test]
    fn short_is_first() {
        let args = arg_list!("-Ch", "--help", "-V", "--no-color", "--color", "-d");
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, Short, 'C'),
            indexed_item!(0, Short, 'h'),
            indexed_item!(1, Long, "help"),
            indexed_item!(2, Short, 'V'),
            indexed_item!(3, Long, "no-color"),
            indexed_item!(4, Long, "color"),
            indexed_item!(5, UnknownShort, 'd'),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            [
                item!(Short, 'C'),
                item!(Short, 'h'),
                item!(Long, "help"),
                item!(Short, 'V'),
                item!(Long, "no-color"),
                item!(Long, "color"),
                item!(UnknownShort, 'd'),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Long("help"),
            FindOption::Long("version"),
            FindOption::Short('h'),
            FindOption::Short('V'),
        ];
        let find2 = [
            FindOption::Pair('h', "help"),
            FindOption::Pair('V', "version"),
        ];
        assert_eq!(Some(FoundOption::Short('h')), item_set.get_first_used(&find));
        assert_eq!(Some(FoundOption::Short('h')), item_set.get_first_used(&find2));
    }

    /// Tests that a different long can be reported other than the first in the set
    #[test]
    fn long_is_first2() {
        let args = arg_list!(
            "--color",
            "--version",
            "-C",
            "--ooo",
            "--no-color",
            "-Ch",
            "--help",
            "--color",
            "-dVC",
            "--version=a",
        );

        // Skipping the overall expectation check here for brevity

        let parser = get_parser();
        let item_set = parser.parse(&args);

        let find = [
            FindOption::Long("help"),
            FindOption::Long("version"),
            FindOption::Short('h'),
            FindOption::Short('V'),
        ];
        let find2 = [
            FindOption::Pair('h', "help"),
            FindOption::Pair('V', "version"),
        ];

        assert_eq!(Some(FoundOption::Long("version")), item_set.get_first_used(&find));
        assert_eq!(Some(FoundOption::Long("version")), item_set.get_first_used(&find2));
    }

    /// Short is first
    #[test]
    fn short_is_first2() {
        let args = arg_list!("-CV", "--help", "-h", "--no-color", "--color", "-d");

        // Skipping the overall expectation check here for brevity

        let parser = get_parser();
        let item_set = parser.parse(&args);

        let find = [
            FindOption::Long("help"),
            FindOption::Long("version"),
            FindOption::Short('h'),
            FindOption::Short('V'),
        ];
        let find2 = [
            FindOption::Pair('h', "help"),
            FindOption::Pair('V', "version"),
        ];
        assert_eq!(Some(FoundOption::Short('V')), item_set.get_first_used(&find));
        assert_eq!(Some(FoundOption::Short('V')), item_set.get_first_used(&find2));
    }

    /// Testing that a long with unexpected data is not considered
    #[test]
    fn long_with_unexpected_data_is_not_first() {
        let args = arg_list!("--help=data", "--version");
        let parser = get_parser();

        let expected = dm_expected!(
            problems: true,
            [
                item!(LongWithUnexpectedData, "help", "data"),
                item!(Long, "version"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Pair('h', "help"),
            FindOption::Pair('V', "version"),
        ];

        assert_eq!(Some(FoundOption::Long("version")), item_set.get_first_used(&find));
    }

    /// No searched for items given
    #[test]
    fn not_present() {
        let args = arg_list!("--color");
        let parser = get_parser();

        let expected = dm_expected!(
            problems: false,
            [
                item!(Long, "color"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Pair('h', "help"),
            FindOption::Pair('V', "version"),
        ];

        assert_eq!(None, item_set.get_first_used(&find));
    }

    /// Empty argument list
    #[test]
    fn no_args() {
        let args: Vec<&OsStr> = Vec::new();
        let parser = get_parser();

        let expected = dm_expected!(
            problems: false,
            []
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        let find = [
            FindOption::Pair('h', "help"),
            FindOption::Pair('V', "version"),
        ];

        assert_eq!(None, item_set.get_first_used(&find));
    }
}
