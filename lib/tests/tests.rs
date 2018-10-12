// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Unit tests

extern crate gong;

use gong::*;

/// A set of sample available options to use in tests
///
/// Note, there is a single test below that checks validity of these, thus it is not necessary to do
/// so in every test!
macro_rules! create_options {
    ( $o:ident ) => {
        let mut $o = Options::new(6, 5);
        $o.add_long("help")
            .add_short('h')
            .add_long("foo")
            .add_long("version")
            .add_long("foobar")
            .add_long_data("hah")
            .add_long("ábc")        // Using a combinator char (accent)
            .add_short('❤')
            .add_short('x')
            .add_short_data('o')
            .add_short('\u{030A}'); // A lone combinator ("ring above")
    };
}

//TODO: now that processing accepts both `String` and `&str` type "available option" sets, cover both somehow
/// Used for cleaner creation of set of test arguments
///
/// Note, arguments will normally be obtained from the environment, and Rust provides these to use
/// as String objects, not &str, hence why we create a vector of Strings!
macro_rules! arg_list {
    ( $($e:expr),+ ) => {
        vec![$(String::from($e)),+]
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Available options validation
////////////////////////////////////////////////////////////////////////////////////////////////////

mod available_options {
    use super::*;

    /// Check the set used for most of the tests here is valid
    #[test]
    fn common() {
        create_options!(opts);
        assert!(opts.is_valid());
    }

    /// Double check after inserting a problem
    #[test]
    #[should_panic]
    fn common2() {
        create_options!(opts);
        opts.add_long("foo"); // Duplicate - should panic here in debug mode!
        assert!(opts.is_valid());
    }

    /// Check the set used for most of the tests here is valid in `alt` mode
    #[test]
    fn common_alt() {
        create_options!(opts);
        opts.set_mode(OptionsMode::Alternate);
        assert!(opts.is_valid());
    }

    /* Dash ('-') is an invalid short option (clashes with early terminator if it were given on its
     * own (`--`), and would be misinterpreted as a long option if given as the first in a short
     * option set (`--abc`)). */

    /// Check `add_short` rejects '-'
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_short_dash() {
        let mut opts = Options::new(0, 1);
        opts.add_short('-'); // Should panic here in debug mode!
    }

    /// Check `add_short_data` rejects '-'
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_short_data_dash() {
        let mut opts = Options::new(0, 1);
        opts.add_short_data('-'); // Should panic here in debug mode!
    }

    /// Bypassing add methods, check `is_valid` rejects dash ('-') as short option
    #[test]
    #[should_panic]
    fn is_valid_short_dash() {
        let opts = Options {
            long: vec![],
            short: vec![
                ShortOption { ch: 'a', expects_data: false },
                ShortOption { ch: '-', expects_data: false },
                ShortOption { ch: 'b', expects_data: false },
            ],
            mode: Default::default(),
            allow_abbreviations: true,
        };
        assert!(opts.is_valid());
    }

    /// Check behaviour when validity check bypassed.
    ///
    /// This situation is an invalid use case, the user should always validate their option set;
    /// this test verifies that things behave though as we expect if the set is invalid due to a
    /// short that is a dash ('-').
    #[test]
    fn short_dash_bypass() {
        let opts = Options {
            long: vec![],
            short: vec![
                ShortOption { ch: '-', expects_data: false },
            ],
            mode: Default::default(),
            allow_abbreviations: true,
        };
        //assert!(opts.is_valid()); DISABLED! WHAT HAPPENS NEXT? LET'S SEE...
        let args = arg_list!("--abc", "-a-bc", "--");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    ItemClass::Warn(ItemW::UnknownLong(0, "abc")),
                    ItemClass::Warn(ItemW::UnknownShort(1, 'a')),
                    ItemClass::Ok(Item::Short(1, '-')),
                    ItemClass::Warn(ItemW::UnknownShort(1, 'b')),
                    ItemClass::Warn(ItemW::UnknownShort(1, 'c')),
                    ItemClass::Ok(Item::EarlyTerminator(2)),
                ],
            }
        );
    }

    /// Check `add_long` rejects empty string
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long_no_name() {
        let mut opts = Options::new(1, 0);
        opts.add_long(""); // Should panic here in debug mode!
    }

    /// Check `add_long_data` rejects empty string
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long_data_no_name() {
        let mut opts = Options::new(1, 0);
        opts.add_long_data(""); // Should panic here in debug mode!
    }

    /// Bypassing add methods, check `is_valid` rejects empty name long option
    #[test]
    #[should_panic]
    fn is_valid_long_no_name() {
        let opts = Options {
            long: vec![
                LongOption { name: "foo", expects_data: false },
                LongOption { name: "", expects_data: false },
                LongOption { name: "bar", expects_data: false },
            ],
            short: vec![],
            mode: Default::default(),
            allow_abbreviations: true,
        };
        assert!(opts.is_valid());
    }

    /* Long option names cannot contain an '=' (used for declaring a data sub-argument in the same
     * argument; if names could contain an '=', as data can, we would not know where to do the
     * split, complicating matching. */

    /// Check `add_long` rejects equals ('=') char
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long_with_equals() {
        let mut opts = Options::new(1, 0);
        opts.add_long("a=b"); // Should panic here in debug mode!
    }

    /// Check `add_long_data` rejects equals ('=') char
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_long_data_with_equals() {
        let mut opts = Options::new(1, 0);
        opts.add_long_data("a=b"); // Should panic here in debug mode!
    }

    /// Bypassing add methods, check `is_valid` rejects long option with equals ('=')
    #[test]
    #[should_panic]
    fn is_valid_long_with_equals() {
        let opts = Options {
            long: vec![
                LongOption { name: "foo", expects_data: false },
                LongOption { name: "a=b", expects_data: false },
                LongOption { name: "bar", expects_data: false },
            ],
            short: vec![],
            mode: Default::default(),
            allow_abbreviations: true,
        };
        assert!(opts.is_valid());
    }

    /// Check behaviour when validity check bypassed.
    ///
    /// This situation is an invalid use case, the user should always validate their option set;
    /// this test verifies that things behave though as we expect if the set is invalid due to a
    /// long with an equals ('=').
    #[test]
    fn long_with_equals_bypass() {
        let opts = Options {
            long: vec![
                LongOption { name: "a=b", expects_data: false },
            ],
            short: vec![],
            mode: Default::default(),
            allow_abbreviations: true,
        };
        //assert!(opts.is_valid()); DISABLED! WHAT HAPPENS NEXT? LET'S SEE...
        let args = arg_list!("--a", "--a=b");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: true,
                items: vec![
                    // If option "a=b" was accepted as a valid option, "--a" would match as an
                    // abbreviation. "--a=b" would be split into "a" being the name and "b" as the
                    // data, and "a" matches as an abbreviation.
                    ItemClass::Ok(Item::Long(0, "a=b")),
                    ItemClass::Warn(ItemW::LongWithUnexpectedData { i: 1, n: "a=b", d: "b" }),
                ],
            }
        );
    }

    /* Option sets should not contain duplicates.
     *
     * Duplicates pose a potential problem due to potential for confusion over differing
     * `expects_data` attributes. They also can result from option name-clashing bugs with programs
     * that dynamically generate (large) option sets (rare? VLC media player is one example, which
     * dynamically builds an option set including options from plugins). An option set containing
     * duplicates is thus considered invalid. */

    /// Short option duplicates
    #[test]
    #[should_panic]
    fn short_dups() {
        let mut opts = Options::new(0, 8);
        opts.add_short('a')
            .add_short('b')
            .add_short('c')
            .add_short('c')       // dup
            .add_short_data('d')
            .add_short_data('b')  // dup (ignore data indicator)
            .add_short('e')
            .add_short('b');      // dup
        assert!(opts.is_valid());
    }

    /// Long option duplicates
    #[test]
    #[should_panic]
    fn long_dups() {
        let mut opts = Options::new(3, 0);
        opts.add_long("aaa")
            .add_long("bbb")
            .add_long("ccc")
            .add_long("ccc")      // dup
            .add_long_data("ddd")
            .add_long_data("bbb") // dup (ignore data indicator)
            .add_long("eee")
            .add_long("bbb");     // dup
        assert!(opts.is_valid());
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Basic option handling
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Some general, basic argument handling
#[test]
fn basic() {
    create_options!(opts);
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

/// Test that only first early terminator is recognised as such, additional ones are interpreted as
/// non-options.
#[test]
fn early_term() {
    create_options!(opts);
    let args = arg_list!("--foo", "--", "--help", "--", "-o");
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
            ],
        }
    );
}

