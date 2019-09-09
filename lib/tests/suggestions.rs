// Copyright 2018 Lyndon Brown
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

#[cfg(feature = "suggestions")]
mod options {
    use std::ffi::OsStr;
    use gong::{longopt, option_set};
    use gong::analysis::*;
    use gong::parser::Parser;

    #[test]
    fn basic() {
        let opts = option_set!(@long [
            longopt!(@flag "b"),
            longopt!(@flag "bar"),
            longopt!(@flag "help"),
            longopt!(@flag "but_i_digress"),
        ]);
        assert!(opts.is_valid());

        let args = arg_list!("--a", "--foo", "--hellp", "--but_i_digest");
        let expected = expected!([
            indexed_item!(0, UnknownLong, "a", None),
            indexed_item!(1, UnknownLong, "foo", None),
            indexed_item!(2, UnknownLong, "hellp", Some("help")),
            indexed_item!(3, UnknownLong, "but_i_digest", Some("but_i_digress")),
        ]);

        let mut parser = Parser::new(&opts);
        parser.settings().set_stop_on_problem(false)
                         .set_serve_suggestions(true);
        let items: Vec<_> = parser.parse_iter(&args).indexed().collect();
        assert_eq!(&items[..], &expected[..]);
    }

    /// Check searching respects best match, i.e. keeping track works in algorithm
    #[test]
    fn best() {
        let opts = option_set!(@long [
            // Putting best match for `bard` first
            longopt!(@flag "bar"),   //bart gets metric of 0.9416666666666667
            longopt!(@flag "bart"),  //bart gets metric of 0.8833333333333333
            // Putting best match for `hellp` last
            longopt!(@flag "hello"), //hellp gets metric of 0.92
            longopt!(@flag "help"),  //hellp gets metric of 0.9533333333333333
            // Equal matches for `fooa`
            longopt!(@flag "foob"),  //fooa gets metric of 0.8833333333333333
            longopt!(@flag "fooc"),  //fooa gets metric of 0.8833333333333333
        ]);
        assert!(opts.is_valid());

        let args = arg_list!("--hellp", "--bard", "--fooa");
        let expected = expected!([
            indexed_item!(0, UnknownLong, "hellp", Some("help")),
            indexed_item!(1, UnknownLong, "bard", Some("bar")),
            indexed_item!(2, UnknownLong, "fooa", Some("foob")),
        ]);

        let mut parser = Parser::new(&opts);
        parser.settings().set_stop_on_problem(false)
                         .set_serve_suggestions(true);
        let items: Vec<_> = parser.parse_iter(&args).indexed().collect();
        assert_eq!(&items[..], &expected[..]);
    }
}

#[cfg(feature = "suggestions")]
mod commands {
    use std::ffi::OsStr;
    use gong::{command, command_set, option_set};
    use gong::analysis::*;
    use gong::parser::CmdParser;
    use self::super::common::{CmdActual, CmdExpected};

    #[test]
    fn basic() {
        let opts = option_set!();
        let cmds = command_set!([
            command!("b"),
            command!("bar"),
            command!("but_i_digress"),
            command!("help"),
        ]);
        assert!(cmds.is_valid());

        let args = arg_list!("but_i_digest");
        let expected = cmd_dm_expected!(
            problems: true,
            @part cmd_part!(item_set: item_set!(
                problems: true,
                opt_set: &opts,
                [
                    item!(UnknownCommand, "but_i_digest", Some("but_i_digress")),
                ])
            ),
            cmd_set: Some(&cmds)
        );

        let mut parser = CmdParser::new(&opts, &cmds);
        parser.settings().set_stop_on_problem(false)
                         .set_serve_suggestions(true);
        let actual_results = CmdActual(parser.parse(&args));
        check_result!(&actual_results, &expected);
    }

    /// Check searching respects best match, i.e. keeping track works in algorithm
    #[test]
    fn best_first() {
        let opts = option_set!();
        let cmds = command_set!([
            // Putting best match for `bard` first
            command!("bar"),   //bart gets metric of 0.9416666666666667
            command!("bart"),  //bart gets metric of 0.8833333333333333
        ]);
        assert!(cmds.is_valid());

        let args = arg_list!("bard");
        let expected = cmd_dm_expected!(
            problems: true,
            @part cmd_part!(item_set: item_set!(
                problems: true,
                opt_set: &opts,
                [
                    item!(UnknownCommand, "bard", Some("bar")),
                ])
            ),
            cmd_set: Some(&cmds)
        );

        let mut parser = CmdParser::new(&opts, &cmds);
        parser.settings().set_stop_on_problem(false)
                         .set_serve_suggestions(true);
        let actual_results = CmdActual(parser.parse(&args));
        check_result!(&actual_results, &expected);
    }

    /// Check searching respects best match, i.e. keeping track works in algorithm
    #[test]
    fn best_last() {
        let opts = option_set!();
        let cmds = command_set!([
            // Putting best match for `hellp` last
            command!("hello"), //hellp gets metric of 0.92
            command!("help"),  //hellp gets metric of 0.9533333333333333
        ]);
        assert!(cmds.is_valid());

        let args = arg_list!("hellp");
        let expected = cmd_dm_expected!(
            problems: true,
            @part cmd_part!(item_set: item_set!(
                problems: true,
                opt_set: &opts,
                [
                    item!(UnknownCommand, "hellp", Some("help")),
                ])
            ),
            cmd_set: Some(&cmds)
        );

        let mut parser = CmdParser::new(&opts, &cmds);
        parser.settings().set_stop_on_problem(false)
                         .set_serve_suggestions(true);
        let actual_results = CmdActual(parser.parse(&args));
        check_result!(&actual_results, &expected);
    }

    /// Check searching respects best match, i.e. keeping track works in algorithm
    #[test]
    fn best_equal() {
        let opts = option_set!();
        let cmds = command_set!([
            // Equal matches for `fooa`
            command!("foob"), //fooa gets metric of 0.8833333333333333
            command!("fooc"), //fooa gets metric of 0.8833333333333333
        ]);
        assert!(cmds.is_valid());

        let args = arg_list!("fooa");
        let expected = cmd_dm_expected!(
            problems: true,
            @part cmd_part!(item_set: item_set!(
                problems: true,
                opt_set: &opts,
                [
                    item!(UnknownCommand, "fooa", Some("foob")),
                ])
            ),
            cmd_set: Some(&cmds)
        );

        let mut parser = CmdParser::new(&opts, &cmds);
        parser.settings().set_stop_on_problem(false)
                         .set_serve_suggestions(true);
        let actual_results = CmdActual(parser.parse(&args));
        check_result!(&actual_results, &expected);
    }
}
