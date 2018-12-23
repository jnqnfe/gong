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
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let _ = option_set!(
///     @long [ longopt!("foo") ],
///     @short [ shortopt!('a') ]
/// );
/// # }
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
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let _ = command_set!([ gong::command!("foo") ]);
/// # }
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
/// Takes an option name, optionally annotated with `@data` to indicate a data-taking option.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let _ = longopt!("foo");       // A flag type option
/// let _ = longopt!(@data "bar"); // One that takes data
/// # }
/// ```
#[macro_export]
macro_rules! longopt {
    ( @data $name:expr ) => { $crate::options::LongOption { name: $name, expects_data: true } };
    ( $name:expr ) => { $crate::options::LongOption { name: $name, expects_data: false } };
}

/// Constructs a [`ShortOption`](options/struct.ShortOption.html)
///
/// Takes a `char`, optionally annotated with `@data` to indicate a data-taking option.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let _ = shortopt!('a');       // A flag type option
/// let _ = shortopt!(@data 'b'); // One that takes data
/// # }
/// ```
#[macro_export]
macro_rules! shortopt {
    ( @data $ch:expr ) => { $crate::options::ShortOption { ch: $ch, expects_data: true } };
    ( $ch:expr ) => { $crate::options::ShortOption { ch: $ch, expects_data: false } };
}

/// Constructs a [`Command`](commands/struct.Command.html)
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let opts = option_set!();     // An example (empty) option set
/// let subcmds = command_set!(); // An example (empty) command set
///
/// let _ = command!("foo");
/// let _ = command!("foo", @opts &opts);           // With option set
/// let _ = command!("foo", @cmds subcmds.clone()); // With sub-command set
/// let _ = command!("foo", @opts &opts, @cmds subcmds.clone());
/// # }
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
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let _ = findopt!(@long "help");      // Long option name only
/// let _ = findopt!(@short 'h');        // Short option character only
/// let _ = findopt!(@pair 'h', "help"); // Related short+long pair
/// # }
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
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let _ = foundopt!(@long "help"); // Long option name only
/// let _ = foundopt!(@short 'h');   // Short option character only
/// # }
/// ```
#[macro_export]
macro_rules! foundopt {
    ( @long $name:expr ) => { $crate::analysis::FoundOption::Long($name) };
    ( @short $ch:expr ) => { $crate::analysis::FoundOption::Short($ch) };
}
