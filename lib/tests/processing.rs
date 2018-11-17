// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
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
use gong::options::*;
use common::{get_base, Actual, Expected, check_result};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Arg list string types
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Check arg processing accepts `&[String]` and `&[&str]`
///
/// All that we really need concern ourselves with is that it compiles.
#[test]
fn arg_list_owned_set() {
    // Test works (compiles) using a `String` based slice (as given from `env::args()` for real args)
    // Note, **deliberately** not using the `arg_list` macro here!
    let args: Vec<String> = vec![ String::from("--foo"), String::from("--bah") ];
    let _ = gong::process(&args, &get_base());

    // Test works (compiles) using a `&str` based slice
    // Note, **deliberately** not using the `arg_list` macro here!
    let args: Vec<&str> = vec![ "--foo", "--bah" ];
    let _ = gong::process(&args, &get_base());
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Basic option handling
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Some general, basic argument handling
#[test]
fn basic() {
    let args = arg_list!(
        "abc", "-", "z",    // Non-options
        "--help",           // Real long option
        "--xxx",            // Non-real long option
        "---yy",            // Extra dash should be taken as part of long option name
        "version",          // Real long option, but no prefix, thus non-option
        "-bxs",             // Short option set, two non-real, one real ('x')
        "ghi",              // Non-option, containing real short option ('h')
        "-a-",              // Dash in short opt set should come out as unknown short opt (can not
                            // be a real one as not allowed), so long as not the first in set, as
                            // would then arg would then be interpreted as long option or early
                            // terminator.
        "-h-",              // Same, but with real short opt in set, which should not matter.
        "--",               // Early terminator
        "--foo",            // Real option, taken as non-option due to early terminator
        "jkl",              // Non-option either way
    );
    let expected = expected!(
        error: false,
        warn: true,
        vec![
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
    check_result(&Actual(gong::process(&args, &get_base())), &expected);
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
        vec![
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
    check_result(&Actual(gong::process(&args, &get_base())), &expected);
}

/// Test empty long option names with data param (-- on it's own is obviously picked up as early
/// terminator, but what happens when an '=' is added?).
#[test]
fn long_no_name() {
    let args = arg_list!("--=a", "--=");
    let expected = expected!(
        error: false,
        warn: true,
        vec![
            expected_item!(0, LongWithNoName),
            expected_item!(1, LongWithNoName),
        ]
    );
    check_result(&Actual(gong::process(&args, &get_base())), &expected);
}

/// Test repetition - each instance should exist in the results in its own right. Note, data arg
/// tests are done later in the data arg section.
#[test]
fn repetition() {
    let args = arg_list!("--foo", "-h", "--version", "-h", "-x", "--blah", "--version", "-hhh");
    let expected = expected!(
        error: false,
        warn: true,
        vec![
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
    check_result(&Actual(gong::process(&args, &get_base())), &expected);
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
            vec![
                expected_item!(0, NonOption, "🗻∈🌏"),
                expected_item!(1, UnknownShort, '🗻'),
                expected_item!(1, UnknownShort, '∈'),
                expected_item!(1, UnknownShort, '🌏'),
                expected_item!(2, UnknownLong, "🗻∈🌏"),
                expected_item!(3, UnknownLong, "ƒoo"),
                expected_item!(4, Short, '❤'), // '\u{2764}' black heart
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Some utf8 multi-byte char handling - chars with combinator chars (e.g. accent)
    #[test]
    fn test2() {
        let args = arg_list!("y̆", "-y̆", "--y̆", "ëéy̆", "-ëéy̆", "--ëéy̆", "--ábc", "--ábc");
        let expected = expected!(
            error: false,
            warn: true,
            vec![
                expected_item!(0, NonOption, "y̆"),
                expected_item!(1, UnknownShort, 'y'),        // 'y'
                expected_item!(1, UnknownShort, '\u{0306}'), // breve
                expected_item!(2, UnknownLong, "y̆"),
                expected_item!(3, NonOption, "ëéy̆"),
                expected_item!(4, UnknownShort, 'ë'),        // e+diaeresis
                expected_item!(4, UnknownShort, 'e'),        // 'e'
                expected_item!(4, UnknownShort, '\u{0301}'), // acute accent
                expected_item!(4, UnknownShort, 'y'),        // 'y'
                expected_item!(4, UnknownShort, '\u{0306}'), // breve
                expected_item!(5, UnknownLong, "ëéy̆"),
                expected_item!(6, UnknownLong, "ábc"),       // without combinator
                expected_item!(7, Long, "ábc"),              // with combinator
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Some utf8 multi-byte char width handling - chars with variation selector
    ///
    /// Here we use the "heavy black heart" char with variation selector #16 (emoji).
    #[test]
    fn test3() {
        // Note: the following is the 'black heart' character, followed by the variation selector
        // #16 (emoji) character.
        let args = arg_list!("❤️", "-❤️", "--❤️");
        let expected = expected!(
            error: false,
            warn: true,
            vec![
                expected_item!(0, NonOption, "❤️"),
                expected_item!(1, Short, '\u{2764}'),        // black-heart
                expected_item!(1, UnknownShort, '\u{fe0f}'), // emoji selector
                expected_item!(2, UnknownLong, "❤️"),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Some utf8 multi-byte char width handling - lone combinator chars
    #[test]
    fn test4() {
        let args = arg_list!("\u{0306}", "-\u{0306}", "--\u{0306}", "-\u{030a}");
        let expected = expected!(
            error: false,
            warn: true,
            vec![
                expected_item!(0, NonOption, "\u{0306}"),
                expected_item!(1, UnknownShort, '\u{0306}'),
                expected_item!(2, UnknownLong, "\u{0306}"),
                expected_item!(3, Short, '\u{030a}'),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
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
            vec![
                expected_item!(0, AmbiguousLong, "f"),
                expected_item!(1, AmbiguousLong, "fo"),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
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
            vec![
                expected_item!(0, Long, "foo"),
                expected_item!(1, Long, "foobar"),
                expected_item!(2, Long, "foobar"),
                expected_item!(3, Long, "foobar"),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test handling when abbreviated matching is disabled
    #[test]
    fn disabled() {
        let args = arg_list!("--f", "--fo", "--foo", "--foob", "--fooba", "--foobar");
        let expected = expected!(
            error: false,
            warn: true,
            vec![
                expected_item!(0, UnknownLong, "f"),
                expected_item!(1, UnknownLong, "fo"),
                expected_item!(2, Long, "foo"),
                expected_item!(3, UnknownLong, "foob"),
                expected_item!(4, UnknownLong, "fooba"),
                expected_item!(5, Long, "foobar"),
            ]
        );
        let mut opts = get_base();
        opts.set_allow_abbreviations(false);
        check_result(&Actual(gong::process(&args, &opts)), &expected);
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
            vec![
                expected_item!(0, Long, "foo"),
            ]
        );
        let opts = gong_option_set!(
            vec![
                // Multiple options that 'foo' will match as an abbreviation for before getting to
                // the exact match.
                gong_longopt!("fooo"),
                gong_longopt!("foooo"),
                gong_longopt!("fooooo"),
                gong_longopt!("foo"),    // Exact match for input `--foo`
            ],
            vec![]
        );
        check_result(&Actual(gong::process(&args, &opts)), &expected);
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
            vec![
                expected_item!(0, LongWithData, "hah", "def", DataLocation::NextArg),
                expected_item!(2, Long, "help"),
                expected_item!(3, LongWithData, "hah", "def", DataLocation::SameArg),
                expected_item!(4, Long, "help"),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
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
            vec![
                expected_item!(0, ShortWithData, 'o', "a", DataLocation::SameArg),
                expected_item!(1, NonOption, "g"),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test option with expected data arg, provided in next argument for short options
    #[test]
    fn arg_placement_short_next() {
        let args = arg_list!(
            // Here 'o' is valid and takes data, 'x' is valid and does not take data.
            "-o", "def",    // Simple in-next-arg
            "-bo", "def",   // Char(s) before 'o' should be correctly captured
            "-bxo", "def",  // Even chars that are valid short opts
            "-xao", "def",  // Different variation
        );
        let expected = expected!(
            error: false,
            warn: true,
            vec![
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
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test option with expected data arg, provided in same argument for short options
    #[test]
    fn arg_placement_short_same() {
        let args = arg_list!(
            "-oa",         // 'o' here takes data; trying here various combinations of
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
            vec![
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
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test missing argument data for long option
    #[test]
    fn missing_long() {
        let args = arg_list!("--hah");
        let expected = expected!(
            error: true,
            warn: false,
            vec![
                expected_item!(0, LongMissingData, "hah"),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test missing argument data for short option
    #[test]
    fn missing_short() {
        let args = arg_list!("-bxso");
        let expected = expected!(
            error: true,
            warn: true,
            vec![
                expected_item!(0, UnknownShort, 'b'),
                expected_item!(0, Short, 'x'),
                expected_item!(0, UnknownShort, 's'),
                expected_item!(0, ShortMissingData, 'o'),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test some misc. data handling.
    ///
    /// Unrecognised option with data; unrecognised with empty data; recognised with unexpected
    /// data; recognised with empty unexpected data; and that long option "component" splitting
    /// based on the first equals character (`=`) has no effect on short option set processing.
    #[test]
    fn misc() {
        let args = arg_list!(
            "--xx=yy",   // Unrecognised long option, with data, name component is "xx"
            "--tt=",     // Unrecognised long option, with data, but data is empty string
            "-x",        // Random
            "--foo=bar", // Real long option, but does **not** take data, thus unexpected
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
            vec![
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
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test repetition - each instance should exist in the results in its own right. Note, basic
    /// non-data-arg related tests done separately already.
    #[test]
    fn repetition() {
        let args = arg_list!("--hah=a", "-o", "s", "--hah=b", "-obc");
        let expected = expected!(
            error: false,
            warn: false,
            vec![
                expected_item!(0, LongWithData, "hah", "a", DataLocation::SameArg),
                expected_item!(1, ShortWithData, 'o', "s", DataLocation::NextArg),
                expected_item!(3, LongWithData, "hah", "b", DataLocation::SameArg),
                expected_item!(4, ShortWithData, 'o', "bc", DataLocation::SameArg),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test option with expected data arg, declared to be in same argument, but empty
    #[test]
    fn same_arg_empty() {
        let args = arg_list!(
            "--hah=",   // Real option, takes data, not given, should be empty string
            "--help",   // Random real option, should not be take as data for previous
            "--hah=",   // Same again...
            "help",     // Non-option this time, also should not be taken as data
        );
        let expected = expected!(
            error: false,
            warn: false,
            vec![
                expected_item!(0, LongWithData, "hah", "", DataLocation::SameArg),
                expected_item!(1, Long, "help"),
                expected_item!(2, LongWithData, "hah", "", DataLocation::SameArg),
                expected_item!(3, NonOption, "help"),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test option with expected data arg, with data containing '='. An '=' in a long option arg
    /// denotes an "in-arg" data value and thus it is broken up into name and data components. Here
    /// we check that an '=' does not result in unwanted behaviour in order positions.
    #[test]
    fn containing_equals() {
        let args = arg_list!(
            "--hah", "d=ef",    // Should just be treated as part of the data
            "--hah", "=",       // Should just be treated as data
            "--hah=d=ef",       // First '=' separates name and data, other is just part of the data
            "--hah==ef",        // Same here
            "--help",           // Random
            "--blah=ggg",       // Long option, but not a matching one, data should be ignored
            "-oa=b",            // Short option, should be part of 'o' option's data
            "-o=",              // Same
            "-o===o",           // Same
        );
        let expected = expected!(
            error: false,
            warn: true,
            vec![
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
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test argument data that looks like options
    #[test]
    fn looking_like_options() {
        let args = arg_list!(
            "--hah=--foo", "--hah", "--foo",   // With real long option, in-arg/next-arg
            "--hah=--blah", "--hah", "--blah", // Not real
            "--hah=-h", "--hah", "-h",         // With real short option
            "--hah=-n", "--hah", "-n",         // Not real
            "-o-h", "-o", "-h",                // Using short-opt, with real short opt
            "-o-n", "-o", "-n",                // Same, but not real
            "-o--foo",                         // Short using real long lookalike
            "-o", "--hah",                     // Same, but long that take data
            "-o--blah", "-o", "--blah",        // With not real
        );
        let expected = expected!(
            error: false,
            warn: false,
            vec![
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
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test argument data that looks like early terminator
    #[test]
    fn looking_like_early_term() {
        let args = arg_list!(
            "--hah=--",     // In long option's data, in-arg
            "--hah", "--",  // Same, next-arg
            "-o", "--",     // In short option's data, in-arg
            "-o--",         // Same, next-arg
        );
        let expected = expected!(
            error: false,
            warn: false,
            vec![
                expected_item!(0, LongWithData, "hah", "--", DataLocation::SameArg),
                expected_item!(1, LongWithData, "hah", "--", DataLocation::NextArg),
                expected_item!(3, ShortWithData, 'o', "--", DataLocation::NextArg),
                expected_item!(5, ShortWithData, 'o', "--", DataLocation::SameArg),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test long option involving multi-byte chars, to ensure "in-arg" component splitting for
    /// instance.
    #[test]
    fn multibyte_long() {
        let args = arg_list!("--ƒƒ", "abc", "--ƒƒ=", "--ƒƒ=abc", "--ƒƒ=❤️", "--ƒƒ");
        let expected = expected!(
            error: true,
            warn: false,
            vec![
                expected_item!(0, LongWithData, "ƒƒ", "abc", DataLocation::NextArg),
                expected_item!(2, LongWithData, "ƒƒ", "", DataLocation::SameArg),
                expected_item!(3, LongWithData, "ƒƒ", "abc", DataLocation::SameArg),
                expected_item!(4, LongWithData, "ƒƒ", "❤️", DataLocation::SameArg),
                expected_item!(5, LongMissingData, "ƒƒ"),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
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
            vec![
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
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test the effect of Utf-8 combinator characters - does this break char iteration or byte
    /// position calculation whilst processing a short option set. Safe to assume it won't, but
    /// may as well throw down a few samples.
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
            vec![
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
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Alt-mode option processing
////////////////////////////////////////////////////////////////////////////////////////////////////

mod alt_mode {
    use super::*;

    /* Some tests for alt-mode option processing (no short options, long options use single dash
     * prefix. There is little difference in processing, so few tests should be required. */

    /// Check a range of inputs
    #[test]
    fn basic() {
        let args = arg_list!(
            "abc",          // Non-opt
            "-",            // Should be non-opt
            "-help",        // Real option, via alt-mode single dash
            "-hah=abc",     // Data-taking variant, in-arg
            "-hah", "cba",  // Same, next-arg
            "-hah=",        // Same, in-arg, data is empty string
            "-=",           // Option with data arg, which is empty, also empty name
            "-=abc",        // Similar, empty name, data provided though, which should be ignored
            "-bxs",         // 'x' is a real short opt, but they should be ignored
            "--foo",        // Real option, 'standard' mode syntax, the second dash should be taken
                            // as being a part of the name.
            "-f",           // Ambiguous long option, matches both `foo` and `foobar`
            "-foo",         // Matches both `foo` and `foobar`, but matches `foo` exactly
            "-foob",        // Unique abbreviation to `foobar`
            "-❤",           // Check real short opt not taken as such
            "--",           // Early term
            "-help",        // Real option, should be non-opt though due to early terminator
        );
        let expected = expected!(
            error: true,
            warn: true,
            vec![
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
        let mut opts = get_base();
        opts.set_mode(OptionsMode::Alternate);
        check_result(&Actual(gong::process(&args, &opts)), &expected);
    }

    /// Check unexpected and missing data
    #[test]
    fn data_basic() {
        let args = arg_list!(
            "-foo=abc", // Real option, takes no data
            "-foo=",    // Same, data is empty though so should just be ignored
            "-hah",     // Real option, takes data, none provided
        );
        let expected = expected!(
            error: true,
            warn: true,
            vec![
                expected_item!(0, LongWithUnexpectedData, "foo", "abc"),
                expected_item!(1, Long, "foo"),
                expected_item!(2, LongMissingData, "hah"),
            ]
        );
        let mut opts = get_base();
        opts.set_mode(OptionsMode::Alternate);
        check_result(&Actual(gong::process(&args, &opts)), &expected);
    }

    /// Test argument data that looks like early terminator
    #[test]
    fn data_looking_like_early_term() {
        let args = arg_list!(
            "-hah=--",      // Real option, takes data, in-arg
            "-hah", "--",   // Same, next-arg
        );
        let expected = expected!(
            error: false,
            warn: false,
            vec![
                expected_item!(0, LongWithData, "hah", "--", DataLocation::SameArg),
                expected_item!(1, LongWithData, "hah", "--", DataLocation::NextArg),
            ]
        );
        let mut opts = get_base();
        opts.set_mode(OptionsMode::Alternate);
        check_result(&Actual(gong::process(&args, &opts)), &expected);
    }
}
