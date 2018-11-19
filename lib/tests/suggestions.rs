// Copyright 2018 Lyndon Brown
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

#[cfg(feature = "suggestions")]
mod options {
    use gong::analysis::*;
    use gong::parser::Parser;
    use common::{Actual, Expected, check_result};

    #[test]
    fn basic() {
        let args = arg_list!("--a", "--foo", "--hellp", "--but_i_digest");
        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, UnknownLong, "a"),
                expected_item!(1, UnknownLong, "foo"),
                expected_item!(2, UnknownLong, "hellp"),
                expected_item!(3, UnknownLong, "but_i_digest"),
            ]
        );
        let opts = gong_option_set!(
            [
                gong_longopt!("b"),
                gong_longopt!("bar"),
                gong_longopt!("help"),
                gong_longopt!("but_i_digress"),
            ], []
        );
        assert!(opts.is_valid());
        let parser = Parser::new(&opts, None);
        let actual_results = Actual(parser.parse(&args));
        check_result(&actual_results, &expected);

        let mut suggestions = Vec::with_capacity(actual_results.0.items.len());
        for item in &actual_results.0.items {
            match item {
                ItemClass::Warn(ItemW::UnknownLong(_, name)) => {
                    suggestions.push((*name, opts.suggest(name)));
                },
                _ => unreachable!(),
            }
        }
        assert_eq!(suggestions, vec!(
            ("a", None),
            ("foo", None),
            ("hellp", Some("help")),
            ("but_i_digest", Some("but_i_digress")),
        ));
    }

    /// Check searching respects best match, i.e. keeping track works in algorithm
    #[test]
    fn best() {
        let args = arg_list!("--hellp", "--bard", "--fooa");
        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, UnknownLong, "hellp"),
                expected_item!(1, UnknownLong, "bard"),
                expected_item!(2, UnknownLong, "fooa"),
            ]
        );
        let opts = gong_option_set!(
            [
                // Putting best match for `bard` first
                gong_longopt!("bar"),   //bart gets metric of 0.9416666666666667
                gong_longopt!("bart"),  //bart gets metric of 0.8833333333333333
                // Putting best match for `hellp` last
                gong_longopt!("hello"), //hellp gets metric of 0.92
                gong_longopt!("help"),  //hellp gets metric of 0.9533333333333333
                // Equal matches for `fooa`
                gong_longopt!("foob"),  //fooa gets metric of 0.8833333333333333
                gong_longopt!("fooc"),  //fooa gets metric of 0.8833333333333333
            ], []
        );
        assert!(opts.is_valid());
        let parser = Parser::new(&opts, None);
        let actual_results = Actual(parser.parse(&args));
        check_result(&actual_results, &expected);

        let mut suggestions = Vec::with_capacity(actual_results.0.items.len());
        for item in &actual_results.0.items {
            match item {
                ItemClass::Warn(ItemW::UnknownLong(_, name)) => {
                    suggestions.push((*name, opts.suggest(name)));
                },
                _ => unreachable!(),
            }
        }
        assert_eq!(suggestions, vec!(
            ("hellp", Some("help")),
            ("bard", Some("bar")),
            ("fooa", Some("foob")),
        ));
    }
}

#[cfg(feature = "suggestions")]
mod commands {
    use gong::analysis::*;
    use gong::parser::Parser;
    use common::{Actual, Expected, check_result};

    #[test]
    fn basic() {
        let args = arg_list!("but_i_digest");
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, NonOption, "but_i_digest"),
            ]
        );
        let opts = gong_option_set!();
        let cmds = gong_command_set!(
            [
                gong_command!("b"),
                gong_command!("bar"),
                gong_command!("but_i_digress"),
                gong_command!("help"),
            ]
        );
        assert!(cmds.is_valid());
        let parser = Parser::new(&opts, Some(&cmds));
        let actual_results = Actual(parser.parse(&args));
        check_result(&actual_results, &expected);

        let mut suggestions = Vec::with_capacity(actual_results.0.items.len());
        for item in &actual_results.0.items {
            match item {
                ItemClass::Ok(Item::NonOption(_, s)) => {
                    suggestions.push((*s, cmds.suggest(s)));
                },
                _ => unreachable!(),
            }
        }
        assert_eq!(suggestions, vec!(
            ("but_i_digest", Some("but_i_digress")),
        ));
    }

    /// Check searching respects best match, i.e. keeping track works in algorithm
    #[test]
    fn best_first() {
        let args = arg_list!("bard");
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, NonOption, "bard"),
            ]
        );
        let opts = gong_option_set!();
        let cmds = gong_command_set!(
            [
                // Putting best match for `bard` first
                gong_command!("bar"),   //bart gets metric of 0.9416666666666667
                gong_command!("bart"),  //bart gets metric of 0.8833333333333333
            ]
        );
        assert!(cmds.is_valid());
        let parser = Parser::new(&opts, Some(&cmds));
        let actual_results = Actual(parser.parse(&args));
        check_result(&actual_results, &expected);

        let mut suggestions = Vec::with_capacity(actual_results.0.items.len());
        for item in &actual_results.0.items {
            match item {
                ItemClass::Ok(Item::NonOption(_, s)) => {
                    suggestions.push((*s, cmds.suggest(s)));
                },
                _ => unreachable!(),
            }
        }
        assert_eq!(suggestions, vec!(
            ("bard", Some("bar")),
        ));
    }

    /// Check searching respects best match, i.e. keeping track works in algorithm
    #[test]
    fn best_last() {
        let args = arg_list!("hellp");
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, NonOption, "hellp"),
            ]
        );
        let opts = gong_option_set!();
        let cmds = gong_command_set!(
            [
                // Putting best match for `hellp` last
                gong_command!("hello"), //hellp gets metric of 0.92
                gong_command!("help"),  //hellp gets metric of 0.9533333333333333
            ]
        );
        assert!(cmds.is_valid());
        let parser = Parser::new(&opts, Some(&cmds));
        let actual_results = Actual(parser.parse(&args));
        check_result(&actual_results, &expected);

        let mut suggestions = Vec::with_capacity(actual_results.0.items.len());
        for item in &actual_results.0.items {
            match item {
                ItemClass::Ok(Item::NonOption(_, s)) => {
                    suggestions.push((*s, cmds.suggest(s)));
                },
                _ => unreachable!(),
            }
        }
        assert_eq!(suggestions, vec!(
            ("hellp", Some("help")),
        ));
    }

    /// Check searching respects best match, i.e. keeping track works in algorithm
    #[test]
    fn best_equal() {
        let args = arg_list!("fooa");
        let expected = expected!(
            error: false,
            warn: false,
            [
                expected_item!(0, NonOption, "fooa"),
            ]
        );
        let opts = gong_option_set!();
        let cmds = gong_command_set!(
            [
                // Equal matches for `fooa`
                gong_command!("foob"), //fooa gets metric of 0.8833333333333333
                gong_command!("fooc"), //fooa gets metric of 0.8833333333333333
            ]
        );
        assert!(cmds.is_valid());
        let parser = Parser::new(&opts, Some(&cmds));
        let actual_results = Actual(parser.parse(&args));
        check_result(&actual_results, &expected);

        let mut suggestions = Vec::with_capacity(actual_results.0.items.len());
        for item in &actual_results.0.items {
            match item {
                ItemClass::Ok(Item::NonOption(_, s)) => {
                    suggestions.push((*s, cmds.suggest(s)));
                },
                _ => unreachable!(),
            }
        }
        assert_eq!(suggestions, vec!(
            ("fooa", Some("foob")),
        ));
    }
}
