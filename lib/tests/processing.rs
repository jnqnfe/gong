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

#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use gong::*;
use common::get_base;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Basic option handling
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Some general, basic argument handling
#[test]
fn basic() {
    let opts = get_base();
    let args = arg_list!("abc", "-", "z", "--help", "--xxx", "---yy", "version", "-bxs", "ghi",
        "--", "--foo", "jkl");
    let results = gong::process(&args, &opts);
    assert_eq!(results,
        Results {
            error: false,
            warn: true,
            items: vec![
                ItemClass::Ok(Item::NonOption(0, "abc")),
                ItemClass::Ok(Item::NonOption(1, "-")),
                ItemClass::Ok(Item::NonOption(2, "z")),
                ItemClass::Ok(Item::Long(3, "help")),
                ItemClass::Warn(ItemW::UnknownLong(4, "xxx")),
                ItemClass::Warn(ItemW::UnknownLong(5, "-yy")),
                ItemClass::Ok(Item::NonOption(6, "version")),
                ItemClass::Warn(ItemW::UnknownShort(7, 'b')),
                ItemClass::Ok(Item::Short(7, 'x')),
                ItemClass::Warn(ItemW::UnknownShort(7, 's')),
                ItemClass::Ok(Item::NonOption(8, "ghi")),
                ItemClass::Ok(Item::EarlyTerminator(9)),
                ItemClass::Ok(Item::NonOption(10, "--foo")),
                ItemClass::Ok(Item::NonOption(11, "jkl")),
            ],
        }
    );
}

/// Test that everything after an early terminator is taken to be a non-option, inclucing any
/// further early terminators.
#[test]
fn early_term() {
    let opts = get_base();
    let args = arg_list!("--foo", "--", "--help", "--", "-o", "--foo", "blah", "--bb", "-h",
        "--hah", "--hah=", "--", "--hah=a", "-oa", "-b");
    let results = gong::process(&args, &opts);
    assert_eq!(results,
        Results {
            error: false,
            warn: false,
            items: vec![
                ItemClass::Ok(Item::Long(0, "foo")),
                ItemClass::Ok(Item::EarlyTerminator(1)),
                ItemClass::Ok(Item::NonOption(2, "--help")),
                ItemClass::Ok(Item::NonOption(3, "--")),
                ItemClass::Ok(Item::NonOption(4, "-o")),
                ItemClass::Ok(Item::NonOption(5, "--foo")),
                ItemClass::Ok(Item::NonOption(6, "blah")),
                ItemClass::Ok(Item::NonOption(7, "--bb")),
                ItemClass::Ok(Item::NonOption(8, "-h")),
                ItemClass::Ok(Item::NonOption(9, "--hah")),
                ItemClass::Ok(Item::NonOption(10, "--hah=")),
                ItemClass::Ok(Item::NonOption(11, "--")),
                ItemClass::Ok(Item::NonOption(12, "--hah=a")),
                ItemClass::Ok(Item::NonOption(13, "-oa")),
                ItemClass::Ok(Item::NonOption(14, "-b")),
            ],
        }
    );
}