/// Test empty long option names with data param (-- on it's own is obviously picked up as early
/// terminator, but what happens when an '=' is added?).
#[test]
fn long_no_name() {
    create_options!(opts);
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
        create_options!(opts);
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
        create_options!(opts);
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
        create_options!(opts);
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
        create_options!(opts);
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
        create_options!(opts);
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
        create_options!(opts);
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
        create_options!(opts);
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

    /// Test some misc. data handling.
    ///
    /// Unrecognised option with data; unrecognised with empty data; recognised with unexpected
    /// data; and recognised with empty unexpected data.
    #[test]
    fn basic() {
        create_options!(opts);
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

    /// Test option with expected data arg, provided in next argument
    #[test]
    fn next_arg() {
        create_options!(opts);
        let args = arg_list!("--hah", "def", "--help");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "def", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::Long(2, "help")),
                ],
            }
        );
    }

    /// Test option with expected data arg, provided in same argument
    #[test]
    fn same_arg() {
        create_options!(opts);
        let args = arg_list!("--hah=def", "--help");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "def", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::Long(1, "help")),
                ],
            }
        );
    }

    /// Test option with expected data arg, declared to be in same argument, but empty
    #[test]
    fn same_arg_empty() {
        create_options!(opts);
        let args = arg_list!("--hah=", "--help");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::Long(1, "help")),
                ],
            }
        );
    }

    /// Test option with expected data arg, with data containing '='
    #[test]
    fn containing_equals() {
        create_options!(opts);
        let args = arg_list!("--hah", "d=ef", "--hah=d=ef", "--help");
        let results = gong::process(&args, &opts);
        assert_eq!(results,
            Results {
                error: false,
                warn: false,
                items: vec![
                    ItemClass::Ok(Item::LongWithData {
                        i: 0, n: "hah", d: "d=ef", l: DataLocation::NextArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 2, n: "hah", d: "d=ef", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::Long(3, "help")),
                ],
            }
        );
    }

    /// Test missing argument data
    #[test]
    fn missing() {
        create_options!(opts);
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

    /// Test argument data for short option, provided in next argument
    #[test]
    fn short_next_arg() {
        create_options!(opts);
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

    /// Test argument data for short option, provided in same argument
    #[test]
    fn short_same_arg() {
        create_options!(opts);
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

    /// Test missing argument data for short option
    #[test]
    fn short_missing() {
        create_options!(opts);
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

    /// Test argument data that looks like options
    #[test]
    fn looking_like_options() {
        create_options!(opts);
        let args = arg_list!("--hah=--foo", "--hah", "--foo", "--hah=-n", "--hah", "-n");
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
                        i: 3, n: "hah", d: "-n", l: DataLocation::SameArg }),
                    ItemClass::Ok(Item::LongWithData {
                        i: 4, n: "hah", d: "-n", l: DataLocation::NextArg }),
                ],
            }
        );
    }

    /// Test argument data that looks like early terminator
    #[test]
    fn looking_like_early_term() {
        create_options!(opts);
        let args = arg_list!("--hah=--", "--hah", "--");
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
        create_options!(opts);
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
        create_options!(opts);
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
        create_options!(opts);
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
