// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use gong::*;
use common::{get_base, Actual, Expected, check_result};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Basic option handling
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Some general, basic argument handling
#[test]
fn basic() {
    let args = arg_list!("abc", "-", "z", "--help", "--xxx", "---yy", "version", "-bxs", "ghi",
        "--", "--foo", "jkl");
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
            expected_item!(9, EarlyTerminator),
            expected_item!(10, NonOption, "--foo"),
            expected_item!(11, NonOption, "jkl"),
        ]
    );
    check_result(&Actual(gong::process(&args, &get_base())), &expected);
}

/// Test that everything after an early terminator is taken to be a non-option, inclucing any
/// further early terminators.
#[test]
fn early_term() {
    let args = arg_list!("--foo", "--", "--help", "--", "-o", "--foo", "blah", "--bb", "-h",
        "--hah", "--hah=", "--", "--hah=a", "-oa", "-b");
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
                expected_item!(4, UnknownShort, '\u{0301}'), // accute accent
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
    fn ambigous() {
        let args = arg_list!("--f");
        let expected = expected!(
            error: true,
            warn: false,
            vec![
                expected_item!(0, AmbiguousLong, "f"),
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test handling of abbreviated long options, without ambiguity
    #[test]
    fn unambigous() {
        let args = arg_list!("--foo", "--foob");
        let expected = expected!(
            error: false,
            warn: false,
            vec![
                expected_item!(0, Long, "foo"),
                expected_item!(1, Long, "foobar"),
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Data sub-arguments
////////////////////////////////////////////////////////////////////////////////////////////////////

mod data {
    use super::*;

    /// Test option with expected data arg for long options
    #[test]
    fn arg_placement_long() {
        let args = arg_list!("--hah", "def", "--help", "--hah=def", "--help");
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
        let args = arg_list!("-o", "def", "-bo", "def", "-bxo", "def", "-xao", "def");
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
        let args = arg_list!("-oa", "-oabc", "-aob", "-aobcd", "-abcod", "-abcodef", "-xoabc",
            "-oaxc", "-oxbc", "-oabx");
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
    /// data; and recognised with empty unexpected data.
    #[test]
    fn misc() {
        let args = arg_list!("--xx=yy", "--tt=", "-x", "--foo=bar", "--foo=", "-x");
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
            ]
        );
        check_result(&Actual(gong::process(&args, &get_base())), &expected);
    }

    /// Test option with expected data arg, declared to be in same argument, but empty
    #[test]
    fn same_arg_empty() {
        let args = arg_list!("--hah=", "--help", "--hah=", "help");
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

    /// Test option with expected data arg, with data containing '='
    #[test]
    fn containing_equals() {
        let args = arg_list!("--hah", "d=ef", "--hah", "=", "--hah=d=ef", "--hah==ef", "--help",
            "--blah=ggg", "-oa=b", "-o=", "-o===o");
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
        let args = arg_list!("--hah=--foo", "--hah", "--foo", "--hah=--blah", "--hah", "--blah",
            "--hah=-h", "--hah", "-h", "--hah=-n", "--hah", "-n", "-o-h", "-o", "-h", "-o-n", "-o",
            "-n", "-o--foo", "-o", "--hah", "-o--blah", "-o", "--blah");
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
        let args = arg_list!("--hah=--", "--hah", "--", "-o", "--", "-o--");
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
        let args = arg_list!("abc", "-", "-help", "-hah=abc", "-hah", "cba", "-hah=", "-=", "-=abc",
            "-bxs", "--foo", "-f", "-foo", "-foob", "--", "-help");
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
                expected_item!(14, EarlyTerminator),
                expected_item!(15, NonOption, "-help"),
            ]
        );
        let mut opts = get_base();
        opts.set_mode(OptionsMode::Alternate);
        check_result(&Actual(gong::process(&args, &opts)), &expected);
    }

    /// Check unexpected and missing data
    #[test]
    fn data_basic() {
        let args = arg_list!("-foo=abc", "-foo=", "-hah");
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
        let args = arg_list!("-hah=--", "-hah", "--");
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