/// Test empty long option names with data param (-- on it's own is obviously picked up as early
/// terminator, but what happens when an '=' is added?).
#[test]
fn long_no_name() {
    let opts = get_base();
    let args = arg_list!("--=a", "--=");
    let results = gong::process(&args, &opts);
    assert_eq!(results,
        Results {
            error: false,
            warn: true,
            items: vec![
                ItemClass::Warn(ItemW::LongWithNoName(0)),
                ItemClass::Warn(ItemW::LongWithNoName(1)),
            ],
        }
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Utf-8 character handling
////////////////////////////////////////////////////////////////////////////////////////////////////

mod utf8 {
    use super::*;

    /// Some utf8 multi-byte char handling
    #[test]
    fn test1() {
        let opts = get_base();
        let args = arg_list!("🗻∈🌏", "-🗻∈🌏", "--🗻∈🌏", "--ƒoo", "-❤");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Ok(Item::NonOption(0, "🗻∈🌏")),
                    ItemClass::Warn(ItemW::UnknownShort(1, '🗻')),
                    ItemClass::Warn(ItemW::UnknownShort(1, '∈')),
                    ItemClass::Warn(ItemW::UnknownShort(1, '🌏')),
                    ItemClass::Warn(ItemW::UnknownLong(2, "🗻∈🌏")),
                    ItemClass::Warn(ItemW::UnknownLong(3, "ƒoo")),
                    ItemClass::Ok(Item::Short(4, '❤')), // '\u{2764}' black heart
                ],
            }
        );
    }

    /// Some utf8 multi-byte char handling - chars with combinator chars (e.g. accent)
    #[test]
    fn test2() {
        let opts = get_base();
        let args = arg_list!("y̆", "-y̆", "--y̆", "ëéy̆", "-ëéy̆", "--ëéy̆", "--ábc", "--ábc");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Ok(Item::NonOption(0, "y̆")),
                    ItemClass::Warn(ItemW::UnknownShort(1, 'y')),        // 'y'
                    ItemClass::Warn(ItemW::UnknownShort(1, '\u{0306}')), // breve
                    ItemClass::Warn(ItemW::UnknownLong(2, "y̆")),
                    ItemClass::Ok(Item::NonOption(3, "ëéy̆")),
                    ItemClass::Warn(ItemW::UnknownShort(4, 'ë')),        // e+diaeresis
                    ItemClass::Warn(ItemW::UnknownShort(4, 'e')),        // 'e'
                    ItemClass::Warn(ItemW::UnknownShort(4, '\u{0301}')), // accute accent
                    ItemClass::Warn(ItemW::UnknownShort(4, 'y')),        // 'y'
                    ItemClass::Warn(ItemW::UnknownShort(4, '\u{0306}')), // breve
                    ItemClass::Warn(ItemW::UnknownLong(5, "ëéy̆")),
                    ItemClass::Warn(ItemW::UnknownLong(6, "ábc")),       // without combinator
                    ItemClass::Ok(Item::Long(7, "ábc")),                 // with combinator
                ],
            }
        );
    }

    /// Some utf8 multi-byte char width handling - chars with variation selector
    ///
    /// Here we use the "heavy black heart" char with variation selector #16 (emoji).
    #[test]
    fn test3() {
        let opts = get_base();
        // Note: the following is the 'black heart' character, followed by the variation selector
        // #16 (emoji) character.
        let args = arg_list!("❤️", "-❤️", "--❤️");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Ok(Item::NonOption(0, "❤️")),
                    ItemClass::Ok(Item::Short(1, '\u{2764}')),           // black-heart
                    ItemClass::Warn(ItemW::UnknownShort(1, '\u{fe0f}')), // emoji selector
                    ItemClass::Warn(ItemW::UnknownLong(2, "❤️")),
                ],
            }
        );
    }

    /// Some utf8 multi-byte char width handling - lone combinator chars
    #[test]
    fn test4() {
        let opts = get_base();
        let args = arg_list!("\u{0306}", "-\u{0306}", "--\u{0306}", "-\u{030A}");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Ok(Item::NonOption(0, "\u{0306}")),
                    ItemClass::Warn(ItemW::UnknownShort(1, '\u{0306}')),
                    ItemClass::Warn(ItemW::UnknownLong(2, "\u{0306}")),
                    ItemClass::Ok(Item::Short(3, '\u{030A}')),
                ],
            }
        );
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
        let opts = get_base();
        let args = arg_list!("--f");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: true,
                warn: false,
                items: vec![
                    ItemClass::Err(ItemE::AmbiguousLong(0, "f")),
                ],
            }
        );
    }

    /// Test handling of abbreviated long options, without ambiguity
    #[test]
    fn unambigous() {
        let opts = get_base();
        let args = arg_list!("--foo", "--foob");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::Long(0, "foo")),
                    ItemClass::Ok(Item::Long(1, "foobar")),
                ],
            }
        );
    }

    /// Test handling when abbreviated matching is disabled
    #[test]
    fn disabled() {
        let mut opts = get_base();
        opts.set_allow_abbreviations(false);
        let args = arg_list!("--f", "--fo", "--foo", "--foob", "--fooba", "--foobar");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Warn(ItemW::UnknownLong(0, "f")),
                    ItemClass::Warn(ItemW::UnknownLong(1, "fo")),
                    ItemClass::Ok(Item::Long(2, "foo")),
                    ItemClass::Warn(ItemW::UnknownLong(3, "foob")),
                    ItemClass::Warn(ItemW::UnknownLong(4, "fooba")),
                    ItemClass::Ok(Item::Long(5, "foobar")),
                ],
            }
        );
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
        let opts = get_base();
        let args = arg_list!("--hah", "def", "--help", "--hah=def", "--help");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "def", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::Long(2, "help")),
                    ItemClass::Ok(Item::LongWithData {
                        i: 3, n: "hah", d: "def", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::Long(4, "help")),
                ],
            }
        );
    }

    /// Test option with expected data arg, provided in next argument for short options
    #[test]
    fn arg_placement_short_next() {
        let opts = get_base();
        let args = arg_list!("-bxso", "def");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Warn(ItemW::UnknownShort(0, 'b')),
                    ItemClass::Ok(Item::Short(0, 'x')),
                    ItemClass::Warn(ItemW::UnknownShort(0, 's')),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 0, c: 'o', d: "def", l: DataLocation::NextArg }),
                ],
            }
        );
    }

    /// Test option with expected data arg, provided in same argument for short options
    #[test]
    fn arg_placement_short_same() {
        let opts = get_base();
        let args = arg_list!("-bsojx", "def");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Warn(ItemW::UnknownShort(0, 'b')),
                    ItemClass::Warn(ItemW::UnknownShort(0, 's')),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 0, c: 'o', d: "jx", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::NonOption(1, "def")),
                ],
            }
        );
    }

    /// Test missing argument data for long option
    #[test]
    fn missing_long() {
        let opts = get_base();
        let args = arg_list!("--hah");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: true,
                warn: false,
                items: vec![
                    ItemClass::Err(ItemE::LongMissingData(0, "hah")),
                ],
            }
        );
    }

    /// Test missing argument data for short option
    #[test]
    fn missing_short() {
        let opts = get_base();
        let args = arg_list!("-bxso");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: true,
                warn: true,
                items: vec![
                    ItemClass::Warn(ItemW::UnknownShort(0, 'b')),
                    ItemClass::Ok(Item::Short(0, 'x')),
                    ItemClass::Warn(ItemW::UnknownShort(0, 's')),
                    ItemClass::Err(ItemE::ShortMissingData(0, 'o')),
                ],
            }
        );
    }

    /// Test some misc. data handling.
    ///
    /// Unrecognised option with data; unrecognised with empty data; recognised with unexpected
    /// data; and recognised with empty unexpected data.
    #[test]
    fn misc() {
        let opts = get_base();
        let args = arg_list!("--xx=yy", "--tt=", "-x", "--foo=bar", "--foo=", "-x");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Warn(ItemW::UnknownLong(0, "xx")),
                    ItemClass::Warn(ItemW::UnknownLong(1, "tt")),
                    ItemClass::Ok(Item::Short(2, 'x')),
                    ItemClass::Warn(ItemW::LongWithUnexpectedData { i: 3, n: "foo", d: "bar" }),
                    ItemClass::Ok(Item::Long(4, "foo")),
                    ItemClass::Ok(Item::Short(5, 'x')),
                ],
            }
        );
    }

    /// Test option with expected data arg, declared to be in same argument, but empty
    #[test]
    fn same_arg_empty() {
        let opts = get_base();
        let args = arg_list!("--hah=", "--help", "--hah=", "help");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::Long(1, "help")),
                    ItemClass::Ok(Item::LongWithData {
                        i: 2, n: "hah", d: "", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::NonOption(3, "help")),
                ],
            }
        );
    }

    /// Test option with expected data arg, with data containing '='
    #[test]
    fn containing_equals() {
        let opts = get_base();
        let args = arg_list!("--hah", "d=ef", "--hah=d=ef", "--help", "--blah=ggg", "-oa=b");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "d=ef", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 2, n: "hah", d: "d=ef", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::Long(3, "help")),
                    ItemClass::Warn(ItemW::UnknownLong(4, "blah")),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 5, c: 'o', d: "a=b", l: DataLocation::SameArg }),
                ],
            }
        );
    }

    /// Test argument data that looks like options
    #[test]
    fn looking_like_options() {
        let opts = get_base();
        let args = arg_list!("--hah=--foo", "--hah", "--foo", "--hah=--blah", "--hah", "--blah",
            "--hah=-h", "--hah", "-h", "--hah=-n", "--hah", "-n", "-o-h", "-o", "-h", "-o-n", "-o",
            "-n", "-o--foo", "-o", "--hah", "-o--blah", "-o", "--blah");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "--foo", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 1, n: "hah", d: "--foo", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 3, n: "hah", d: "--blah", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 4, n: "hah", d: "--blah", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 6, n: "hah", d: "-h", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 7, n: "hah", d: "-h", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 9, n: "hah", d: "-n", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 10, n: "hah", d: "-n", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 12, c: 'o', d: "-h", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 13, c: 'o', d: "-h", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 15, c: 'o', d: "-n", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 16, c: 'o', d: "-n", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 18, c: 'o', d: "--foo", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 19, c: 'o', d: "--hah", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 21, c: 'o', d: "--blah", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 22, c: 'o', d: "--blah", l: DataLocation::NextArg }),
                ],
            }
        );
    }

    /// Test argument data that looks like early terminator
    #[test]
    fn looking_like_early_term() {
        let opts = get_base();
        let args = arg_list!("--hah=--", "--hah", "--", "-o", "--", "-o--");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "--", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 1, n: "hah", d: "--", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 3, c: 'o', d: "--", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::ShortWithData {
                        i: 5, c: 'o', d: "--", l: DataLocation::SameArg }),
                ],
            }
        );
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
        let mut opts = get_base();
        opts.set_mode(OptionsMode::Alternate);
        let args = arg_list!("abc", "-", "-help", "-hah=abc", "-hah", "cba", "-hah=", "-=", "-=abc",
            "-bxs", "--foo", "-f", "-foo", "-foob", "--", "-help");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: true,
                warn: true,
                items: vec![
                    ItemClass::Ok(Item::NonOption(0, "abc")),
                    ItemClass::Ok(Item::NonOption(1, "-")),
                    ItemClass::Ok(Item::Long(2, "help")),
                    ItemClass::Ok(Item::LongWithData {
                        i: 3, n: "hah", d: "abc", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 4, n: "hah", d: "cba", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 6, n: "hah", d: "", l: DataLocation::SameArg }),
                    ItemClass::Warn(ItemW::LongWithNoName(7)),
                    ItemClass::Warn(ItemW::LongWithNoName(8)),
                    ItemClass::Warn(ItemW::UnknownLong(9, "bxs")),
                    ItemClass::Warn(ItemW::UnknownLong(10, "-foo")),
                    ItemClass::Err(ItemE::AmbiguousLong(11, "f")),
                    ItemClass::Ok(Item::Long(12, "foo")),
                    ItemClass::Ok(Item::Long(13, "foobar")),
                    ItemClass::Ok(Item::EarlyTerminator(14)),
                    ItemClass::Ok(Item::NonOption(15, "-help")),
                ],
            }
        );
    }

    /// Check unexpected and missing data
    #[test]
    fn data_basic() {
        let mut opts = get_base();
        opts.set_mode(OptionsMode::Alternate);
        let args = arg_list!("-foo=abc", "-foo=", "-hah");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: true,
                warn: true,
                items: vec![
                    ItemClass::Warn(ItemW::LongWithUnexpectedData { i: 0, n: "foo", d: "abc" }),
                    ItemClass::Ok(Item::Long(1, "foo")),
                    ItemClass::Err(ItemE::LongMissingData(2, "hah")),
                ],
            }
        );
    }

    /// Test argument data that looks like early terminator
    #[test]
    fn data_looking_like_early_term() {
        let mut opts = get_base();
        opts.set_mode(OptionsMode::Alternate);
        let args = arg_list!("-hah=--", "-hah", "--");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "--", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 1, n: "hah", d: "--", l: DataLocation::NextArg }),
                ],
            }
        );
    }
}
