// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use gong::analysis::*;
use gong::parser::{Parser, OptionsMode};
use common::{get_parser, Actual, Expected, check_result};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Arg list string types
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Check arg parsing accepts `&[String]` and `&[&str]`
///
/// All that we really need concern ourselves with is that it compiles.
#[test]
fn arg_list_owned_set() {
    // Test works (compiles) using a `String` based slice (as given from `env::args()` for real args)
    // Note, **deliberately** not using the `arg_list` macro here!
    let args: Vec<String> = vec![ String::from("--foo"), String::from("--bah") ];
    let _ = get_parser().parse(&args);

    // Test works (compiles) using a `&str` based slice
    // Note, **deliberately** not using the `arg_list` macro here!
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
        "abc", "-", "z",    // Non-options
        "--help",           // Known long option
        "--xxx",            // Unknown long option
        "---yy",            // Extra dash should be taken as part of long option name
        "version",          // Known long option, but no prefix, thus non-option
        "-bxs",             // Short option set, two unknown, one known (`x`)
        "ghi",              // Non-option, containing known short option (`h`)
        "-a-",              // Dash in short opt set should come out as unknown short opt (can not
                            // be a known one as not allowed), so long as not the first in set, as
                            // would then arg would then be interpreted as long option or early
                            // terminator.
        "-h-",              // Same, but with known short opt in set, which should not matter.
        "--",               // Early terminator
        "--foo",            // Known option, taken as non-option due to early terminator
        "jkl",              // Non-option either way
    );
    let expected = expected!(
        error: false,
        warn: true,
        [
            expected_item!(0, NonOption, "abc"),
            expected_item!(1, NonOption, "-"),
            expected_item!(2, NonOption, "z"),
            expected_item!(3, Long, "help"),
            expected_item!(4, UnknownLong, "xxx"),
            expected_item!(5, UnknownLong, "-yy"),
            expected_item!(6, NonOption, "version"),
            expected_item!(7, UnknownShort, 'b'),
            expected_item!(7, Short, 'x'),
            expected_item!(7, UnknownShort, 's'),
            expected_item!(8, NonOption, "ghi"),
            expected_item!(9, UnknownShort, 'a'),
            expected_item!(9, UnknownShort, '-'),
            expected_item!(10, Short, 'h'),
            expected_item!(10, UnknownShort, '-'),
            expected_item!(11, EarlyTerminator),
            expected_item!(12, NonOption, "--foo"),
            expected_item!(13, NonOption, "jkl"),
        ]
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
        [
            expected_item!(0, UnknownLong, "Foo"),
            expected_item!(1, UnknownShort, 'O'),
        ]
    );
    check_result(&Actual(get_parser().parse(&args)), &expected);
}

