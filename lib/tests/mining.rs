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
use gong::{shortopt, longopt, findopt, foundopt};
use gong::analysis::*;
use self::common::{get_parser, get_base_opts, Actual, Expected};

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
        opt_set: get_base_opts(),
        [
            dm_item!(0, Long, "help"),
            dm_item!(1, UnknownLong, "ooo"),
            dm_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
            dm_item!(3, Short, 'h'),
            dm_item!(4, UnknownShort, 'd'),
            dm_item!(5, ShortWithData, 'o', "321", DataLocation::SameArg),
            dm_item!(6, LongWithUnexpectedData, "version", "a"),
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
    assert_eq!(true, item_set.option_used(FindOption::Long("version"))); //Positive, ignoring unexpected data
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
        opt_set: get_base_opts(),
        [
            dm_item!(0, Long, "help"),
            dm_item!(1, UnknownLong, "ooo"),
            dm_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
            dm_item!(3, Short, 'v'),
            dm_item!(3, Short, 'h'),
            dm_item!(3, Short, 'v'),
            dm_item!(4, Short, 'v'),
            dm_item!(5, UnknownShort, 'd'),
            dm_item!(6, LongWithData, "hah", "456", DataLocation::SameArg),
            dm_item!(7, ShortWithData, 'o', "321", DataLocation::SameArg),
            dm_item!(8, Long, "help"),
            dm_item!(9, ShortWithData, 'o', "654", DataLocation::SameArg),
            dm_item!(10, LongWithUnexpectedData, "version", "a"),
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
    assert_eq!(1, item_set.count_instances(FindOption::Long("version")));
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
            opt_set: get_base_opts(),
            [
                dm_item!(0, LongMissingData, "hah"),
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
            opt_set: get_base_opts(),
            [
                dm_item!(0, ShortMissingData, 'o'),
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
        opt_set: get_base_opts(),
        [
            dm_item!(0, UnknownLong, "why"),
            dm_item!(1, AmbiguousLong, "fo"),
        ]
    );
    let item_set = parser.parse(&args);
    check_result!(&Actual(item_set.clone()), &expected);

    assert_eq!(2, item_set.get_problem_items().count());

    assert_eq!(item_set.get_first_problem(), Some(&ProblemItem::UnknownLong(OsStr::new("why"))));
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
            opt_set: get_base_opts(),
            [
                dm_item!(0, Positional, "abc"),
                dm_item!(1, UnknownLong, "why"),
                dm_item!(2, AmbiguousLong, "fo"),
                dm_item!(3, Long, "foo"),
                dm_item!(4, LongWithUnexpectedData, "help", "blah"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        // All items
        let mut iter = item_set.get_items();
        assert_eq!(iter.next(), Some(&dm_item!(0, Positional, "abc")));
        assert_eq!(iter.next(), Some(&dm_item!(1, UnknownLong, "why")));
        assert_eq!(iter.next(), Some(&dm_item!(2, AmbiguousLong, "fo")));
        assert_eq!(iter.next(), Some(&dm_item!(3, Long, "foo")));
        assert_eq!(iter.next(), Some(&dm_item!(4, LongWithUnexpectedData, "help", "blah")));
        assert_eq!(iter.next(), None);

        // Good items
        let mut iter = item_set.get_good_items();
        assert_eq!(iter.next(), Some(&Item::Positional(OsStr::new("abc"))));
        assert_eq!(iter.next(), Some(&Item::Long("foo", None)));
        assert_eq!(iter.next(), None);

        // Problem items
        let mut iter = item_set.get_problem_items();
        assert_eq!(iter.next(), Some(&ProblemItem::UnknownLong(OsStr::new("why"))));
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
        let parser = get_parser();

        let expected = expected!([
            indexed_item!(0, Positional, "abc"),
            indexed_item!(1, Long, "help"),
            indexed_item!(2, Positional, "def"),
            indexed_item!(3, Positional, "hij"),
            indexed_item!(4, UnknownLong, "jjj"),
            indexed_item!(5, Positional, "klm"),
            indexed_item!(6, EarlyTerminator),
            indexed_item!(7, Positional, "nop"),
            indexed_item!(8, Positional, "--help"),
        ]);
        check_iter_result!(parser, args, expected);

        let expected = dm_expected!(
            problems: true,
            opt_set: get_base_opts(),
            [
                dm_item!(0, Positional, "abc"),
                dm_item!(1, Long, "help"),
                dm_item!(2, Positional, "def"),
                dm_item!(3, Positional, "hij"),
                dm_item!(4, UnknownLong, "jjj"),
                dm_item!(5, Positional, "klm"),
                dm_item!(6, EarlyTerminator),
                dm_item!(7, Positional, "nop"),
                dm_item!(8, Positional, "--help"),
            ]
        );
        let item_set = parser.parse(&args);
        check_result!(&Actual(item_set.clone()), &expected);

        assert_eq!(6, item_set.get_positionals().count());

        let mut iter = item_set.get_positionals();
        assert_eq!(iter.next(), Some(OsStr::new("abc")));
        assert_eq!(iter.next(), Some(OsStr::new("def")));
        assert_eq!(iter.next(), Some(OsStr::new("hij")));
        assert_eq!(iter.next(), Some(OsStr::new("klm")));
        assert_eq!(iter.next(), Some(OsStr::new("nop")));
        assert_eq!(iter.next(), Some(OsStr::new("--help")));
        assert_eq!(iter.next(), None);
    }
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
        opt_set: get_base_opts(),
        [
            dm_item!(0, Long, "help"),
            dm_item!(1, UnknownLong, "ooo"),
            dm_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
            dm_item!(3, Short, 'v'),
            dm_item!(3, Short, 'h'),
            dm_item!(3, Short, 'v'),
            dm_item!(4, Short, 'v'),
            dm_item!(5, UnknownShort, 'd'),
            dm_item!(6, LongWithData, "hah", "456", DataLocation::SameArg),
            dm_item!(7, ShortWithData, 'o', "321", DataLocation::SameArg),
            dm_item!(8, Long, "help"),
            dm_item!(9, ShortWithData, 'o', "654", DataLocation::SameArg),
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
        opt_set: get_base_opts(),
        [
            dm_item!(0, Long, "help"),
            dm_item!(1, UnknownLong, "ooo"),
            dm_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
            dm_item!(3, Short, 'v'),
            dm_item!(3, Short, 'h'),
            dm_item!(3, Short, 'v'),
            dm_item!(4, Short, 'v'),
            dm_item!(5, UnknownShort, 'd'),
            dm_item!(6, ShortWithData, 'o', "321", DataLocation::SameArg),
            dm_item!(7, ShortWithData, 'o', "654", DataLocation::SameArg),
            dm_item!(8, LongWithData, "hah", "456", DataLocation::SameArg),
            dm_item!(9, Long, "help"),
            dm_item!(10, ShortWithData, 'o', "987", DataLocation::SameArg),
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
            opt_set: get_base_opts(),
            [
                dm_item!(0, Long, "color"),
                dm_item!(1, Long, "help"),
                dm_item!(2, Short, 'C'),
                dm_item!(3, UnknownLong, "ooo"),
                dm_item!(4, Long, "no-color"),
                dm_item!(5, Short, 'C'),
                dm_item!(5, Short, 'h'),
                dm_item!(6, Long, "no-color"),
                dm_item!(7, Long, "color"),
                dm_item!(8, UnknownShort, 'd'),
                dm_item!(8, Short, 'C'),
                dm_item!(9, LongWithUnexpectedData, "version", "a"),
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
            opt_set: get_base_opts(),
            [
                dm_item!(0, Long, "help"),
                dm_item!(1, Short, 'C'),
                dm_item!(2, Long, "no-color"),
                dm_item!(3, Long, "color"),
                dm_item!(4, UnknownShort, 'd'),
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
            opt_set: get_base_opts(),
            [
                dm_item!(0, Long, "help"),
                dm_item!(1, Short, 'C'),
                dm_item!(2, Long, "color"),
                dm_item!(3, Long, "no-color"),
                dm_item!(4, UnknownShort, 'd'),
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

    /// Testing that a long with unexpected data is considered
    #[test]
    fn long_with_unexpected_data_is_last() {
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
            opt_set: get_base_opts(),
            [
                dm_item!(0, Long, "help"),
                dm_item!(1, Short, 'C'),
                dm_item!(2, Long, "no-color"),
                dm_item!(3, LongWithUnexpectedData, "color", "data"),
                dm_item!(4, UnknownShort, 'd'),
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

        assert_eq!(Some(FoundOption::Long("color")), item_set.get_last_used(&find));
        assert_eq!(Some(true), item_set.get_bool_flag_state(find_pos, find_neg));
        assert_eq!(Some(true), item_set.get_bool_flag_state_multi(&find_pos_list, &find_neg_list));
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
            opt_set: get_base_opts(),
            [
                dm_item!(0, Long, "help"),
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
            opt_set: get_base_opts(),
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
