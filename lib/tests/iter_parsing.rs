// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Iterative style parsing tests
//!
//! Most of the test suite uses “all in one” style, and considering how it is built upon “iterative”
//! style, we do not need to do much testing of “iterative” style itself, but some things should be
//! checked, which is done here.

extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use std::ffi::OsStr;
use gong::{longopt, command, command_set, option_set};
use gong::analysis::*;
use common::get_parser;

/// Some general, basic argument handling
#[test]
fn basic() {
    let args = arg_list!(
        "abc",          // Positional
        "--help",       // Long option
        "-bxs",         // Short option set, two unknown, one known (`x`)
        "--hah=xyz",    // Data taking option, in-same-arg
        "--ƒƒ", "cba",  // Data taking option, in-next-arg
        "-o123",        // Data taking short option, in-same-arg
        "-Ɛ", "456",    // Data taking short option, in-next-arg
    );
    let parser = get_parser();
    let mut parse_iter = parser.parse_iter(&args);
    assert_eq!(parse_iter.next(), Some(expected_item!(0, Positional, "abc")));
    assert_eq!(parse_iter.next(), Some(expected_item!(1, Long, "help")));
    assert_eq!(parse_iter.next(), Some(expected_item!(2, UnknownShort, 'b')));
    assert_eq!(parse_iter.next(), Some(expected_item!(2, Short, 'x')));
    assert_eq!(parse_iter.next(), Some(expected_item!(2, UnknownShort, 's')));
    assert_eq!(parse_iter.next(), Some(expected_item!(3, LongWithData, "hah", "xyz", DataLocation::SameArg)));
    assert_eq!(parse_iter.next(), Some(expected_item!(4, LongWithData, "ƒƒ", "cba", DataLocation::NextArg)));
    assert_eq!(parse_iter.next(), Some(expected_item!(6, ShortWithData, 'o', "123", DataLocation::SameArg)));
    assert_eq!(parse_iter.next(), Some(expected_item!(7, ShortWithData, 'Ɛ', "456", DataLocation::NextArg)));
    assert_eq!(parse_iter.next(), None);
    assert_eq!(parse_iter.next(), None);
}

/// Testing change of option/command set and settings during iterations, and with passing iterator
/// to a command-specific handling function.
mod change_data {
    use super::*;
    use gong::parser::{Parser, OptionsMode, ParseIter};

    #[test]
    fn main() {
        let args = arg_list!(
            "--foo",    // Long option from set #1, using standard style
            "c1",       // Command from set #1
            "-bar",     // Long option belonging to `c1` command, using alternate style
            "c2",       // Command belonging to `c1` command
        );

        // Here, we describe only the top level configuration of the application only, where there
        // is one option (`foo`), and one command (`c1`), where we do not specify any option set or
        // sub-command set for the `c1` command.
        let main_opt_set = option_set!(@long [ longopt!("foo") ]);
        let main_cmd_set = command_set!([ command!("c1") ]);

        let mut parser = Parser::new(&main_opt_set, Some(&main_cmd_set));
        parser.settings.set_mode(OptionsMode::Standard); // Explicitly enforce right starting state

        let mut parse_iter = parser.parse_iter(&args);

        assert_eq!(parse_iter.next(), Some(expected_item!(0, Long, "foo")));
        assert_eq!(parse_iter.next(), Some(expected_item!(1, Command, "c1")));

        // Here we pretend that the application responds to the `c1` command by passing the iterator
        // on to a function dedicated to handling a `c1` command situation, which is responsible for
        // modifying the parse iterator as necessary, and continuing the parse remaining arguments.
        c1(&parse_iter);
    }

    fn c1(parse_iter: &ParseIter<&OsStr>) {
        let c1_opt_set = option_set!(@long [ longopt!("bar") ]);
        let c1_cmd_set = command_set!([ command!("c2") ]);

        let mut parse_iter = parse_iter.clone(); //Necessary to get around borrow checker

        parse_iter.set_option_set(&c1_opt_set);   // Change option set
        parse_iter.set_command_set(&c1_cmd_set);  // Change command set

        // Programs would not normally change settings part way through, it would confuse users,
        // this just tests that the ability to change settings (if a program really wanted to, or
        // actually has a genuine need) works.
        let mut new_settings = parse_iter.get_parse_settings(); // Get current settings
        new_settings.set_mode(OptionsMode::Alternate);          // Change them
        parse_iter.set_parse_settings(new_settings);            // Apply them
        // NB: We confirm change of settings took place successfully by the fact that we're matching
        // the next option in alternate mode.

        assert_eq!(parse_iter.next(), Some(expected_item!(2, Long, "bar")));
        assert_eq!(parse_iter.next(), Some(expected_item!(3, Command, "c2")));
        assert_eq!(parse_iter.next(), None);
    }
}
