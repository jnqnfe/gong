// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

/// Constructs an [`OptionSet`](options/struct.OptionSet.html)
///
/// Takes:
///
/// 1. An optional array of long options, annotated with `@long`
/// 2. An optional array of short options, annotated with `@short`
///
/// The macro, thanks to the required annotations, is flexible, allowing one, both, or neither set
/// to be provided, and in either order.
///
/// # Example
///
/// ```rust
/// let _ = gong::option_set!(
///     @long [ gong::longopt!(@flag "foo") ],
///     @short [ gong::shortopt!(@flag 'a') ]
/// );
/// ```
#[macro_export]
macro_rules! option_set {
    ( @long $long:tt, @short $short:tt ) => {
        $crate::options::OptionSet { long: &$long, short: &$short }
    };
    ( @short $short:tt, @long $long:tt ) => { $crate::option_set!(@long $long, @short $short) };
    ( @long $long:tt ) => { $crate::option_set!(@long $long, @short []) };
    ( @short $short:tt ) => { $crate::option_set!(@long [], @short $short) };
    () => { $crate::option_set!(@long [], @short []) };
}

/// Constructs a [`CommandSet`](commands/struct.CommandSet.html)
///
/// Takes an array of commands
///
/// # Example
///
/// ```rust
/// let _ = gong::command_set!([ gong::command!("foo") ]);
/// ```
#[macro_export]
macro_rules! command_set {
    ( $cmds:tt ) => {
        $crate::commands::CommandSet { commands: &$cmds }
    };
    () => { $crate::command_set!([]) };
}

/// Constructs a [`LongOption`](options/struct.LongOption.html)
///
/// Takes an option name after one of the following annotations:
///
///  * `@flag` to indicate a flag type option, i.e. one which does not take a data value.
///  * `@data` to indicate a data-taking option.
///
/// See the [options documentation](docs/options/index.html) for discussion of the differences.
///
/// # Examples
///
/// ```rust
/// let _ = gong::longopt!(@flag "foo"); // A flag type option
/// let _ = gong::longopt!(@data "bar"); // One that takes data
/// ```
#[macro_export]
macro_rules! longopt {
    ( @data $name:expr ) => { $crate::options::LongOption { name: $name, expects_data: true } };
    ( @flag $name:expr ) => { $crate::options::LongOption { name: $name, expects_data: false } };
}

/// Constructs a [`ShortOption`](options/struct.ShortOption.html)
///
/// Takes a `char` after one of the following annotations:
///
///  * `@flag` to indicate a flag type option, i.e. one which does not take a data value.
///  * `@data` to indicate a data-taking option.
///
/// See the [options documentation](docs/options/index.html) for discussion of the differences.
///
/// # Examples
///
/// ```rust
/// let _ = gong::shortopt!(@flag 'a'); // A flag type option
/// let _ = gong::shortopt!(@data 'b'); // One that takes data
/// ```
#[macro_export]
macro_rules! shortopt {
    ( @data $ch:expr ) => { $crate::options::ShortOption { ch: $ch, expects_data: true } };
    ( @flag $ch:expr ) => { $crate::options::ShortOption { ch: $ch, expects_data: false } };
}

/// Constructs a [`Command`](commands/struct.Command.html)
///
/// # Examples
///
/// ```rust
/// let opts = gong::option_set!();     // An example (empty) option set
/// let subcmds = gong::command_set!(); // An example (empty) command set
///
/// let _ = gong::command!("foo");
/// let _ = gong::command!("foo", @opts &opts);           // With option set
/// let _ = gong::command!("foo", @cmds subcmds.clone()); // With sub-command set
/// let _ = gong::command!("foo", @opts &opts, @cmds subcmds.clone());
/// ```
#[macro_export]
macro_rules! command {
    ( $name:expr ) => {
        $crate::command!($name, @opts $crate::option_set!(), @cmds $crate::command_set!())
    };
    ( $name:expr, @opts $opts:expr ) => {
        $crate::command!($name, @opts $opts, @cmds $crate::command_set!())
    };
    ( $name:expr, @cmds $sub_cmds:expr ) => {
        $crate::command!($name, @opts $crate::option_set!(), @cmds $sub_cmds)
    };
    ( $name:expr, @opts $opts:expr, @cmds $sub_cmds:expr ) => {
        $crate::commands::Command {
            name: $name,
            options: &$opts,
            sub_commands: $sub_cmds,
        }
    };
}

/// Constructs a [`FindOption`](analysis/enum.FindOption.html)
///
/// Takes either a long option name, a short option character, or both as a related pair. All must
/// be annotated as appropriate to indicate which form.
///
/// # Examples
///
/// ```rust
/// let _ = gong::findopt!(@long "help");      // Long option name only
/// let _ = gong::findopt!(@short 'h');        // Short option character only
/// let _ = gong::findopt!(@pair 'h', "help"); // Related short+long pair
/// ```
#[macro_export]
macro_rules! findopt {
    ( @long $name:expr ) => { $crate::analysis::FindOption::Long($name) };
    ( @short $ch:expr ) => { $crate::analysis::FindOption::Short($ch) };
    ( @pair $ch:expr, $name:expr ) => { $crate::analysis::FindOption::Pair($ch, $name) };
}

/// Constructs a [`FoundOption`](analysis/enum.FoundOption.html)
///
/// Takes either a long option name or a short option character. Both must be annotated as
/// appropriate to indicate which form.
///
/// # Examples
///
/// ```rust
/// let _ = gong::foundopt!(@long "help"); // Long option name only
/// let _ = gong::foundopt!(@short 'h');   // Short option character only
/// ```
#[macro_export]
macro_rules! foundopt {
    ( @long $name:expr ) => { $crate::analysis::FoundOption::Long($name) };
    ( @short $ch:expr ) => { $crate::analysis::FoundOption::Short($ch) };
}
