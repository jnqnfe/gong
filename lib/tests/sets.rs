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

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

mod options {
    use gong::options::*;

    /// Check basic valid construction methods
    #[test]
    fn basic() {
        let mut opts = OptionSetEx::new();
        opts.add_short('h')
            .add_short_data('o')
            .add_existing_short(gong_shortopt!('a', false))
            .add_long("foo")
            .add_long_data("bar")
            .add_existing_long(gong_longopt!("foobar", false));

        let expected = OptionSetEx {
            long: vec![
                gong_longopt!("foo", false),
                gong_longopt!("bar", true),
                gong_longopt!("foobar", false),
            ],
            short: vec![
                gong_shortopt!('h', false),
                gong_shortopt!('o', true),
                gong_shortopt!('a', false),
            ],
        };

        assert_eq!(opts, expected);
        assert!(opts.validate().is_ok());
    }

    /// Check `is_empty`
    #[test]
    fn is_empty() {
        // Here, let's double-check that the derive of `Default` for fixed option sets is really an
        // empty set
        let opt_set = OptionSet::default();
        assert!(opt_set.is_empty());

        let opt_set = gong_option_set!();
        assert!(opt_set.is_empty());
        let opt_set = gong_option_set!(@long [], @short []);
        assert!(opt_set.is_empty());
        let opt_set = gong_option_set!(@short [], @long []);
        assert!(opt_set.is_empty());
        let opt_set = gong_option_set!(@long []);
        assert!(opt_set.is_empty());
        let opt_set = gong_option_set!(@short []);
        assert!(opt_set.is_empty());

        let opt_set = gong_option_set!(@long [ gong_longopt!("foo", false) ]);
        assert!(!opt_set.is_empty());

        let opt_set = gong_option_set!(@short [ gong_shortopt!('h', false) ]);
        assert!(!opt_set.is_empty());

        let opt_set = gong_option_set!(
            @long [ gong_longopt!("foo", false) ],
            @short [ gong_shortopt!('h', false) ]
        );
        assert!(!opt_set.is_empty());
    }

    /// Check set type (`OptionSet`/`OptionSetEx`) conversion and comparison
    #[test]
    fn set_types() {
        // Test set - fixed
        let opts_fixed = OptionSet {
            long: &[
                gong_longopt!("foo", false),
                gong_longopt!("bar", true),
                gong_longopt!("foobar", false),
            ],
            short: &[
                gong_shortopt!('h', false),
                gong_shortopt!('o', true),
                gong_shortopt!('a', false),
            ],
        };

        // Test set - extendible
        let opts_extendible = OptionSetEx {
            long: vec![
                gong_longopt!("foo", false),
                gong_longopt!("bar", true),
                gong_longopt!("foobar", false),
            ],
            short: vec![
                gong_shortopt!('h', false),
                gong_shortopt!('o', true),
                gong_shortopt!('a', false),
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
                gong_longopt!("blah", false),
            ],
            short: &[],
        };

        let opts_extendible_2 = OptionSetEx {
            long: vec![
                gong_longopt!("blah", false),
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
        const SHORT_OPT_H: ShortOption = gong_shortopt!('h');
        const LONG_OPT_HELP: LongOption = gong_longopt!("help");

        let _ = gong_option_set!(
            @long [
                LONG_OPT_HELP,
                gong_longopt!("foo", false),
                gong_longopt!("bar", true),
                gong_longopt!("foobar", false),
            ],
            @short [
                SHORT_OPT_H,
                gong_shortopt!('o', true),
                gong_shortopt!('a', false),
            ]
        );
    }
}

mod commands {
    use gong::commands::*;

    /// Check basic valid construction methods
    #[test]
    fn basic() {
        let mut cmds = CommandSetEx::new();
        cmds.add_command("wave", None, Default::default())
            .add_existing_command(gong_command!("throw_ninja_star"));

        let expected = CommandSetEx {
            commands: vec![
                gong_command!("wave"),
                gong_command!("throw_ninja_star"),
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

        let cmd_set = gong_command_set!();
        assert!(cmd_set.is_empty());
        let cmd_set = gong_command_set!([]);
        assert!(cmd_set.is_empty());

        let cmd_set = gong_command_set!([ gong_command!("foo") ]);
        assert!(!cmd_set.is_empty());
    }

    /// Check set type (`CommandSet`/`CommandSetEx`) conversion and comparison
    #[test]
    fn set_types() {
        // Construction of commands/sub-commands to be used in test set
        let subcmd_opts = gong_option_set!(
            @long [
                gong_longopt!("manic"),
            ],
            @short [
                gong_shortopt!('m'),
            ]
        );
        let cmd_opts = gong_option_set!(
            @long [
                gong_longopt!("hammer"),
                gong_longopt!("saw"),
            ],
            @short [
                gong_shortopt!('h'),
            ]
        );
        let cmd_subcmds = gong_command_set!(
            [
                gong_command!("build", @opts &subcmd_opts),
                gong_command!("destroy", @opts &subcmd_opts),
            ]
        );

        // Test set - fixed
        let cmds_fixed = CommandSet {
            commands: &[
                gong_command!("take_a_break"),
                gong_command!("use_tools", @opts &cmd_opts, @cmds cmd_subcmds.clone()),
            ],
        };

        // Test set - extendible
        let cmds_extendible = CommandSetEx {
            commands: vec![
                gong_command!("take_a_break"),
                gong_command!("use_tools", @opts &cmd_opts, @cmds cmd_subcmds.clone()),
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
                gong_command!("not_available"),
            ],
        };

        let cmds_extendible_2 = CommandSetEx {
            commands: vec![
                gong_command!("not_available"),
            ],
        };

        // Verify not equal
        assert!(cmds_fixed != cmds_fixed_2);
        assert!(cmds_fixed != cmds_extendible_2);
        assert!(cmds_extendible != cmds_fixed_2);
        assert!(cmds_extendible != cmds_extendible_2);
    }
}
