// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Testing “available” option/command set construction/modification
//!
//! Note, construction with macros is tested separately

extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use gong::{longopt, shortopt, command, command_set, option_set};

mod options {
    use super::*;
    use gong::options::*;

    /// Check basic valid construction methods
    #[test]
    fn basic() {
        let mut opts = OptionSetEx::new();
        opts.add_short('h')
            .add_short_data('o')
            .add_existing_short(shortopt!('a'))
            .add_long("foo")
            .add_long_data("bar")
            .add_existing_long(longopt!("foobar"));

        let expected = OptionSetEx {
            long: vec![
                longopt!("foo"),
                longopt!(@data "bar"),
                longopt!("foobar"),
            ],
            short: vec![
                shortopt!('h'),
                shortopt!(@data 'o'),
                shortopt!('a'),
            ],
        };

        assert_eq!(opts, expected);
        assert!(opts.validate().is_ok());
    }

    /// Check adding more than one short option at a time from string
    #[test]
    fn short_opt_string() {
        let mut opts = OptionSetEx::new();

        // Add some existing options, to check they are not modified in any way
        opts.add_short('h')
            .add_long("foo")
            .add_short_data('o')
            .add_long_data("bar");

        opts.add_shorts_from_str("ab:cde:f");

        let mut expected = OptionSetEx {
            long: vec![
                longopt!("foo"),
                longopt!(@data "bar"),
            ],
            short: vec![
                shortopt!('h'),
                shortopt!(@data 'o'),
                shortopt!('a'),
                shortopt!(@data 'b'),
                shortopt!('c'),
                shortopt!('d'),
                shortopt!(@data 'e'),
                shortopt!('f'),
            ],
        };
        assert_eq!(opts, expected);

        opts.add_shorts_from_str("");
        assert_eq!(opts, expected);

        opts.add_shorts_from_str(":");
        assert_eq!(opts, expected);

        opts.add_shorts_from_str(":::::");
        assert_eq!(opts, expected);

        opts.add_shorts_from_str(" ");
        expected.add_short(' ');
        assert_eq!(opts, expected);

        opts.add_shorts_from_str(" :");
        expected.add_short_data(' ');
        assert_eq!(opts, expected);

        opts.add_shorts_from_str(":jkl");
        expected.add_short('j')
                .add_short('k')
                .add_short('l');
        assert_eq!(opts, expected);

        opts.add_shorts_from_str("mn:::op");
        expected.add_short('m')
                .add_short_data('n')
                .add_short('o')
                .add_short('p');
        assert_eq!(opts, expected);

        // Double check
        let expected = OptionSetEx {
            long: vec![
                longopt!("foo"),
                longopt!(@data "bar"),
            ],
            short: vec![
                shortopt!('h'),
                shortopt!(@data 'o'),
                shortopt!('a'),
                shortopt!(@data 'b'),
                shortopt!('c'),
                shortopt!('d'),
                shortopt!(@data 'e'),
                shortopt!('f'),
                shortopt!(' '),
                shortopt!(@data ' '),
                shortopt!('j'),
                shortopt!('k'),
                shortopt!('l'),
                shortopt!('m'),
                shortopt!(@data 'n'),
                shortopt!('o'),
                shortopt!('p'),
            ],
        };
        assert_eq!(opts, expected);
    }

    /// Check `is_empty`
    #[test]
    fn is_empty() {
        // Here, let's double-check that the derive of `Default` for fixed option sets is really an
        // empty set
        let opt_set = OptionSet::default();
        assert!(opt_set.is_empty());

        let opt_set = option_set!();
        assert!(opt_set.is_empty());
        let opt_set = option_set!(@long [], @short []);
        assert!(opt_set.is_empty());
        let opt_set = option_set!(@short [], @long []);
        assert!(opt_set.is_empty());
        let opt_set = option_set!(@long []);
        assert!(opt_set.is_empty());
        let opt_set = option_set!(@short []);
        assert!(opt_set.is_empty());

        let opt_set = option_set!(@long [ longopt!("foo") ]);
        assert!(!opt_set.is_empty());

        let opt_set = option_set!(@short [ shortopt!('h') ]);
        assert!(!opt_set.is_empty());

        let opt_set = option_set!(
            @long [ longopt!("foo") ],
            @short [ shortopt!('h') ]
        );
        assert!(!opt_set.is_empty());
    }