/// Test that everything after an early terminator is taken to be a non-option, including any
/// further early terminators.
#[test]
fn early_term() {
    let args = arg_list!(
        "--foo",    // Before the early terminator, should work as option
        "--",       // Our early terminator
        "--help",   // Should be affected, thus non-option
        "--",       // Should be a non-option, **not** another early terminator
        // A mix of various other items, some of which might be interpreted differently if it were
        // not for the early terminator.
        "-o", "--foo", "blah", "--bb", "-h", "--hah", "--hah=", "--", "--hah=a", "-oa", "-b",
    );
    let expected = expected!(
        error: false,
        warn: false,
        [
            expected_item!(0, Long, "foo"),
            expected_item!(1, EarlyTerminator),
            expected_item!(2, NonOption, "--help"),
            expected_item!(3, NonOption, "--"),
            expected_item!(4, NonOption, "-o"),
            expected_item!(5, NonOption, "--foo"),
            expected_item!(6, NonOption, "blah"),
            expected_item!(7, NonOption, "--bb"),
            expected_item!(8, NonOption, "-h"),
            expected_item!(9, NonOption, "--hah"),
            expected_item!(10, NonOption, "--hah="),
            expected_item!(11, NonOption, "--"),
            expected_item!(12, NonOption, "--hah=a"),
            expected_item!(13, NonOption, "-oa"),
            expected_item!(14, NonOption, "-b"),
        ]
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
        [
            expected_item!(0, LongWithNoName),
            expected_item!(1, LongWithNoName),
        ]
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
        ]
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
    fn test1() {
        let args = arg_list!("🗻∈🌏", "-🗻∈🌏", "--🗻∈🌏", "--ƒoo", "-❤");
        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, NonOption, "🗻∈🌏"),
                expected_item!(1, UnknownShort, '🗻'),
                expected_item!(1, UnknownShort, '∈'),
                expected_item!(1, UnknownShort, '🌏'),
                expected_item!(2, UnknownLong, "🗻∈🌏"),
                expected_item!(3, UnknownLong, "ƒoo"),
                expected_item!(4, Short, '❤'), // '\u{2764}' black heart
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Some utf8 multi-byte char handling - chars with combinator chars (e.g. accent)
    #[test]
    fn test2() {
        let args = arg_list!("y̆", "-y̆", "--y̆", "ëéy̆", "-ëéy̆", "--ëéy̆", "--ábc", "--ábc");
        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, NonOption, "y̆"),
                expected_item!(1, UnknownShort, 'y'),        // `y`
                expected_item!(1, UnknownShort, '\u{0306}'), // breve
                expected_item!(2, UnknownLong, "y̆"),
                expected_item!(3, NonOption, "ëéy̆"),
                expected_item!(4, UnknownShort, 'ë'),        // e+diaeresis
                expected_item!(4, UnknownShort, 'e'),        // `e`
                expected_item!(4, UnknownShort, '\u{0301}'), // acute accent
                expected_item!(4, UnknownShort, 'y'),        // `y`
                expected_item!(4, UnknownShort, '\u{0306}'), // breve
                expected_item!(5, UnknownLong, "ëéy̆"),
                expected_item!(6, UnknownLong, "ábc"),       // without combinator
                expected_item!(7, Long, "ábc"),              // with combinator
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Some utf8 multi-byte char width handling - chars with variation selector
    ///
    /// Here we use the "heavy black heart" char with variation selector #16 (emoji).
    #[test]
    fn test3() {
        // Note: the following is the “black heart” character, followed by the variation selector
        // #16 (emoji) character.
        let args = arg_list!("❤️", "-❤️", "--❤️");
        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, NonOption, "❤️"),
                expected_item!(1, Short, '\u{2764}'),        // black-heart
                expected_item!(1, UnknownShort, '\u{fe0f}'), // emoji selector
                expected_item!(2, UnknownLong, "❤️"),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Some utf8 multi-byte char width handling - lone combinator chars
    #[test]
    fn test4() {
        let args = arg_list!("\u{0306}", "-\u{0306}", "--\u{0306}", "-\u{030a}");
        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, NonOption, "\u{0306}"),
                expected_item!(1, UnknownShort, '\u{0306}'),
                expected_item!(2, UnknownLong, "\u{0306}"),
                expected_item!(3, Short, '\u{030a}'),
            ]
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
            [
                expected_item!(0, AmbiguousLong, "f"),
                expected_item!(1, AmbiguousLong, "fo"),
            ]
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
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Long, "foobar"),
                expected_item!(2, Long, "foobar"),
                expected_item!(3, Long, "foobar"),
            ]
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
            [
                expected_item!(0, UnknownLong, "f"),
                expected_item!(1, UnknownLong, "fo"),
                expected_item!(2, Long, "foo"),
                expected_item!(3, UnknownLong, "foob"),
                expected_item!(4, UnknownLong, "fooba"),
                expected_item!(5, Long, "foobar"),
            ]
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
        let args = arg_list!("--foo");
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, Long, "foo"),
            ]
        );
        let opts = gong_option_set_fixed!(
            [
                // Multiple options that “foo” will match as an abbreviation for before getting to
                // the exact match.
                gong_longopt!("fooo"),
                gong_longopt!("foooo"),
                gong_longopt!("fooooo"),
                gong_longopt!("foo"),    // Exact match for input `--foo`
            ],
            []
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
            [
                expected_item!(0, LongWithData, "hah", "def", DataLocation::NextArg),
                expected_item!(2, Long, "help"),
                expected_item!(3, LongWithData, "hah", "def", DataLocation::SameArg),
                expected_item!(4, Long, "help"),
            ]
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
            warn: false,
            [
                expected_item!(0, ShortWithData, 'o', "a", DataLocation::SameArg),
                expected_item!(1, NonOption, "g"),
            ]
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
            ]
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
            ]
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
            [
                expected_item!(0, LongMissingData, "hah"),
            ]
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
            [
                expected_item!(0, UnknownShort, 'b'),
                expected_item!(0, Short, 'x'),
                expected_item!(0, UnknownShort, 's'),
                expected_item!(0, ShortMissingData, 'o'),
            ]
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
            ]
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
            [
                expected_item!(0, LongWithData, "hah", "a", DataLocation::SameArg),
                expected_item!(1, ShortWithData, 'o', "s", DataLocation::NextArg),
                expected_item!(3, LongWithData, "hah", "b", DataLocation::SameArg),
                expected_item!(4, ShortWithData, 'o', "bc", DataLocation::SameArg),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Test option with expected data arg, declared to be in same argument, but empty
    #[test]
    fn same_arg_empty() {
        let args = arg_list!(
            "--hah=",   // Known option, takes data, not given, should be empty string
            "--help",   // Random known option, should not be take as data for previous
            "--hah=",   // Same again...
            "help",     // Non-option this time, also should not be taken as data
        );
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, LongWithData, "hah", "", DataLocation::SameArg),
                expected_item!(1, Long, "help"),
                expected_item!(2, LongWithData, "hah", "", DataLocation::SameArg),
                expected_item!(3, NonOption, "help"),
            ]
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
            ]
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
            ]
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
            [
                expected_item!(0, LongWithData, "hah", "--", DataLocation::SameArg),
                expected_item!(1, LongWithData, "hah", "--", DataLocation::NextArg),
                expected_item!(3, ShortWithData, 'o', "--", DataLocation::NextArg),
                expected_item!(5, ShortWithData, 'o', "--", DataLocation::SameArg),
            ]
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
            [
                expected_item!(0, LongWithData, "ƒƒ", "abc", DataLocation::NextArg),
                expected_item!(2, LongWithData, "ƒƒ", "", DataLocation::SameArg),
                expected_item!(3, LongWithData, "ƒƒ", "abc", DataLocation::SameArg),
                expected_item!(4, LongWithData, "ƒƒ", "❤️", DataLocation::SameArg),
                expected_item!(5, LongMissingData, "ƒƒ"),
            ]
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
            ]
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
            ]
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
            [
                expected_item!(0, Command, "commit"),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// We do not support case-insensitive matching
    #[test]
    fn case_sensitivity() {
        let args = arg_list!("Commit");
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, NonOption, "Commit"),
            ]
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
            [
                expected_item!(0, Command, "commit"),
                expected_item!(1, NonOption, "commit"),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check repeated use correctly fails, where different commands are used
    #[test]
    fn repeated_different() {
        let args = arg_list!("push", "commit");
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, Command, "push"),
                expected_item!(1, NonOption, "commit"),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check early terminator forces command to be treated as non-option
    #[test]
    fn after_early_term() {
        let args = arg_list!("--", "commit");
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, EarlyTerminator),
                expected_item!(1, NonOption, "commit"),
            ]
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
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Short, 'h'),
                expected_item!(2, Command, "commit"),
            ]
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
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Command, "foo"),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check names that look like options, take preference as options
    ///
    /// Command names simply should not be prefixed like this.
    #[test]
    fn name_like_option() {
        let opts = gong_option_set_fixed!(
            [
                gong_longopt!("foo"),
            ], []
        );
        let cmds = gong_command_set_fixed!([
            gong_command!("--foo"),
            gong_command!("--bar"),
        ]);

        let args = arg_list!("--foo", "--bar");
        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, UnknownLong, "bar"),
            ]
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
            [
                expected_item!(0, LongWithData, "hah", "commit", DataLocation::NextArg),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check not recognised as command in secondary non-option position, and first non-option is
    /// reported as a non-option, not as an unrecognised command (n/a to current design).
    #[test]
    fn following_unknown() {
        let args = arg_list!(
            "--hah", "foo",     // Option taking data
            "blah",             // Non-option, not matching any command
            "commit"            // A command, but one or more non-options already given
        );
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, LongWithData, "hah", "foo", DataLocation::NextArg),
                expected_item!(2, NonOption, "blah"),
                expected_item!(3, NonOption, "commit"),
            ]
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
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Short, 'h'),
                expected_item!(2, Command, "commit"),
                expected_item!(3, UnknownLong, "foo"),
                expected_item!(4, UnknownShort, 'o'),
                expected_item!(4, UnknownShort, 'q'),
            ]
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
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Short, 'h'),
                expected_item!(2, Command, "push"),
                expected_item!(3, UnknownLong, "foo"),
                expected_item!(4, Long, "help"),
                expected_item!(5, Long, "tags"),
                expected_item!(6, UnknownShort, 'o'),
            ]
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
            "blah"              // Non-option
        );
        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, Command, "branch"),
                expected_item!(1, UnknownLong, "foo"),
                expected_item!(2, Long, "help"),
                expected_item!(3, Long, "sorted"),
                expected_item!(4, UnknownLong, "show-current"),
                expected_item!(5, Command, "list"),
                expected_item!(6, Long, "foo"),
                expected_item!(7, Long, "help"),
                expected_item!(8, UnknownLong, "sorted"),
                expected_item!(9, Long, "show-current"),
                expected_item!(10, Command, "remote"),
                expected_item!(11, UnknownLong, "foo"),
                expected_item!(12, UnknownLong, "help"),
                expected_item!(13, UnknownLong, "nope"),
                expected_item!(14, UnknownLong, "show-current"),
                expected_item!(15, NonOption, "blah"),
            ]
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
            "--show-current",   // Everything here onwards should be taken as non-options
            "remotely",         // Sub-command (level 2)
            "--foo",            // Option unknown to sub-command
            "blah"              // Non-option
        );
        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, Command, "branch"),
                expected_item!(1, UnknownLong, "foo"),
                expected_item!(2, Long, "help"),
                expected_item!(3, Long, "sorted"),
                expected_item!(4, UnknownLong, "show-current"),
                expected_item!(5, Command, "del"),
                expected_item!(6, UnknownLong, "foo"),
                expected_item!(7, EarlyTerminator),
                expected_item!(8, NonOption, "--show-current"),
                expected_item!(9, NonOption, "remotely"),
                expected_item!(10, NonOption, "--foo"),
                expected_item!(11, NonOption, "blah"),
            ]
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
            warn: false,
            [
                expected_item!(0, Command, "branch"),
                expected_item!(1, Command, "del"),
                expected_item!(2, NonOption, "list"),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    /// Check known sub-command, given after a non-option that is not a known sub-command
    #[test]
    fn nested_following_nonoption() {
        let args = arg_list!(
            "branch",   // Primary command
            "blah",     // Non-option
            "list",     // Sub-command, but follows non-option, so not taken as a command
        );
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, Command, "branch"),
                expected_item!(1, NonOption, "blah"),
                expected_item!(2, NonOption, "list"),
            ]
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
            "abc",          // Non-opt
            "-",            // Should be non-opt
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
            "-help",        // Known option, should be non-opt though due to early terminator
        );
        let expected = expected!(
            error: true,
            warn: true,
            [
                expected_item!(0, NonOption, "abc"),
                expected_item!(1, NonOption, "-"),
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
                expected_item!(16, NonOption, "-help"),
            ]
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
            [
                expected_item!(0, LongWithUnexpectedData, "foo", "abc"),
                expected_item!(1, Long, "foo"),
                expected_item!(2, LongMissingData, "hah"),
            ]
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
            [
                expected_item!(0, LongWithData, "hah", "--", DataLocation::SameArg),
                expected_item!(1, LongWithData, "hah", "--", DataLocation::NextArg),
            ]
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
            [
                expected_item!(0, Long, "foo"),
                expected_item!(1, Command, "push"),
                expected_item!(2, UnknownLong, "foo"),
                expected_item!(3, Long, "help"),
                expected_item!(4, Long, "tags"),
            ]
        );
        let mut parser = get_parser();
        parser.settings.set_mode(OptionsMode::Alternate);
        check_result(&Actual(parser.parse(&args)), &expected);
    }
}
