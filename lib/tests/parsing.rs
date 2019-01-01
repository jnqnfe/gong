// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use std::ffi::{OsStr, OsString};
use gong::{option_set, longopt, command_set, command};
use gong::analysis::*;
use gong::parser::{Parser, OptionsMode};
use self::common::{get_parser, get_base_opts, get_base_cmds, Actual, Expected, check_result};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Arg list string types
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Check arg parsing accepts `&[OsString]`, `&[&OsStr]`, `&[String]` and `&[&str]`
///
/// All that we really need concern ourselves with is that it compiles.
#[test]
fn arg_list_owned_set() {
    // Test works (compiles) using a `OsString` based slice (as given from `env::args_os()` for real
    // args).
    let args: Vec<OsString> = vec![ OsString::from("--foo"), OsString::from("--bah") ];
    let _ = get_parser().parse(&args);

    // Test works (compiles) using an `&OsStr` based slice
    let args: Vec<&OsStr> = vec![ OsStr::new("--foo"), OsStr::new("--bah") ];
    let _ = get_parser().parse(&args);

    // Test works (compiles) using a `String` based slice (as given from `env::args()` for real args)
    let args: Vec<String> = vec![ String::from("--foo"), String::from("--bah") ];
    let _ = get_parser().parse(&args);

    // Test works (compiles) using a `&str` based slice
    let args: Vec<&str> = vec![ "--foo", "--bah" ];
    let _ = get_parser().parse(&args);
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Basic option handling
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Some general, basic argument handling
#[test]
fn basic() {
    let args = arg_list!(
        "abc",              // Unknown command
        "-", "z",           // Positionals
        "--help",           // Known long option
        "--xxx",            // Unknown long option
        "---yy",            // Extra dash should be taken as part of long option name
        "version",          // Known long option, but no prefix, thus positional
        "-bxs",             // Short option set, two unknown, one known (`x`)
        "ghi",              // Positional, containing known short option (`h`)
        "-a-",              // Dash in short opt set should come out as unknown short opt (can not
                            // be a known one as not allowed), so long as not the first in set, as
                            // would then arg would then be interpreted as long option or early
                            // terminator.
        "-h-",              // Same, but with known short opt in set, which should not matter
        "--",               // Early terminator
        "--foo",            // Known option, taken as positional due to early terminator
        "jkl",              // Positional either way
    );
    let expected = expected!(
        error: false,
        warn: true,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
        [
            expected_item!(0, UnknownCommand, "abc"),
            expected_item!(1, Positional, "-"),
            expected_item!(2, Positional, "z"),
            expected_item!(3, Long, "help"),
            expected_item!(4, UnknownLong, "xxx"),
            expected_item!(5, UnknownLong, "-yy"),
            expected_item!(6, Positional, "version"),
            expected_item!(7, UnknownShort, 'b'),
            expected_item!(7, Short, 'x'),
            expected_item!(7, UnknownShort, 's'),
            expected_item!(8, Positional, "ghi"),
            expected_item!(9, UnknownShort, 'a'),
            expected_item!(9, UnknownShort, '-'),
            expected_item!(10, Short, 'h'),
            expected_item!(10, UnknownShort, '-'),
            expected_item!(11, EarlyTerminator),
            expected_item!(12, Positional, "--foo"),
            expected_item!(13, Positional, "jkl"),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    check_result(&Actual(get_parser().parse(&args)), &expected);
}

/// Verify that option matching is case sensitive
#[test]
fn case_sensitivity() {
    let args = arg_list!("--Foo", "-O");
    let expected = expected!(
        error: false,
        warn: true,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
        [
            expected_item!(0, UnknownLong, "Foo"),
            expected_item!(1, UnknownShort, 'O'),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    check_result(&Actual(get_parser().parse(&args)), &expected);
}

/// Test that everything after an early terminator is taken to be a positional, including any
/// further early terminators.
#[test]
fn early_term() {
    let args = arg_list!(
        "--foo",    // Before the early terminator, should work as option
        "--",       // Our early terminator
        "--help",   // Should be affected, thus positional
        "--",       // Should be a positional, **not** another early terminator
        // A mix of various other items, some of which might be interpreted differently if it were
        // not for the early terminator.
        "-o", "--foo", "blah", "--bb", "-h", "--hah", "--hah=", "--", "--hah=a", "-oa", "-b",
    );
    let expected = expected!(
        error: false,
        warn: false,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
        [
            expected_item!(0, Long, "foo"),
            expected_item!(1, EarlyTerminator),
            expected_item!(2, Positional, "--help"),
            expected_item!(3, Positional, "--"),
            expected_item!(4, Positional, "-o"),
            expected_item!(5, Positional, "--foo"),
            expected_item!(6, Positional, "blah"),
            expected_item!(7, Positional, "--bb"),
            expected_item!(8, Positional, "-h"),
            expected_item!(9, Positional, "--hah"),
            expected_item!(10, Positional, "--hah="),
            expected_item!(11, Positional, "--"),
            expected_item!(12, Positional, "--hah=a"),
            expected_item!(13, Positional, "-oa"),
            expected_item!(14, Positional, "-b"),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    check_result(&Actual(get_parser().parse(&args)), &expected);
}

/// Test empty long option names with data param (-- on it’s own is obviously picked up as early
/// terminator, but what happens when an `=` is added?).
#[test]
fn long_no_name() {
    let args = arg_list!("--=a", "--=");
    let expected = expected!(
        error: false,
        warn: true,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
        [
            expected_item!(0, LongWithNoName),
            expected_item!(1, LongWithNoName),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    check_result(&Actual(get_parser().parse(&args)), &expected);
}

/// Test repetition - each instance should exist in the results in its own right. Note, data arg
/// tests are done later in the data arg section.
#[test]
fn repetition() {
    let args = arg_list!("--foo", "-h", "--version", "-h", "-x", "--blah", "--version", "-hhh");
    let expected = expected!(
        error: false,
        warn: true,
        @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
        [
            expected_item!(0, Long, "foo"),
            expected_item!(1, Short, 'h'),
            expected_item!(2, Long, "version"),
            expected_item!(3, Short, 'h'),
            expected_item!(4, Short, 'x'),
            expected_item!(5, UnknownLong, "blah"),
            expected_item!(6, Long, "version"),
            expected_item!(7, Short, 'h'),
            expected_item!(7, Short, 'h'),
            expected_item!(7, Short, 'h'),
        ]),
        cmd_set: Some(get_base_cmds())
    );
    check_result(&Actual(get_parser().parse(&args)), &expected);
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Utf-8 character handling
////////////////////////////////////////////////////////////////////////////////////////////////////

mod utf8 {
    use super::*;

    /// Some utf8 multi-byte char handling
    #[test]
    fn basic() {
        let args = arg_list!(
            "🗻∈🌏",              // Unknown command
            "--x❤x",            // Unknown long option
            "--🗻∈🌏",            // Another unknown long option
            "--ƒoo",            // Yet another
            "--ábc",           // Known long option
            "--ƒƒ=💖abc",        // Known long, with in-same-arg data
            "--ƒƒ", "abc💖",     // Known long, with in-next-arg data
            "--ƒƒ=",            // Known long option, taking data, which is empty string, in-same-arg
            "--ábc=",          // Known long option, does not take data, empty data ignored
            "--ábc=x💖z",       // Known long option, does not take data, unexpected data
            "--ƒ",              // Ambiguous long option
            "--=",              // Long with no name, and empty data
            "--=x💖z",           // Long with no name, and non-empty data
            "-ă",               // Unknown short option
            "-🗻∈🌏",             // More unknown short options
            "-❤",               // Known short option
            "-Ɛaşrg",           // Known short option, with in-same-arg data
            "-Ɛ", "argş",       // Known short option, with in-next-arg data
            "🌏∈🗻",              // Positional
        );
        let expected = expected!(
            error: true,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: true,
            [
                expected_item!(0, UnknownCommand, "🗻∈🌏"),
                expected_item!(1, UnknownLong, "x❤x"),
                expected_item!(2, UnknownLong, "🗻∈🌏"),
                expected_item!(3, UnknownLong, "ƒoo"),
                expected_item!(4, Long, "ábc"),
                expected_item!(5, LongWithData, "ƒƒ", "💖abc", DataLocation::SameArg),
                expected_item!(6, LongWithData, "ƒƒ", "abc💖", DataLocation::NextArg),
                expected_item!(8, LongWithData, "ƒƒ", "", DataLocation::SameArg),
                expected_item!(9, Long, "ábc"),
                expected_item!(10, LongWithUnexpectedData, "ábc", "x💖z"),
                expected_item!(11, AmbiguousLong, "ƒ"),
                expected_item!(12, LongWithNoName),
                expected_item!(13, LongWithNoName),
                expected_item!(14, UnknownShort, 'ă'),
                expected_item!(15, UnknownShort, '🗻'),
                expected_item!(15, UnknownShort, '∈'),
                expected_item!(15, UnknownShort, '🌏'),
                expected_item!(16, Short, '❤'),
                expected_item!(17, ShortWithData, 'Ɛ', "aşrg", DataLocation::SameArg),
                expected_item!(18, ShortWithData, 'Ɛ', "argş", DataLocation::NextArg),
                expected_item!(20, Positional, "🌏∈🗻"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Chars with combinator chars (e.g. accent)
    #[test]
    fn combinators() {
        let args = arg_list!("y̆", "-y̆", "--y̆", "ëéy̆", "-ëéy̆", "--ëéy̆", "--ábc", "--ábc");
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownCommand, "y̆"),
                expected_item!(1, UnknownShort, 'y'),        // `y`
                expected_item!(1, UnknownShort, '\u{0306}'), // breve
                expected_item!(2, UnknownLong, "y̆"),
                expected_item!(3, Positional, "ëéy̆"),
                expected_item!(4, UnknownShort, 'ë'),        // e+diaeresis
                expected_item!(4, UnknownShort, 'e'),        // `e`
                expected_item!(4, UnknownShort, '\u{0301}'), // acute accent
                expected_item!(4, UnknownShort, 'y'),        // `y`
                expected_item!(4, UnknownShort, '\u{0306}'), // breve
                expected_item!(5, UnknownLong, "ëéy̆"),
                expected_item!(6, UnknownLong, "ábc"),       // without combinator
                expected_item!(7, Long, "ábc"),              // with combinator
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Chars with variation selector
    ///
    /// Here we use the "heavy black heart" char with variation selector #16 (emoji).
    #[test]
    fn variation_selectors() {
        // Note: the following is the “black heart” character, followed by the variation selector
        // #16 (emoji) character.
        let args = arg_list!("❤️", "-❤️", "--❤️");
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownCommand, "❤️"),
                expected_item!(1, Short, '\u{2764}'),        // black-heart
                expected_item!(1, UnknownShort, '\u{fe0f}'), // emoji selector
                expected_item!(2, UnknownLong, "❤️"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Lone combinator chars
    #[test]
    fn lone_combinators() {
        let args = arg_list!("\u{0306}", "-\u{0306}", "--\u{0306}", "-\u{030a}");
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownCommand, "\u{0306}"),
                expected_item!(1, UnknownShort, '\u{0306}'),
                expected_item!(2, UnknownLong, "\u{0306}"),
                expected_item!(3, Short, '\u{030a}'),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Abbreviations
////////////////////////////////////////////////////////////////////////////////////////////////////

mod abbreviations {
    use super::*;

    /// Test handling of abbreviated long options, with ambiguity
    #[test]
    fn ambiguous() {
        let args = arg_list!(
            "--f",  // Abbreviation of both `foo` and `foobar`
            "--fo", // Same
        );
        let expected = expected!(
            error: true,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: false,
            [
                expected_item!(0, AmbiguousLong, "f"),
                expected_item!(1, AmbiguousLong, "fo"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test handling of abbreviated long options, without ambiguity
    #[test]
    fn unambiguous() {
        let args = arg_list!(
            "--foo",    // Exact match for `foo`
            "--foob",   // Abbreviation of `foobar` only
            "--fooba",  // Abbreviation of `foobar` only
            "--foobar", // Exact match for `foobar`
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Long, "foobar"),
                expected_item!(2, Long, "foobar"),
                expected_item!(3, Long, "foobar"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test handling when abbreviated matching is disabled
    #[test]
    fn disabled() {
        let args = arg_list!("--f", "--fo", "--foo", "--foob", "--fooba", "--foobar");
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownLong, "f"),
                expected_item!(1, UnknownLong, "fo"),
                expected_item!(2, Long, "foo"),
                expected_item!(3, UnknownLong, "foob"),
                expected_item!(4, UnknownLong, "fooba"),
                expected_item!(5, Long, "foobar"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let mut parser = get_parser();
        parser.settings.set_allow_abbreviations(false);
        check_result(&Actual(parser.parse(&args)), &expected);
    }

    /// Test that an exact match overrides ambiguity
    ///
    /// I.e. if it finds multiple abbreviated matches before the exact match (which can depends upon
    /// the order options are inserted into the set), that it keeps going to eventually find the
    /// exact match, rather than ending early as ambiguous.
    #[test]
    fn exact_override() {
        let opts = option_set!(@long [
            // Multiple options that “foo” will match as an abbreviation for before getting to the
            // exact match.
            longopt!("fooo"),
            longopt!("foooo"),
            longopt!("fooooo"),
            longopt!("foo"),    // Exact match for input `--foo`
        ]);

        let args = arg_list!("--foo");
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: &opts, error: false, warn: false,
            [
                expected_item!(0, Long, "foo"),
            ]),
            cmd_set: None
        );

        let parser = Parser::new(&opts, None);
        check_result(&Actual(parser.parse(&args)), &expected);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Data sub-arguments
////////////////////////////////////////////////////////////////////////////////////////////////////

mod data {
    use super::*;

    /// Test option with expected data arg for long options
    #[test]
    fn arg_placement_long() {
        let args = arg_list!(
            "--hah", "def", // In-next-arg
            "--help",       // Random
            "--hah=def",    // In-same-arg
            "--help",       // Random
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, LongWithData, "hah", "def", DataLocation::NextArg),
                expected_item!(2, Long, "help"),
                expected_item!(3, LongWithData, "hah", "def", DataLocation::SameArg),
                expected_item!(4, Long, "help"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test calculation of whether or not short-opt taking data is the last character in the short
    /// option set argument. This is done explicitly as its own test, even though happening to also
    /// be covered by the general arg-placement tests, for the purpose of catching off-by-one
    /// position tracking issues like that fixed in version 1.0.3.
    ///
    /// Note: calculation checks involving multi-byte chars is done separately below.
    #[test]
    fn arg_placement_short_calc() {
        let args = arg_list!("-oa", "g");
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, ShortWithData, 'o', "a", DataLocation::SameArg),
                expected_item!(1, UnknownCommand, "g"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test option with expected data arg, provided in next argument for short options
    #[test]
    fn arg_placement_short_next() {
        let args = arg_list!(
            // Here 'o' is valid and takes data, `x` is valid and does not take data.
            "-o", "def",    // Simple in-next-arg
            "-bo", "def",   // Char(s) before `o` should be correctly captured
            "-bxo", "def",  // Even chars that are valid short opts
            "-xao", "def",  // Different variation
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, ShortWithData, 'o', "def", DataLocation::NextArg),
                expected_item!(2, UnknownShort, 'b'),
                expected_item!(2, ShortWithData, 'o', "def", DataLocation::NextArg),
                expected_item!(4, UnknownShort, 'b'),
                expected_item!(4, Short, 'x'),
                expected_item!(4, ShortWithData, 'o', "def", DataLocation::NextArg),
                expected_item!(6, Short, 'x'),
                expected_item!(6, UnknownShort, 'a'),
                expected_item!(6, ShortWithData, 'o', "def", DataLocation::NextArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test option with expected data arg, provided in same argument for short options
    #[test]
    fn arg_placement_short_same() {
        let args = arg_list!(
            "-oa",         // `o` here takes data; trying here various combinations of
            "-oabc",       // different length, either side, with known and unknown other
            "-aob",        // (non-data-taking) short options.
            "-aobcd",
            "-abcod",
            "-abcodef",
            "-xoabc",
            "-oaxc",
            "-oxbc",
            "-oabx",
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, ShortWithData, 'o', "a", DataLocation::SameArg),
                expected_item!(1, ShortWithData, 'o', "abc", DataLocation::SameArg),
                expected_item!(2, UnknownShort, 'a'),
                expected_item!(2, ShortWithData, 'o', "b", DataLocation::SameArg),
                expected_item!(3, UnknownShort, 'a'),
                expected_item!(3, ShortWithData, 'o', "bcd", DataLocation::SameArg),
                expected_item!(4, UnknownShort, 'a'),
                expected_item!(4, UnknownShort, 'b'),
                expected_item!(4, UnknownShort, 'c'),
                expected_item!(4, ShortWithData, 'o', "d", DataLocation::SameArg),
                expected_item!(5, UnknownShort, 'a'),
                expected_item!(5, UnknownShort, 'b'),
                expected_item!(5, UnknownShort, 'c'),
                expected_item!(5, ShortWithData, 'o', "def", DataLocation::SameArg),
                expected_item!(6, Short, 'x'),
                expected_item!(6, ShortWithData, 'o', "abc", DataLocation::SameArg),
                expected_item!(7, ShortWithData, 'o', "axc", DataLocation::SameArg),
                expected_item!(8, ShortWithData, 'o', "xbc", DataLocation::SameArg),
                expected_item!(9, ShortWithData, 'o', "abx", DataLocation::SameArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test missing argument data for long option
    #[test]
    fn missing_long() {
        let args = arg_list!("--hah");
        let expected = expected!(
            error: true,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: false,
            [
                expected_item!(0, LongMissingData, "hah"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test missing argument data for short option
    #[test]
    fn missing_short() {
        let args = arg_list!("-bxso");
        let expected = expected!(
            error: true,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: true,
            [
                expected_item!(0, UnknownShort, 'b'),
                expected_item!(0, Short, 'x'),
                expected_item!(0, UnknownShort, 's'),
                expected_item!(0, ShortMissingData, 'o'),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test some misc. data handling.
    ///
    /// Unrecognised option with data; unrecognised with empty data; recognised with unexpected
    /// data; recognised with empty unexpected data; and that long option "component" splitting
    /// based on the first equals character (`=`) has no effect on short option set parsing.
    #[test]
    fn misc() {
        let args = arg_list!(
            "--xx=yy",   // Unrecognised long option, with data, name component is "xx"
            "--tt=",     // Unrecognised long option, with data, but data is empty string
            "-x",        // Random
            "--foo=bar", // Known long option, but does **not** take data, thus unexpected
            "--foo=",    // Same, but empty string, so data component should be ignored
            "-x",        // Random, ensures next-arg not taken as data for last one
            "-=",        // Equals char valid in short opt set, long opt name/value component
                         // splitting functionality should have no effect.
            "-a=b",      // Try with other chars
            "-o=b",      // Try with short option that takes data, which should consume it
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownLong, "xx"),
                expected_item!(1, UnknownLong, "tt"),
                expected_item!(2, Short, 'x'),
                expected_item!(3, LongWithUnexpectedData, "foo", "bar"),
                expected_item!(4, Long, "foo"),
                expected_item!(5, Short, 'x'),
                expected_item!(6, UnknownShort, '='),
                expected_item!(7, UnknownShort, 'a'),
                expected_item!(7, UnknownShort, '='),
                expected_item!(7, UnknownShort, 'b'),
                expected_item!(8, ShortWithData, 'o', "=b", DataLocation::SameArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test repetition - each instance should exist in the results in its own right. Note, basic
    /// non-data-arg related tests done separately already.
    #[test]
    fn repetition() {
        let args = arg_list!("--hah=a", "-o", "s", "--hah=b", "-obc");
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, LongWithData, "hah", "a", DataLocation::SameArg),
                expected_item!(1, ShortWithData, 'o', "s", DataLocation::NextArg),
                expected_item!(3, LongWithData, "hah", "b", DataLocation::SameArg),
                expected_item!(4, ShortWithData, 'o', "bc", DataLocation::SameArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test option with expected data arg, declared to be in same argument, but empty
    #[test]
    fn same_arg_empty() {
        /* The design decision for this scenario is that the data value be taken as an empty string,
         * not ignored and the next argument consumed. Rational below.
         *
         * Consider the following possible situations:
         *   1) `<prog-name> --hah=`
         *   2) `<prog-name> --hah=""`
         *   3) `<prog-name> --hah ""`
         *   4) `<prog-name> --hah= abc`
         *
         * An argument could be made that it is easy for a user (or script that fails to wrap a
         * value in quotes) to end up with an unwanted space between the equals (`=`) and the value,
         * as in #4, thus taking an empty string, with `abc` taken as a positional (possibly even
         * matching a command), and it thus makes sense to consume the next argument as a way of
         * trying to correct the users’ (presumed) mistake.
         *
         * However, we cannot tell the difference between the usage of #1 and #2, they are both
         * presented to program code as the same, thus if that were the behaviour, the only way to
         * provide an empty string would be via in-next-arg style (#3). In fact trying to do so
         * in-same-arg style like #2 might surprise the user by the fact that it would consume the
         * next argument. This arguably is much worse.
         */

        let args = arg_list!(
            "--hah=",   // Known option, takes data, not given, should be empty string
            "--help",   // Random known option, should not be take as data for previous
            "--hah=",   // Same again...
            "help",     // Unknown command this time, also should not be taken as data
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, LongWithData, "hah", "", DataLocation::SameArg),
                expected_item!(1, Long, "help"),
                expected_item!(2, LongWithData, "hah", "", DataLocation::SameArg),
                expected_item!(3, UnknownCommand, "help"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test option with expected data arg, provided in next argument, but empty. Note that users
    /// can easily achieve this in a command line using quotes (the command lines does not
    /// necessarily remove such an empty string item), e.g. `<prog-name> --hah ""`.
    #[test]
    fn next_arg_empty() {
        let args = arg_list!(
            "--hah", "",    // Long
            "-o", "",       // Short
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, LongWithData, "hah", "", DataLocation::NextArg),
                expected_item!(2, ShortWithData, 'o', "", DataLocation::NextArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test option with expected data arg, with data containing `=`. An `=` in a long option arg
    /// denotes an "in-arg" data value and thus it is broken up into name and data components. Here
    /// we check that an `=` does not result in unwanted behaviour in order positions.
    #[test]
    fn containing_equals() {
        let args = arg_list!(
            "--hah", "d=ef",    // Should just be treated as part of the data
            "--hah", "=",       // Should just be treated as data
            "--hah=d=ef",       // First `=` separates name and data, other is just part of the data
            "--hah==ef",        // Same here
            "--help",           // Random
            "--blah=ggg",       // Long option, but not a matching one, data should be ignored
            "-oa=b",            // Short option, should be part of `o` option’s data
            "-o=",              // Same
            "-o===o",           // Same
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, LongWithData, "hah", "d=ef", DataLocation::NextArg),
                expected_item!(2, LongWithData, "hah", "=", DataLocation::NextArg),
                expected_item!(4, LongWithData, "hah", "d=ef", DataLocation::SameArg),
                expected_item!(5, LongWithData, "hah", "=ef", DataLocation::SameArg),
                expected_item!(6, Long, "help"),
                expected_item!(7, UnknownLong, "blah"),
                expected_item!(8, ShortWithData, 'o', "a=b", DataLocation::SameArg),
                expected_item!(9, ShortWithData, 'o', "=", DataLocation::SameArg),
                expected_item!(10, ShortWithData, 'o', "===o", DataLocation::SameArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test argument data that looks like options
    #[test]
    fn looking_like_options() {
        let args = arg_list!(
            "--hah=--foo", "--hah", "--foo",   // With known long option, in-arg/next-arg
            "--hah=--blah", "--hah", "--blah", // Unknown
            "--hah=-h", "--hah", "-h",         // With known short option
            "--hah=-n", "--hah", "-n",         // Unknown
            "-o-h", "-o", "-h",                // Using short-opt, with known short opt
            "-o-n", "-o", "-n",                // Same, but unknown
            "-o--foo",                         // Short using known long lookalike
            "-o", "--hah",                     // Same, but long that take data
            "-o--blah", "-o", "--blah",        // With unknown
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, LongWithData, "hah", "--foo", DataLocation::SameArg),
                expected_item!(1, LongWithData, "hah", "--foo", DataLocation::NextArg),
                expected_item!(3, LongWithData, "hah", "--blah", DataLocation::SameArg),
                expected_item!(4, LongWithData, "hah", "--blah", DataLocation::NextArg),
                expected_item!(6, LongWithData, "hah", "-h", DataLocation::SameArg),
                expected_item!(7, LongWithData, "hah", "-h", DataLocation::NextArg),
                expected_item!(9, LongWithData, "hah", "-n", DataLocation::SameArg),
                expected_item!(10, LongWithData, "hah", "-n", DataLocation::NextArg),
                expected_item!(12, ShortWithData, 'o', "-h", DataLocation::SameArg),
                expected_item!(13, ShortWithData, 'o', "-h", DataLocation::NextArg),
                expected_item!(15, ShortWithData, 'o', "-n", DataLocation::SameArg),
                expected_item!(16, ShortWithData, 'o', "-n", DataLocation::NextArg),
                expected_item!(18, ShortWithData, 'o', "--foo", DataLocation::SameArg),
                expected_item!(19, ShortWithData, 'o', "--hah", DataLocation::NextArg),
                expected_item!(21, ShortWithData, 'o', "--blah", DataLocation::SameArg),
                expected_item!(22, ShortWithData, 'o', "--blah", DataLocation::NextArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test argument data that looks like early terminator
    #[test]
    fn looking_like_early_term() {
        let args = arg_list!(
            "--hah=--",     // In long option’s data, in-arg
            "--hah", "--",  // Same, next-arg
            "-o", "--",     // In short option’s data, in-arg
            "-o--",         // Same, next-arg
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, LongWithData, "hah", "--", DataLocation::SameArg),
                expected_item!(1, LongWithData, "hah", "--", DataLocation::NextArg),
                expected_item!(3, ShortWithData, 'o', "--", DataLocation::NextArg),
                expected_item!(5, ShortWithData, 'o', "--", DataLocation::SameArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test long option involving multi-byte chars, to ensure "in-arg" component splitting for
    /// instance.
    #[test]
    fn multibyte_long() {
        let args = arg_list!("--ƒƒ", "abc", "--ƒƒ=", "--ƒƒ=abc", "--ƒƒ=❤️", "--ƒƒ");
        let expected = expected!(
            error: true,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: false,
            [
                expected_item!(0, LongWithData, "ƒƒ", "abc", DataLocation::NextArg),
                expected_item!(2, LongWithData, "ƒƒ", "", DataLocation::SameArg),
                expected_item!(3, LongWithData, "ƒƒ", "abc", DataLocation::SameArg),
                expected_item!(4, LongWithData, "ƒƒ", "❤️", DataLocation::SameArg),
                expected_item!(5, LongMissingData, "ƒƒ"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test short options involving multi-byte chars to check offset calculations in iterating
    /// through a short option set and extracting "in-arg" data.
    #[test]
    fn multibyte_short() {
        let args = arg_list!(
            "-o", "❤",                // Single-byte option, multi-byte data char, next-arg
            "-Ɛ", "❤",                // Multi-byte short options, otherwise same
            "-o❤",                    // Single-byte option, multi-byte data char, same-arg
            "-❤oa", "-❤o❤", "-❤o❤Ɛ",  // Variations of multi-byte chars around single-byte option
            "-Ɛa", "-Ɛ❤",             // Multi-byte option, data same-arg
            // Misc. additional combinations
            "-❤Ɛa", "-❤Ɛ❤", "-❤Ɛa❤", "-x❤Ɛb❤",
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, ShortWithData, 'o', "❤", DataLocation::NextArg),
                expected_item!(2, ShortWithData, 'Ɛ', "❤", DataLocation::NextArg),
                expected_item!(4, ShortWithData, 'o', "❤", DataLocation::SameArg),
                expected_item!(5, Short, '❤'),
                expected_item!(5, ShortWithData, 'o', "a", DataLocation::SameArg),
                expected_item!(6, Short, '❤'),
                expected_item!(6, ShortWithData, 'o', "❤", DataLocation::SameArg),
                expected_item!(7, Short, '❤'),
                expected_item!(7, ShortWithData, 'o', "❤Ɛ", DataLocation::SameArg),
                expected_item!(8, ShortWithData, 'Ɛ', "a", DataLocation::SameArg),
                expected_item!(9, ShortWithData, 'Ɛ', "❤", DataLocation::SameArg),
                expected_item!(10, Short, '❤'),
                expected_item!(10, ShortWithData, 'Ɛ', "a", DataLocation::SameArg),
                expected_item!(11, Short, '❤'),
                expected_item!(11, ShortWithData, 'Ɛ', "❤", DataLocation::SameArg),
                expected_item!(12, Short, '❤'),
                expected_item!(12, ShortWithData, 'Ɛ', "a❤", DataLocation::SameArg),
                expected_item!(13, Short, 'x'),
                expected_item!(13, Short, '❤'),
                expected_item!(13, ShortWithData, 'Ɛ', "b❤", DataLocation::SameArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test the effect of Utf-8 combinator characters - does this break char iteration or byte
    /// position calculation whilst parsing a short option set. Safe to assume it won’t, but may as
    /// well throw down a few samples.
    #[test]
    fn multibyte_utf8combi_short() {
        let args = arg_list!(
            "-❤\u{fe0f}oa",
            "-❤o\u{030a}a",
            "-❤oa\u{030a}",
            "-❤\u{fe0f}o\u{030a}a",
            "-\u{030a}❤oa",
            "-x❤\u{fe0f}Ɛ\u{030a}b❤",
            "-x\u{030a}❤\u{fe0f}Ɛ\u{030a}b❤\u{fe0f}",
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, Short, '❤'),
                expected_item!(0, UnknownShort, '\u{fe0f}'),
                expected_item!(0, ShortWithData, 'o', "a", DataLocation::SameArg),
                expected_item!(1, Short, '❤'),
                expected_item!(1, ShortWithData, 'o', "\u{030a}a", DataLocation::SameArg),
                expected_item!(2, Short, '❤'),
                expected_item!(2, ShortWithData, 'o', "a\u{030a}", DataLocation::SameArg),
                expected_item!(3, Short, '❤'),
                expected_item!(3, UnknownShort, '\u{fe0f}'),
                expected_item!(3, ShortWithData, 'o', "\u{030a}a", DataLocation::SameArg),
                expected_item!(4, Short, '\u{030a}'),
                expected_item!(4, Short, '❤'),
                expected_item!(4, ShortWithData, 'o', "a", DataLocation::SameArg),
                expected_item!(5, Short, 'x'),
                expected_item!(5, Short, '❤'),
                expected_item!(5, UnknownShort, '\u{fe0f}'),
                expected_item!(5, ShortWithData, 'Ɛ', "\u{030a}b❤", DataLocation::SameArg),
                expected_item!(6, Short, 'x'),
                expected_item!(6, Short, '\u{030a}'),
                expected_item!(6, Short, '❤'),
                expected_item!(6, UnknownShort, '\u{fe0f}'),
                expected_item!(6, ShortWithData, 'Ɛ', "\u{030a}b❤\u{fe0f}", DataLocation::SameArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Command parsing
////////////////////////////////////////////////////////////////////////////////////////////////////

mod commands {
    use super::*;

    #[test]
    fn basic() {
        let args = arg_list!("commit");
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false, []),
            @itemset item_set!(cmd: "commit",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 1),
                error: false, warn: false,
            []),
            cmd_set: None
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// We do not support case-insensitive matching
    #[test]
    fn case_sensitivity() {
        let args = arg_list!("Commit");
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownCommand, "Commit"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check repeated use correctly fails, where same command is used
    ///
    /// After the first, the command set, if any, of that command is in effect, so the second use
    /// should be checked against that; since that does not contain it, this should fail!
    #[test]
    fn repeated_same() {
        let args = arg_list!("commit", "commit");
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false, []),
            @itemset item_set!(cmd: "commit",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 1),
                error: false, warn: false,
            [
                expected_item!(1, Positional, "commit"),
            ]),
            cmd_set: None
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check repeated use correctly fails, where different commands are used
    #[test]
    fn repeated_different() {
        let args = arg_list!("push", "commit");
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false, []),
            @itemset item_set!(cmd: "push",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 3),
                error: false, warn: true,
            [
                expected_item!(1, UnknownCommand, "commit"),
            ]),
            cmd_set: Some(cmdset_subcmdset_ref!(get_base_cmds(), 3))
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check early terminator forces command to be treated as positional
    #[test]
    fn after_early_term() {
        let args = arg_list!("--", "commit");
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, EarlyTerminator),
                expected_item!(1, Positional, "commit"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check works alongside option arguments
    #[test]
    fn with_options() {
        let args = arg_list!(
            "--foo", "-h",  // Long and short options from the main set
            "commit"        // Our command
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Short, 'h'),
            ]),
            @itemset item_set!(cmd: "commit",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 1),
                error: false, warn: false,
            []),
            cmd_set: None
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check name clash with option
    #[test]
    fn name_clash() {
        let args = arg_list!(
            "--foo", // As long option
            "foo"    // As command
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, Long, "foo"),
            ]),
            @itemset item_set!(cmd: "foo",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 0),
                error: false, warn: false,
            []),
            cmd_set: None
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check names that look like options, take preference as options
    ///
    /// Command names simply should not be prefixed like this.
    #[test]
    fn name_like_option() {
        let opts = option_set!(@long [
            longopt!("foo"),
        ]);
        let cmds = command_set!([
            command!("--foo"),
            command!("--bar"),
        ]);

        let args = arg_list!("--foo", "--bar");
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: &opts, error: false, warn: true,
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, UnknownLong, "bar"),
            ]),
            cmd_set: Some(&cmds)
        );

        let parser = Parser::new(&opts, Some(&cmds));
        check_result(&Actual(parser.parse(&args)), &expected);
    }

    /// Check consumed as option argument data
    #[test]
    fn opt_data_consumed() {
        let args = arg_list!(
            "--hah",    // Long option taking data
            "commit"    // Available command, but should be consumed as option data
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, LongWithData, "hah", "commit", DataLocation::NextArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check not recognised as command in secondary non-option position, and first is reported as
    /// an unrecognised command.
    #[test]
    fn following_unknown() {
        let args = arg_list!(
            "--hah", "foo",     // Option taking data
            "blah",             // Unknown command
            "commit"            // A known command, but one or more non-options already given
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, LongWithData, "hah", "foo", DataLocation::NextArg),
                expected_item!(2, UnknownCommand, "blah"),
                expected_item!(3, Positional, "commit"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check that the option set is changed following the command, with command that has no option
    /// set of its own.
    #[test]
    fn option_set_change_none() {
        let args = arg_list!(
            "--foo", "-h",      // Options from main set
            "commit",           // Our command, which has no options of its own
            "--foo", "-oq"      // Options, some match the main set, but set in use should have
                                // changed, resulting in them not being recognised.
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Short, 'h'),
            ]),
            @itemset item_set!(cmd: "commit",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 1),
                error: false, warn: true,
            [
                expected_item!(3, UnknownLong, "foo"),
                expected_item!(4, UnknownShort, 'o'),
                expected_item!(4, UnknownShort, 'q'),
            ]),
            cmd_set: None
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check that the option set is changed following the command, with command that has an option
    /// set of its own.
    #[test]
    fn option_set_change_some() {
        let args = arg_list!(
            "--foo", "-h",      // Options from the main set
            "push",             // Our command, which has options of its own
            "--foo",            // Option unknown in command’s option set, and should **not**
                                // be matched against same option in main set.
            "--help",           // Option applicable to command (as well as main set, though
                                // the main set is irrelevant here).
            "--tags",           // Option applicable to command
            "-o"                // Option unknown to command, but exists in main set
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Short, 'h'),
            ]),
            @itemset item_set!(cmd: "push",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 3),
                error: false, warn: true,
            [
                expected_item!(3, UnknownLong, "foo"),
                expected_item!(4, Long, "help"),
                expected_item!(5, Long, "tags"),
                expected_item!(6, UnknownShort, 'o'),
            ]),
            cmd_set: Some(cmdset_subcmdset_ref!(get_base_cmds(), 3))
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check nested sub-commands
    #[test]
    fn nested() {
        let args = arg_list!(
            "branch",           // Our command, which has options of its own
            "--foo",            // Option unknown to command, known to main set
            "--help",           // Option known to command (and main set)
            "--sorted",         // Option known to command (only)
            "--show-current",   // Option for sub-command, but unknown to command itself
            "list",             // Sub-command
            "--foo",            // Option known to sub-command (and main, but not parent command)
            "--help",           // Option known to sub-command
            "--sorted",         // Option unknown to sub-command (but known to parent)
            "--show-current",   // Option known to sub-command (only)
            "remote",           // Sub-command (level 2)
            "--foo",            // Option unknown to sub-command
            "--help",           // Option unknown to sub-command
            "--nope",           // Option unknown to sub-command
            "--show-current",   // Option unknown to sub-command
            "blah"              // Positional
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false, []),
            @itemset item_set!(cmd: "branch",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 4),
                error: false, warn: true,
            [
                expected_item!(1, UnknownLong, "foo"),
                expected_item!(2, Long, "help"),
                expected_item!(3, Long, "sorted"),
                expected_item!(4, UnknownLong, "show-current"),
            ]),
            @itemset item_set!(cmd: "list",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 4, 2),
                error: false, warn: true,
            [
                expected_item!(6, Long, "foo"),
                expected_item!(7, Long, "help"),
                expected_item!(8, UnknownLong, "sorted"),
                expected_item!(9, Long, "show-current"),
            ]),
            @itemset item_set!(cmd: "remote",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 4, 2, 1),
                error: false, warn: true,
            [
                expected_item!(11, UnknownLong, "foo"),
                expected_item!(12, UnknownLong, "help"),
                expected_item!(13, UnknownLong, "nope"),
                expected_item!(14, UnknownLong, "show-current"),
                expected_item!(15, Positional, "blah"),
            ]),
            cmd_set: None
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check early terminator works in nested sub-command usage
    #[test]
    fn nested_earlyterm() {
        let args = arg_list!(
            "branch",           // Our command, which has options of its own
            "--foo",            // Option unknown to command, known to main set
            "--help",           // Option known to command (and main set)
            "--sorted",         // Option known to command (only)
            "--show-current",   // Option for sub-command, but unknown to command itself
            "del",              // Sub-command
            "--foo",            // Option unknown to sub-command
            "--",               // Early terminator
            "--show-current",   // Everything here onwards should be taken as positionals
            "remotely",         // Sub-command (level 2)
            "--foo",            // Option unknown to sub-command
            "blah"              // Positional
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false, []),
            @itemset item_set!(cmd: "branch",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 4),
                error: false, warn: true,
            [
                expected_item!(1, UnknownLong, "foo"),
                expected_item!(2, Long, "help"),
                expected_item!(3, Long, "sorted"),
                expected_item!(4, UnknownLong, "show-current"),
            ]),
            @itemset item_set!(cmd: "del",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 4, 1),
                error: false, warn: true,
            [
                expected_item!(6, UnknownLong, "foo"),
                expected_item!(7, EarlyTerminator),
                expected_item!(8, Positional, "--show-current"),
                expected_item!(9, Positional, "remotely"),
                expected_item!(10, Positional, "--foo"),
                expected_item!(11, Positional, "blah"),
            ]),
            cmd_set: Some(cmdset_subcmdset_ref!(get_base_cmds(), 4, 1))
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check known command from wrong set
    #[test]
    fn nested_wrong_set() {
        let args = arg_list!(
            "branch",   // Primary command
            "del",      // Sub-command (level 1)
            "list",     // Sub-command from level 1, used at level 2, thus unrecognised
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false, []),
            @itemset item_set!(cmd: "branch",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 4),
                error: false, warn: false,
            []),
            @itemset item_set!(cmd: "del",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 4, 1),
                error: false, warn: true,
            [
                expected_item!(2, UnknownCommand, "list"),
            ]),
            cmd_set: Some(cmdset_subcmdset_ref!(get_base_cmds(), 4, 1))
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check known sub-command, given after a non-option that is not a known sub-command
    #[test]
    fn nested_following_unknown() {
        let args = arg_list!(
            "branch",   // Primary command
            "blah",     // Unknown sub-command
            "list",     // Known, sub-command, but following unknown, so only positional
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false, []),
            @itemset item_set!(cmd: "branch",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 4),
                error: false, warn: true,
            [
                expected_item!(1, UnknownCommand, "blah"),
                expected_item!(2, Positional, "list"),
            ]),
            cmd_set: Some(cmdset_subcmdset_ref!(get_base_cmds(), 4))
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Trimming - We do not do any, prove this and ensure implementation stays consistent with respect
// to this, by causing test suite failure on change.
////////////////////////////////////////////////////////////////////////////////////////////////////

mod trimming {
    use super::*;

    #[test]
    fn basic() {
        let args = arg_list!(
            // In all of these cases the whitespace is expected to be preserved, which thus might
            // cause arguments to be seen as positionals, or cause option/command match failure.
            " --foo",                   // Whitespace at start of long option style argument
            "--foo ",                   // Whitespace at end of long option name
            "-- foo",                   // Whitespace at start of long option name
            "--f o\to",                 // Whitespace in middle of option name
            " -a",                      // Whitespace at start of short option style argument
            "-a ",                      // Whitespace at end of short option set
            "- a",                      // Whitespace between prefix and first non-whitespace short
            "-a \tb",                   // Whitespace in middle of short option set
            "--hah=   a  b\t c ",       // Whitespace in in-same-arg long option data value
            "--hah", "   a  b\t c ",    // Whitespace in in-next-arg long option data value
            "-o   a  b\t c ",           // Whitespace in in-same-arg short option data value
            "-o", "   a  b\t c ",       // Whitespace in in-next-arg short option data value
            "   a  b\t c ",             // Whitespace in positional
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownCommand, " --foo"),
                expected_item!(1, UnknownLong, "foo "),
                expected_item!(2, UnknownLong, " foo"),
                expected_item!(3, UnknownLong, "f o\to"),
                expected_item!(4, Positional, " -a"),
                expected_item!(5, UnknownShort, 'a'),
                expected_item!(5, UnknownShort, ' '),
                expected_item!(6, UnknownShort, ' '),
                expected_item!(6, UnknownShort, 'a'),
                expected_item!(7, UnknownShort, 'a'),
                expected_item!(7, UnknownShort, ' '),
                expected_item!(7, UnknownShort, '\t'),
                expected_item!(7, UnknownShort, 'b'),
                expected_item!(8, LongWithData, "hah", "   a  b\t c ", DataLocation::SameArg),
                expected_item!(9, LongWithData, "hah", "   a  b\t c ", DataLocation::NextArg),
                expected_item!(11, ShortWithData, 'o', "   a  b\t c ", DataLocation::SameArg),
                expected_item!(12, ShortWithData, 'o', "   a  b\t c ", DataLocation::NextArg),
                expected_item!(14, Positional, "   a  b\t c "),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Alt-mode option parsing
////////////////////////////////////////////////////////////////////////////////////////////////////

mod alt_mode {
    use super::*;

    /* Some tests for alt-mode option parsing (no short options, long options use single dash
     * prefix. There is little difference in parsing, so few tests should be required. */

    /// Check a range of inputs
    #[test]
    fn basic() {
        let args = arg_list!(
            "abc",          // Unknown command
            "-",            // Should be positional
            "-help",        // Known option, via alt-mode single dash
            "-hah=abc",     // Data-taking variant, in-arg
            "-hah", "cba",  // Same, next-arg
            "-hah=",        // Same, in-arg, data is empty string
            "-=",           // Option with data arg, which is empty, also empty name
            "-=abc",        // Similar, empty name, data provided though, which should be ignored
            "-bxs",         // `x` is a known short opt, but they should be ignored
            "--foo",        // Known option, “standard” mode syntax, the second dash should be taken
                            // as being a part of the name.
            "-f",           // Ambiguous long option, matches both `foo` and `foobar`
            "-foo",         // Matches both `foo` and `foobar`, but matches `foo` exactly
            "-foob",        // Unique abbreviation to `foobar`
            "-❤",           // Check known short opt not taken as such
            "--",           // Early term
            "-help",        // Known option, should be positional though due to early terminator
        );
        let expected = expected!(
            error: true,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: true,
            [
                expected_item!(0, UnknownCommand, "abc"),
                expected_item!(1, Positional, "-"),
                expected_item!(2, Long, "help"),
                expected_item!(3, LongWithData, "hah", "abc", DataLocation::SameArg),
                expected_item!(4, LongWithData, "hah", "cba", DataLocation::NextArg),
                expected_item!(6, LongWithData, "hah", "", DataLocation::SameArg),
                expected_item!(7, LongWithNoName),
                expected_item!(8, LongWithNoName),
                expected_item!(9, UnknownLong, "bxs"),
                expected_item!(10, UnknownLong, "-foo"),
                expected_item!(11, AmbiguousLong, "f"),
                expected_item!(12, Long, "foo"),
                expected_item!(13, Long, "foobar"),
                expected_item!(14, UnknownLong, "❤"),
                expected_item!(15, EarlyTerminator),
                expected_item!(16, Positional, "-help"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let mut parser = get_parser();
        parser.settings.set_mode(OptionsMode::Alternate);
        check_result(&Actual(parser.parse(&args)), &expected);
    }

    /// Check unexpected and missing data
    #[test]
    fn data_basic() {
        let args = arg_list!(
            "-foo=abc", // Known option, takes no data
            "-foo=",    // Same, data is empty though so should just be ignored
            "-hah",     // Known option, takes data, none provided
        );
        let expected = expected!(
            error: true,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: true, warn: true,
            [
                expected_item!(0, LongWithUnexpectedData, "foo", "abc"),
                expected_item!(1, Long, "foo"),
                expected_item!(2, LongMissingData, "hah"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let mut parser = get_parser();
        parser.settings.set_mode(OptionsMode::Alternate);
        check_result(&Actual(parser.parse(&args)), &expected);
    }

    /// Test argument data that looks like early terminator
    #[test]
    fn data_looking_like_early_term() {
        let args = arg_list!(
            "-hah=--",      // Known option, takes data, in-arg
            "-hah", "--",   // Same, next-arg
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, LongWithData, "hah", "--", DataLocation::SameArg),
                expected_item!(1, LongWithData, "hah", "--", DataLocation::NextArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let mut parser = get_parser();
        parser.settings.set_mode(OptionsMode::Alternate);
        check_result(&Actual(parser.parse(&args)), &expected);
    }

    /// Check command use
    #[test]
    fn with_commands() {
        let args = arg_list!(
            "-foo",     // Option from the main set
            "push",     // Our command, which has options of its own
            "-foo",     // Option unknown in command’s option set, and should **not** be matched
                        // against same option in main set.
            "-help",    // Option applicable to command (as well as main set, though the main set is
                        // irrelevant here).
            "-tags"     // Option applicable to command
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, Long, "foo"),
            ]),
            @itemset item_set!(cmd: "push",
                opt_set: cmdset_optset_ref!(get_base_cmds(), 3),
                error: false, warn: true,
            [
                expected_item!(2, UnknownLong, "foo"),
                expected_item!(3, Long, "help"),
                expected_item!(4, Long, "tags"),
            ]),
            cmd_set: Some(cmdset_subcmdset_ref!(get_base_cmds(), 3))
        );
        let mut parser = get_parser();
        parser.settings.set_mode(OptionsMode::Alternate);
        check_result(&Actual(parser.parse(&args)), &expected);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// “Posixly correct” parsing, i.e. no mixing of options and positionals, the first positionals
// causes all subsequent arguments to be treated as positionals.
////////////////////////////////////////////////////////////////////////////////////////////////////

mod posixly_correct {
    use super::*;

    #[test]
    fn basic() {
        let args = arg_list!(
            "--help",   // Option
            "abc",      // Unknown command
            "def",      // Positional
            "--foo",    // Option, to be taken as positional
            "--",       // Early terminator, to be taken as positional
            "bar",      // Option, to be taken as positional
        );
        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, Long, "help"),
                expected_item!(1, UnknownCommand, "abc"),
                expected_item!(2, Positional, "def"),
                expected_item!(3, Positional, "--foo"),
                expected_item!(4, Positional, "--"),
                expected_item!(5, Positional, "bar"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let mut parser = get_parser();
        parser.settings.set_posixly_correct(true);
        check_result(&Actual(parser.parse(&args)), &expected);
    }

    /// Check works with early terminator use, where it should not make any difference
    #[test]
    fn early_term_first() {
        let args = arg_list!(
            "--help",   // Option
            "--",       // Early terminator
            "abc",      // Positional
            "--foo",    // Option, to be taken as positional
        );
        let expected = expected!(
            error: false,
            warn: false,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: false,
            [
                expected_item!(0, Long, "help"),
                expected_item!(1, EarlyTerminator),
                expected_item!(2, Positional, "abc"),
                expected_item!(3, Positional, "--foo"),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        let mut parser = get_parser();
        parser.settings.set_posixly_correct(true);
        check_result(&Actual(parser.parse(&args)), &expected);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Invalid unicode sequences
//
// These test some example items using invalid UTF-8 byte sequences, where the invalid sequences
// will be changed to unicode replacement characters in the inner `str` based parsing engine, and we
// need to correctly map those to the original bytes for correct `OsStr` based parsing results.
//
// Note, we assume that available option/command names are always valid UTF-8, as we require.
////////////////////////////////////////////////////////////////////////////////////////////////////

mod invalid_byte_sequences {
    use super::*;

    /*
     * On Unix, UTF-8 is used for OS strings, though without proper enforcement of all of the rules
     * of valid encodings. In Rust, an `OsStr`/`OsString` uses a simple `u8` sequence to represent
     * an OS string, and a lossy conversion to `str` form (truly valid UTF-8) simply involves a full
     * validation check, replacing invalid sequences with a unicode replacement character (U+FFFD).
     *
     * On Windows, UTF-16 is used for OS Strings, though again, not enforcing all of the rules
     * (which are more simple than with UTF-8, simply forbidding unpaired surrogates). In Rust, an
     * `OSStr`/`OsString` uses WTF-8 (wobbly transformation format, 8-bit) to represent as OS
     * string. Storing an OS string in an `OsStr`/`OsString` simply involves a basic UTF-16 decoding
     * to code points, then re-encoding in UTF-8 form, storing in a WTF-8 based type, which does
     * lossy conversion to `str` form by simply replacing sequences that represent unpaired UTF-16
     * surrogates with the replacement character. The way the original UTF-16 is transformed into
     * WTF-8 ensures that it is always valid WTF-8.
     *
     * To create invalid strings for testing:
     *
     *  - On Unix, there are many single bytes that are invalid, for instance `0x80`, a continuation
     *    byte, and since the OS string byte sequence is simply stored in `OsStr/`OsString`, we can
     *    simply just store such a byte in one.
     *  - On Windows, a byte like `0x80` directly injected into an `OsStr`/`OsString`, e.g. via a
     *    transmute, creates an invalid WTF-8, which then transforms lossily into an invalid `str`
     *    form. As mentioned above, the conversion of an OS string into WTF-8 would not allow such
     *    a lone byte to end up in the WTF-8 string. We must use something else which is valid in
     *    WTF-8, which gets swapped with a replacement character in lossy `str` conversion, for
     *    instance a code point like `0xD800`, i.e. [ 0xed, 0xa0, 0x80 ].
     *
     * Hence the below test is different for Unix vs. Windows.
     *
     * Note, the only way to create invalid strings here is by specifying them as a byte sequence.
     * Rust does not allow creating literals with invalid bytes. (Naturally users of a program
     * using this library would not have any trouble providing strings containing invalid byte
     * sequences, for instance with a Unix command line, a user could simply enter an argument like
     * `--a$'\x80'b` to inject an invalid 0x80 continuation byte).
     */

    #[cfg(any(unix, target_os = "redox"))]
    #[test]
    fn unix() {
        use std::os::unix::ffi::OsStrExt;

        let args = [
            OsStr::from_bytes(b"a\x80bc"),       // Unknown command
            OsStr::from_bytes(b"--\x80xx"),      // Unknown long option
            OsStr::from_bytes(b"--hah=a\x80bc"), // Known long, with in-same-arg data
            OsStr::from_bytes(b"--hah"),         // Known long, with in-next-arg data
            OsStr::from_bytes(b"abc\x80"),       // The in-next-arg data
            OsStr::from_bytes(b"--foo=\x80xyz"), // Known long option, does not take data, unexpected data
            OsStr::from_bytes(b"--=xy\x80z"),    // Long with no name, and non-empty data
            OsStr::from_bytes(b"-m\x80h"),       // Short option set, unknown, invalid, known
            OsStr::from_bytes(b"-oar\x80g"),     // Known short option, with in-same-arg data
            OsStr::from_bytes(b"-o"),            // Known short option, with in-next-arg data
            OsStr::from_bytes(b"\x80arg"),       // The in-next-arg data
            // Multiple invalid short option set bytes, which, being entirely unrelated (not a
            // prematurely-terminated multi-byte sequence) should each result in a single unicode
            // replacement character.
            OsStr::from_bytes(b"-\x80\x81\x82"),
            // This uses the first three bytes of a four byte character, checking that these being
            // related actually results in only one single unicode replacement character, not three.
            OsStr::from_bytes(b"-\xf0\x9f\x92"),
            // This uses the first two bytes of a four byte character, checking that this does not
            // trip up the check for a real replacement character, which uses a slice of three
            // bytes, which could panic if done wrong.
            OsStr::from_bytes(b"-\xf0\x9f"),
            // This uses a mix of basic known/unknown simple characters, including multibyte; an
            // incomplete multi-byte char; a simple invalid single-byte; and an actual unicode
            // replacement char (U+FFFD). The purpose is partly to throw lots of stuff into the mix
            // together, and partly to check that a replacement character itself is handled
            // correctly. Note, a byte string is ASCII only, thus: `\xe2\x9d\xa4` is `❤`; `\xc5\x9f`
            // is `ş`; and `\xef\xbf\xbd` is the rep char.
            OsStr::from_bytes(b"-m\xe2\x9d\xa4a\xc5\x9f\xf0\x9f\x92j\x80\xef\xbf\xbdk"),
            // Here we have a situation of in-same-arg data, with invalid bytes within the
            // proceeding short option set characters, thus checking whether or not we correctly
            // extract the right byte slice for the data value. An invalid byte is also present in
            // the value for good measure.
            OsStr::from_bytes(b"-m\x80\x81\x82oar\xf0\x9f\x92g"),
            // And what happens if an actual unicode replacement character (u+FFFD) is given?
            OsStr::from_bytes(b"-m\xef\xbf\xbdoar\x84\x85g"),
            // Finally, let's assert that a combined sequence of invalid bytes, of both related and
            // unrelated, comes out as the number of unicode replacement characters we expect. The
            // following should be four.
            OsStr::from_bytes(b"-\x80\x81\xef\xbf\xbd\x82"),
        ];

        let expected_strings = [
            OsStr::from_bytes(b"a\x80bc"),
            OsStr::from_bytes(b"\x80xx"),
            OsStr::from_bytes(b"a\x80bc"),
            OsStr::from_bytes(b"abc\x80"),
            OsStr::from_bytes(b"\x80xyz"),
            OsStr::from_bytes(b"ar\x80g"),
            OsStr::from_bytes(b"\x80arg"),
            OsStr::from_bytes(b"ar\xf0\x9f\x92g"),
            OsStr::from_bytes(b"ar\x84\x85g"),
        ];

        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownCommand, expected_strings[0]),
                expected_item!(1, UnknownLong, expected_strings[1]),
                expected_item!(2, LongWithData, "hah", expected_strings[2], DataLocation::SameArg),
                expected_item!(3, LongWithData, "hah", expected_strings[3], DataLocation::NextArg),
                expected_item!(5, LongWithUnexpectedData, "foo", expected_strings[4]),
                expected_item!(6, LongWithNoName),
                expected_item!(7, UnknownShort, 'm'),
                // Note, here, it is right that we do not receive the original invalid byte(s) as
                // the unrecognised short option, since it would be a pain to determine exactly what
                // byte(s) were turned into each individual unicode replacement char that was
                // analysed by the inner `str` based parser, which would also potentially involve
                // merging some of its analysis items. Thus we expect a replacement char here.
                expected_item!(7, UnknownShort, '�'),
                expected_item!(7, Short, 'h'),
                expected_item!(8, ShortWithData, 'o', expected_strings[5], DataLocation::SameArg),
                expected_item!(9, ShortWithData, 'o', expected_strings[6], DataLocation::NextArg),
                expected_item!(11, UnknownShort, '�'), // Notice three individual instances for arg 11
                expected_item!(11, UnknownShort, '�'),
                expected_item!(11, UnknownShort, '�'),
                expected_item!(12, UnknownShort, '�'), // Note only one instance for arg 12
                expected_item!(13, UnknownShort, '�'), // Note only one instance for arg 13
                expected_item!(14, UnknownShort, 'm'),
                expected_item!(14, Short, '❤'),
                expected_item!(14, UnknownShort, 'a'),
                expected_item!(14, UnknownShort, 'ş'),
                expected_item!(14, UnknownShort, '�'), // This one is from the incomplete multi-byte
                expected_item!(14, UnknownShort, 'j'),
                expected_item!(14, UnknownShort, '�'), // This one is from the other invalid byte
                expected_item!(14, UnknownShort, '�'), // This one is from the actual U+FFFD char
                expected_item!(14, UnknownShort, 'k'),
                expected_item!(15, UnknownShort, 'm'),
                expected_item!(15, UnknownShort, '�'),
                expected_item!(15, UnknownShort, '�'),
                expected_item!(15, UnknownShort, '�'),
                expected_item!(15, ShortWithData, 'o', expected_strings[7], DataLocation::SameArg),
                expected_item!(16, UnknownShort, 'm'),
                expected_item!(16, UnknownShort, '�'),
                expected_item!(16, ShortWithData, 'o', expected_strings[8], DataLocation::SameArg),
                expected_item!(17, UnknownShort, '�'),
                expected_item!(17, UnknownShort, '�'),
                expected_item!(17, UnknownShort, '�'),
                expected_item!(17, UnknownShort, '�'),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    #[cfg(windows)]
    #[test]
    fn windows() {
        // Necessary hack due to lack of access to raw bytes on Windows currently
        pub trait OsStrExt {
            fn from_bytes(slice: &[u8]) -> &Self;
        }
        impl OsStrExt for OsStr {
            fn from_bytes(slice: &[u8]) -> &OsStr { unsafe { std::mem::transmute(slice) } }
        }

        let args = [
            OsStr::from_bytes(b"a\xed\xa0\x80bc"),       // Unknown command
            OsStr::from_bytes(b"--\xed\xa0\x80xx"),      // Unknown long option
            OsStr::from_bytes(b"--hah=a\xed\xa0\x80bc"), // Known long, with in-same-arg data
            OsStr::from_bytes(b"--hah"),                 // Known long, with in-next-arg data
            OsStr::from_bytes(b"abc\xed\xa0\x80"),       // The in-next-arg data
            OsStr::from_bytes(b"--foo=\xed\xa0\x80xyz"), // Known long option, does not take data, unexpected data
            OsStr::from_bytes(b"--=xy\xed\xa0\x80z"),    // Long with no name, and non-empty data
            OsStr::from_bytes(b"-m\xed\xa0\x80h"),       // Short option set, unknown, invalid, known
            OsStr::from_bytes(b"-oar\xed\xa0\x80g"),     // Known short option, with in-same-arg data
            OsStr::from_bytes(b"-o"),                    // Known short option, with in-next-arg data
            OsStr::from_bytes(b"\xed\xa0\x80arg"),       // The in-next-arg data
            // Multiple invalid short option set character byte sequences, which should each result
            // in a single unicode replacement character.
            OsStr::from_bytes(b"-\xed\xa0\x80\xed\xa0\x81\xed\xa0\x82"),
            // This uses a mix of basic known/unknown simple characters, including multibyte; an
            // an invalid sequence (unpaired surrogate); and an actual unicode replacement char
            // (U+FFFD). The purpose is partly to throw lots of stuff into the mix together, and
            // partly to check that a replacement character itself is handled correctly. Note, a
            // byte string is ASCII only, thus: `\xe2\x9d\xa4` is `❤`; `\xc5\x9f` is `ş`; and
            // `\xef\xbf\xbd` is the rep char.
            OsStr::from_bytes(b"-m\xe2\x9d\xa4a\xc5\x9fj\xed\xa0\x80\xef\xbf\xbdk"),
            // Here we have a situation of in-same-arg data, with invalid bytes within the
            // proceeding short option set characters, thus checking whether or not we correctly
            // extract the right byte slice for the data value. Invalid bytes are also present in
            // the value for good measure.
            OsStr::from_bytes(b"-m\xed\xa0\x80\xed\xa0\x81\xed\xa0\x82oar\xed\xa0\x83g"),
            // And what happens if an actual unicode replacement character (u+FFFD) is given?
            OsStr::from_bytes(b"-m\xef\xbf\xbdoar\xed\xa0\x84\xed\xa0\x85g"),
        ];

        let expected_strings = [
            OsStr::from_bytes(b"a\xed\xa0\x80bc"),
            OsStr::from_bytes(b"\xed\xa0\x80xx"),
            OsStr::from_bytes(b"a\xed\xa0\x80bc"),
            OsStr::from_bytes(b"abc\xed\xa0\x80"),
            OsStr::from_bytes(b"\xed\xa0\x80xyz"),
            OsStr::from_bytes(b"ar\xed\xa0\x80g"),
            OsStr::from_bytes(b"\xed\xa0\x80arg"),
            OsStr::from_bytes(b"ar\xed\xa0\x83g"),
            OsStr::from_bytes(b"ar\xed\xa0\x84\xed\xa0\x85g"),
        ];

        let expected = expected!(
            error: false,
            warn: true,
            @itemset item_set!(cmd: "", opt_set: get_base_opts(), error: false, warn: true,
            [
                expected_item!(0, UnknownCommand, expected_strings[0]),
                expected_item!(1, UnknownLong, expected_strings[1]),
                expected_item!(2, LongWithData, "hah", expected_strings[2], DataLocation::SameArg),
                expected_item!(3, LongWithData, "hah", expected_strings[3], DataLocation::NextArg),
                expected_item!(5, LongWithUnexpectedData, "foo", expected_strings[4]),
                expected_item!(6, LongWithNoName),
                expected_item!(7, UnknownShort, 'm'),
                expected_item!(7, UnknownShort, '�'),
                expected_item!(7, Short, 'h'),
                expected_item!(8, ShortWithData, 'o', expected_strings[5], DataLocation::SameArg),
                expected_item!(9, ShortWithData, 'o', expected_strings[6], DataLocation::NextArg),
                expected_item!(11, UnknownShort, '�'), // Notice three individual instances for arg 11
                expected_item!(11, UnknownShort, '�'),
                expected_item!(11, UnknownShort, '�'),
                expected_item!(12, UnknownShort, 'm'),
                expected_item!(12, Short, '❤'),
                expected_item!(12, UnknownShort, 'a'),
                expected_item!(12, UnknownShort, 'ş'),
                expected_item!(12, UnknownShort, 'j'),
                expected_item!(12, UnknownShort, '�'),
                expected_item!(12, UnknownShort, '�'), // This one is from the actual U+FFFD char
                expected_item!(12, UnknownShort, 'k'),
                expected_item!(13, UnknownShort, 'm'),
                expected_item!(13, UnknownShort, '�'),
                expected_item!(13, UnknownShort, '�'),
                expected_item!(13, UnknownShort, '�'),
                expected_item!(13, ShortWithData, 'o', expected_strings[7], DataLocation::SameArg),
                expected_item!(14, UnknownShort, 'm'),
                expected_item!(14, UnknownShort, '�'),
                expected_item!(14, ShortWithData, 'o', expected_strings[8], DataLocation::SameArg),
            ]),
            cmd_set: Some(get_base_cmds())
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }
}