    /// Check set type (`OptionSet`/`OptionSetEx`) conversion and comparison
    #[test]
    fn set_types() {
        // Test set - fixed
        let opts_fixed = OptionSet {
            long: &[
                longopt!("foo"),
                longopt!(@data "bar"),
                longopt!("foobar"),
            ],
            short: &[
                shortopt!('h'),
                shortopt!(@data 'o'),
                shortopt!('a'),
            ],
        };

        // Test set - extendible
        let opts_extendible = OptionSetEx {
            long: vec![
                longopt!("foo"),
                longopt!(@data "bar"),
                longopt!("foobar"),
            ],
            short: vec![
                shortopt!('h'),
                shortopt!(@data 'o'),
                shortopt!('a'),
            ],
        };

        // Check the two types can be compared
        assert_eq!(true, opts_fixed.eq(&opts_extendible));
        assert_eq!(true, opts_extendible.eq(&opts_fixed));

        // Check conversions
        let fixed_from_extendible: OptionSet = opts_extendible.as_fixed();
        assert_eq!(true, opts_fixed.eq(&fixed_from_extendible));
        assert_eq!(true, opts_extendible.eq(&fixed_from_extendible));
        assert_eq!(true, fixed_from_extendible.eq(&opts_fixed));
        assert_eq!(true, fixed_from_extendible.eq(&opts_extendible));
        let extendible_from_fixed: OptionSetEx = opts_fixed.to_extendible();
        assert_eq!(true, opts_fixed.eq(&extendible_from_fixed));
        assert_eq!(true, opts_extendible.eq(&extendible_from_fixed));
        assert_eq!(true, extendible_from_fixed.eq(&opts_fixed));
        assert_eq!(true, extendible_from_fixed.eq(&opts_extendible));

        // Test negative comparisons

        let opts_fixed_2 = OptionSet {
            long: &[
                longopt!("blah"),
            ],
            short: &[],
        };

        let opts_extendible_2 = OptionSetEx {
            long: vec![
                longopt!("blah"),
            ],
            short: vec![],
        };

        // Verify not equal
        assert!(opts_fixed != opts_fixed_2);
        assert!(opts_fixed != opts_extendible_2);
        assert!(opts_extendible != opts_fixed_2);
        assert!(opts_extendible != opts_extendible_2);
    }

    /// Check re-use of descriptors
    #[test]
    fn reuse() {
        const SHORT_OPT_H: ShortOption = shortopt!('h');
        const LONG_OPT_HELP: LongOption = longopt!("help");

        let _ = option_set!(
            @long [
                LONG_OPT_HELP,
                longopt!("foo"),
                longopt!(@data "bar"),
                longopt!("foobar"),
            ],
            @short [
                SHORT_OPT_H,
                shortopt!(@data 'o'),
                shortopt!('a'),
            ]
        );
    }
}

mod commands {
    use super::*;
    use gong::commands::*;

    /// Check basic valid construction methods
    #[test]
    fn basic() {
        let mut cmds = CommandSetEx::new();
        cmds.add_command("wave", None, Default::default())
            .add_existing_command(command!("throw_ninja_star"));

        let expected = CommandSetEx {
            commands: vec![
                command!("wave"),
                command!("throw_ninja_star"),
            ],
        };

        assert_eq!(cmds, expected);
        assert!(cmds.validate().is_ok());
    }

    /// Check `is_empty`
    #[test]
    fn is_empty() {
        // Here, let's double-check that the derive of `Default` for fixed command sets is an empty set
        let cmd_set = CommandSet::default();
        assert!(cmd_set.is_empty());

        let cmd_set = command_set!();
        assert!(cmd_set.is_empty());
        let cmd_set = command_set!([]);
        assert!(cmd_set.is_empty());

        let cmd_set = command_set!([ command!("foo") ]);
        assert!(!cmd_set.is_empty());
    }

    /// Check set type (`CommandSet`/`CommandSetEx`) conversion and comparison
    #[test]
    fn set_types() {
        // Construction of commands/sub-commands to be used in test set
        let subcmd_opts = option_set!(
            @long [
                longopt!("manic"),
            ],
            @short [
                shortopt!('m'),
            ]
        );
        let cmd_opts = option_set!(
            @long [
                longopt!("hammer"),
                longopt!("saw"),
            ],
            @short [
                shortopt!('h'),
            ]
        );
        let cmd_subcmds = command_set!(
            [
                command!("build", @opts &subcmd_opts),
                command!("destroy", @opts &subcmd_opts),
            ]
        );

        // Test set - fixed
        let cmds_fixed = CommandSet {
            commands: &[
                command!("take_a_break"),
                command!("use_tools", @opts &cmd_opts, @cmds cmd_subcmds.clone()),
            ],
        };

        // Test set - extendible
        let cmds_extendible = CommandSetEx {
            commands: vec![
                command!("take_a_break"),
                command!("use_tools", @opts &cmd_opts, @cmds cmd_subcmds.clone()),
            ],
        };

        // Check the two types can be compared
        assert_eq!(true, cmds_fixed.eq(&cmds_extendible));
        assert_eq!(true, cmds_extendible.eq(&cmds_fixed));

        // Check conversions
        let fixed_from_extendible: CommandSet = cmds_extendible.as_fixed();
        assert_eq!(true, cmds_fixed.eq(&fixed_from_extendible));
        assert_eq!(true, cmds_extendible.eq(&fixed_from_extendible));
        assert_eq!(true, fixed_from_extendible.eq(&cmds_fixed));
        assert_eq!(true, fixed_from_extendible.eq(&cmds_extendible));
        let extendible_from_fixed: CommandSetEx = cmds_fixed.to_extendible();
        assert_eq!(true, cmds_fixed.eq(&extendible_from_fixed));
        assert_eq!(true, cmds_extendible.eq(&extendible_from_fixed));
        assert_eq!(true, extendible_from_fixed.eq(&cmds_fixed));
        assert_eq!(true, extendible_from_fixed.eq(&cmds_extendible));

        // Test negative comparisons

        let cmds_fixed_2 = CommandSet {
            commands: &[
                command!("not_available"),
            ],
        };

        let cmds_extendible_2 = CommandSetEx {
            commands: vec![
                command!("not_available"),
            ],
        };

        // Verify not equal
        assert!(cmds_fixed != cmds_fixed_2);
        assert!(cmds_fixed != cmds_extendible_2);
        assert!(cmds_extendible != cmds_fixed_2);
        assert!(cmds_extendible != cmds_extendible_2);
    }
}
