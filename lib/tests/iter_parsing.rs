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

extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use std::ffi::OsStr;
use gong::{longopt, command, command_set, option_set};
use gong::analysis::*;

/// Testing changing positionals policy during iterating
mod change_positionals_policy {
    use super::*;
    use gong::parser::{Parser, CmdParser};
    use gong::positionals::Policy;

    #[test]
    fn test() {
        let args = arg_list!("a", "b", "c", "d");

        let opts = option_set!();
        let mut parser = Parser::new(&opts);
        parser.set_positionals_policy(Policy::Fixed(1));

        let mut parse_iter = parser.parse_iter(&args).indexed();

        assert_eq!(0, parse_iter.get_positionals_count());
        assert_eq!(parse_iter.next(), Some(indexed_item!(0, Positional, "a")));
        assert_eq!(1, parse_iter.get_positionals_count());

        // You can freely change the policy now, though of course reducing the number of accepted
        // positionals below the number so far returned would make little sense.
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Unlimited));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Fixed(0)));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Fixed(1)));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Fixed(3)));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Max(0)));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Max(1)));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Max(3)));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Min(0)));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Min(1)));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::Min(3)));
        assert_eq!(Ok(()), parse_iter.set_positionals_policy(Policy::MinMax(0, 3)));

        // Actual to use from this point
        let _ = parse_iter.set_positionals_policy(Policy::Fixed(2));

        assert_eq!(parse_iter.next(), Some(indexed_item!(1, Positional, "b")));
        assert_eq!(2, parse_iter.get_positionals_count());
        assert_eq!(parse_iter.next(), Some(indexed_item!(2, UnexpectedPositional, "c")));
        assert_eq!(2, parse_iter.get_positionals_count()); // No change expected!

        // Changing the policy now (after an unexpected-positional) should fail, no matter what you
        // try to change it to
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Unlimited));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Fixed(0)));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Fixed(1)));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Fixed(3)));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Max(0)));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Max(1)));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Max(3)));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Min(0)));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Min(1)));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::Min(3)));
        assert_eq!(Err(()), parse_iter.set_positionals_policy(Policy::MinMax(0, 3)));

        // Actual to use from this point
        let _ = parse_iter.set_positionals_policy(Policy::Unlimited);

        // Should still be rejected as unexpected since changing policy should have failed
        assert_eq!(parse_iter.next(), Some(indexed_item!(3, UnexpectedPositional, "d")));
        assert_eq!(2, parse_iter.get_positionals_count()); // No change expected!
    }

    /// Checking that policy gets changed per-command
    #[test]
    fn commands() {
        let opts = option_set!();
        let cmds = command_set!([ command!("pull", @pp Policy::Max(3)) ]);
        let mut parser = CmdParser::new(&opts, &cmds);
        parser.set_positionals_policy(Policy::Max(1));

        let args = arg_list!("a", "b", "c", "d");
        let expected = expected!([
            cmd_indexed_item!(0, Positional, "a"),
            cmd_indexed_item!(1, UnexpectedPositional, "b"),
            cmd_indexed_item!(2, UnexpectedPositional, "c"),
            cmd_indexed_item!(3, UnexpectedPositional, "d"),
        ]);
        check_iter_result!(parser, args, expected);

        let args = arg_list!("pull", "a", "b", "c", "d");
        let expected = expected!([
            cmd_indexed_item!(0, Command, "pull"),
            cmd_indexed_item!(1, Positional, "a"),
            cmd_indexed_item!(2, Positional, "b"),
            cmd_indexed_item!(3, Positional, "c"),
            cmd_indexed_item!(4, UnexpectedPositional, "d"),
        ]);
        check_iter_result!(parser, args, expected);
    }
}

/// Testing change of option/command set and settings during iterations, and with passing iterator
/// to a command-specific handling function.
mod change_data {
    use super::*;
    use gong::parser::{CmdParser, OptionsMode, CmdParseIterIndexed};

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
        let main_opt_set = option_set!(@long [ longopt!(@flag "foo") ]);
        let main_cmd_set = command_set!([ command!("c1") ]);

        let mut parser = CmdParser::new(&main_opt_set, &main_cmd_set);
        parser.settings().set_mode(OptionsMode::Standard); // Explicitly enforce right starting state

        let mut parse_iter = parser.parse_iter(&args).indexed();

        assert_eq!(parse_iter.next(), Some(cmd_indexed_item!(0, Long, "foo")));
        assert_eq!(parse_iter.next(), Some(cmd_indexed_item!(1, Command, "c1")));

        // Here we pretend that the application responds to the `c1` command by passing the iterator
        // on to a function dedicated to handling a `c1` command situation, which is responsible for
        // modifying the parse iterator as necessary, and continuing the parse remaining arguments.
        c1(&parse_iter);
    }

    fn c1(parse_iter: &CmdParseIterIndexed<&str>) {
        let c1_opt_set = option_set!(@long [ longopt!(@flag "bar") ]);
        let c1_cmd_set = command_set!([ command!("c2") ]);

        let mut parse_iter = parse_iter.clone(); //Necessary to get around borrow checker

        parse_iter.set_option_set(&c1_opt_set);   // Change option set
        parse_iter.set_command_set(&c1_cmd_set);  // Change command set

        // Programs would not normally change settings part way through, it would confuse users,
        // this just tests that the ability to change settings (if a program really wanted to, or
        // actually has a genuine need) works.
        parse_iter.get_parse_settings().set_mode(OptionsMode::Alternate);
        // NB: We confirm change of settings took place successfully by the fact that we’re matching
        // the next option in alternate mode.

        assert_eq!(parse_iter.next(), Some(cmd_indexed_item!(2, Long, "bar")));
        assert_eq!(parse_iter.next(), Some(cmd_indexed_item!(3, Command, "c2")));
        assert_eq!(parse_iter.next(), None);
    }
}
