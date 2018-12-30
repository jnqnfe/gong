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
use self::common::{get_parser, get_base_opts, get_base_cmds, Actual, Expected, check_result};

/// Some of the tests here expect certain options to exist in the common options set, where such
/// options are **not** being used in the test arguments, so we need to assert that they definitely
/// are there.
#[test]
fn env() {
    let parser = get_parser();
    assert!(parser.options.long.contains(&longopt!("foo")));
    assert!(parser.options.short.contains(&shortopt!('x')));
}

/// Check `ItemClass` type checking shortcuts
#[test]
fn itemclass_type_shortcuts() {
    let item = expected_item!(0, Long, "help");
    assert!(item.is_ok() && !item.is_err() && !item.is_warn());

    let item = expected_item!(0, UnknownLong, "help");
    assert!(!item.is_ok() && !item.is_err() && item.is_warn());

    let item = expected_item!(0, LongMissingData, "help");
    assert!(!item.is_ok() && item.is_err() && !item.is_warn());
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
        assert_eq!(FindOption::from(longopt!("help")), findopt!(@long "help"));
        assert_eq!(FindOption::from(shortopt!('h')), findopt!(@short 'h'));
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
    let expected = expected!(
        error: false,
        warn: true,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
        [
            expected_item!(0, Long, "help"),
            expected_item!(1, UnknownLong, "ooo"),
            expected_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
            expected_item!(3, Short, 'h'),
            expected_item!(4, UnknownShort, 'd'),
            expected_item!(5, ShortWithData, 'o', "321", DataLocation::SameArg),
            expected_item!(6, LongWithUnexpectedData, "version", "a"),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    let parser = get_parser();
    let analysis = parser.parse(&args);
    check_result(&Actual(analysis.clone()), &expected);

    let item_set = &analysis.item_sets[0];

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
    let expected = expected!(
        error: false,
        warn: true,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
        [
            expected_item!(0, Long, "help"),
            expected_item!(1, UnknownLong, "ooo"),
            expected_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
            expected_item!(3, Short, 'v'),
            expected_item!(3, Short, 'h'),
            expected_item!(3, Short, 'v'),
            expected_item!(4, Short, 'v'),
            expected_item!(5, UnknownShort, 'd'),
            expected_item!(6, LongWithData, "hah", "456", DataLocation::SameArg),
            expected_item!(7, ShortWithData, 'o', "321", DataLocation::SameArg),
            expected_item!(8, Long, "help"),
            expected_item!(9, ShortWithData, 'o', "654", DataLocation::SameArg),
            expected_item!(10, LongWithUnexpectedData, "version", "a"),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    let parser = get_parser();
    let analysis = parser.parse(&args);
    check_result(&Actual(analysis.clone()), &expected);

    let item_set = &analysis.item_sets[0];

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
        let expected = expected!(
            error: true,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: false,
            [
                expected_item!(0, LongMissingData, "hah"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        let item_set = &analysis.item_sets[0];

        assert_eq!(false, item_set.option_used(FindOption::Long("hah")));
    }

    /// Test that checking option use/count works with a short option missing data
    #[test]
    fn short() {
        let args = arg_list!(
            "-o",    // Known option with missing data
        );
        let expected = expected!(
            error: true,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: false,
            [
                expected_item!(0, ShortMissingData, 'o'),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        let item_set = &analysis.item_sets[0];

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
    let expected = expected!(
        error: true,
        warn: true,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: true,
        [
            expected_item!(0, UnknownLong, "why"),
            expected_item!(1, AmbiguousLong, "fo"),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    let analysis = get_parser().parse(&args);
    check_result(&Actual(analysis.clone()), &expected);

    assert_eq!(2, analysis.get_problem_items().count());

    assert_eq!(analysis.get_first_problem(), Some(&expected_item!(0, UnknownLong, "why")));
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
        let expected = expected!(
            error: true,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: true,
            [
                expected_item!(0, UnknownCommand, "abc"),
                expected_item!(1, UnknownLong, "why"),
                expected_item!(2, AmbiguousLong, "fo"),
                expected_item!(3, Long, "foo"),
                expected_item!(4, LongWithUnexpectedData, "help", "blah"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        let item_set = &analysis.item_sets[0];

        // All items
        let mut iter = item_set.get_items();
        assert_eq!(iter.next(), Some(&expected_item!(0, UnknownCommand, "abc")));
        assert_eq!(iter.next(), Some(&expected_item!(1, UnknownLong, "why")));
        assert_eq!(iter.next(), Some(&expected_item!(2, AmbiguousLong, "fo")));
        assert_eq!(iter.next(), Some(&expected_item!(3, Long, "foo")));
        assert_eq!(iter.next(), Some(&expected_item!(4, LongWithUnexpectedData, "help", "blah")));
        assert_eq!(iter.next(), None);

        // Good items
        let mut iter = item_set.get_good_items();
        assert_eq!(iter.next(), Some(&expected_item!(3, Long, "foo")));
        assert_eq!(iter.next(), None);

        // Problem items
        let mut iter = item_set.get_problem_items();
        assert_eq!(iter.next(), Some(&expected_item!(0, UnknownCommand, "abc")));
        assert_eq!(iter.next(), Some(&expected_item!(1, UnknownLong, "why")));
        assert_eq!(iter.next(), Some(&expected_item!(2, AmbiguousLong, "fo")));
        assert_eq!(iter.next(), Some(&expected_item!(4, LongWithUnexpectedData, "help", "blah")));
        assert_eq!(iter.next(), None);
    }

    /// Test that iterating over positionals works
    #[test]
    fn positionals() {
        let args = arg_list!(
            "abc",          // Unknown command
            "--help",       // Known option
            "def",          // Positional
            "hij",          // Positional
            "--jjj",        // Unknown option
            "klm",          // Positional
            "--",           // Early terminator
            "nop",          // Positional
            "--help",       // Positional
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownCommand, "abc"),
                expected_item!(1, Long, "help"),
                expected_item!(2, Positional, "def"),
                expected_item!(3, Positional, "hij"),
                expected_item!(4, UnknownLong, "jjj"),
                expected_item!(5, Positional, "klm"),
                expected_item!(6, EarlyTerminator),
                expected_item!(7, Positional, "nop"),
                expected_item!(8, Positional, "--help"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        // Via item set

        let item_set = &analysis.item_sets[0];

        assert_eq!(5, item_set.get_positionals().count());

        let mut iter = item_set.get_positionals();
        assert_eq!(iter.next(), Some(OsStr::new("def")));
        assert_eq!(iter.next(), Some(OsStr::new("hij")));
        assert_eq!(iter.next(), Some(OsStr::new("klm")));
        assert_eq!(iter.next(), Some(OsStr::new("nop")));
        assert_eq!(iter.next(), Some(OsStr::new("--help")));
        assert_eq!(iter.next(), None);

        // Via analysis convenience function

        assert_eq!(5, analysis.get_positionals().count());

        let mut iter = analysis.get_positionals();
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
    let expected = expected!(
        error: false,
        warn: true,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
        [
            expected_item!(0, Long, "help"),
            expected_item!(1, UnknownLong, "ooo"),
            expected_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
            expected_item!(3, Short, 'v'),
            expected_item!(3, Short, 'h'),
            expected_item!(3, Short, 'v'),
            expected_item!(4, Short, 'v'),
            expected_item!(5, UnknownShort, 'd'),
            expected_item!(6, LongWithData, "hah", "456", DataLocation::SameArg),
            expected_item!(7, ShortWithData, 'o', "321", DataLocation::SameArg),
            expected_item!(8, Long, "help"),
            expected_item!(9, ShortWithData, 'o', "654", DataLocation::SameArg),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    let parser = get_parser();
    let analysis = parser.parse(&args);
    check_result(&Actual(analysis.clone()), &expected);

    let item_set = &analysis.item_sets[0];

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
    let expected = expected!(
        error: false,
        warn: true,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
        [
            expected_item!(0, Long, "help"),
            expected_item!(1, UnknownLong, "ooo"),
            expected_item!(2, LongWithData, "hah", "123", DataLocation::SameArg),
            expected_item!(3, Short, 'v'),
            expected_item!(3, Short, 'h'),
            expected_item!(3, Short, 'v'),
            expected_item!(4, Short, 'v'),
            expected_item!(5, UnknownShort, 'd'),
            expected_item!(6, ShortWithData, 'o', "321", DataLocation::SameArg),
            expected_item!(7, ShortWithData, 'o', "654", DataLocation::SameArg),
            expected_item!(8, LongWithData, "hah", "456", DataLocation::SameArg),
            expected_item!(9, Long, "help"),
            expected_item!(10, ShortWithData, 'o', "987", DataLocation::SameArg),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    let parser = get_parser();
    let analysis = parser.parse(&args);
    check_result(&Actual(analysis.clone()), &expected);

    let item_set = &analysis.item_sets[0];

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
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, Long, "color"),
                expected_item!(1, Long, "help"),
                expected_item!(2, Short, 'C'),
                expected_item!(3, UnknownLong, "ooo"),
                expected_item!(4, Long, "no-color"),
                expected_item!(5, Short, 'C'),
                expected_item!(5, Short, 'h'),
                expected_item!(6, Long, "no-color"),
                expected_item!(7, Long, "color"),
                expected_item!(8, UnknownShort, 'd'),
                expected_item!(8, Short, 'C'),
                expected_item!(9, LongWithUnexpectedData, "version", "a"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        let item_set = &analysis.item_sets[0];

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
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, Long, "help"),
                expected_item!(1, Short, 'C'),
                expected_item!(2, Long, "no-color"),
                expected_item!(3, Long, "color"),
                expected_item!(4, UnknownShort, 'd'),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        let item_set = &analysis.item_sets[0];

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
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, Long, "help"),
                expected_item!(1, Short, 'C'),
                expected_item!(2, Long, "color"),
                expected_item!(3, Long, "no-color"),
                expected_item!(4, UnknownShort, 'd'),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        let item_set = &analysis.item_sets[0];

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
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, Long, "help"),
                expected_item!(1, Short, 'C'),
                expected_item!(2, Long, "no-color"),
                expected_item!(3, LongWithUnexpectedData, "color", "data"),
                expected_item!(4, UnknownShort, 'd'),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        let item_set = &analysis.item_sets[0];

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
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, Long, "help"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        let item_set = &analysis.item_sets[0];

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
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false, []),
            cmd_set: Some(get_base_cmds())
        );
        let parser = get_parser();
        let analysis = parser.parse(&args);
        check_result(&Actual(analysis.clone()), &expected);

        let item_set = &analysis.item_sets[0];

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
