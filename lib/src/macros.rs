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
/// 1. An array of long options
/// 2. An array of short options
///
/// # Example
///
/// ```rust
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let _ = gong_option_set_fixed!([ gong_longopt!("foo") ], [ gong_shortopt!('a') ]);
/// # }
/// ```
#[macro_export]
macro_rules! gong_option_set_fixed {
    ( $long:tt, $short:tt ) => {
        $crate::options::OptionSet { long: &$long, short: &$short }
    };
    () => { gong_option_set_fixed!([], []) };
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
/// let _ = gong_command_set_fixed!([ gong_command!("foo") ]);
/// # }
/// ```
#[macro_export]
macro_rules! gong_command_set_fixed {
    ( $cmds:tt ) => {
        $crate::commands::CommandSet { commands: &$cmds }
    };
    () => { gong_command_set_fixed!([]) };
}

/// Constructs a [`LongOption`](options/struct.LongOption.html)
///
/// Takes:
///
/// 1. Option name
/// 2. Boolean indicating whether or not it takes a data arg (optional, defaults to false)
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let _ = gong_longopt!("foo");       // A simple option
/// let _ = gong_longopt!("bar", true); // One that takes data
/// # }
/// ```
#[macro_export]
macro_rules! gong_longopt {
    ( $name:expr, $data:expr ) => {
        $crate::options::LongOption { name: $name, expects_data: $data }
    };
    ( $name:expr ) => { $crate::options::LongOption { name: $name, expects_data: false } };
}

/// Constructs a [`ShortOption`](options/struct.ShortOption.html)
///
/// Takes:
///
/// 1. Option char
/// 2. Boolean indicating whether or not it takes a data arg (optional, defaults to false)
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// let _ = gong_shortopt!('a');       // A simple option
/// let _ = gong_shortopt!('b', true); // One that takes data
/// # }
/// ```
#[macro_export]
macro_rules! gong_shortopt {
    ( $ch:expr, $data:expr ) => { $crate::options::ShortOption { ch: $ch, expects_data: $data } };
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
/// let opts = gong_option_set_fixed!();     // An example (empty) option set
/// let subcmds = gong_command_set_fixed!(); // An example (empty) command set
///
/// let _ = gong_command!("foo");
/// let _ = gong_command!("foo", @opts &opts);           // With option set
/// let _ = gong_command!("foo", @cmds subcmds.clone()); // With sub-command set
/// let _ = gong_command!("foo", @opts &opts, @cmds subcmds.clone());
/// # }
/// ```
#[macro_export]
macro_rules! gong_command {
    ( $name:expr ) => {
        $crate::gong_command!($name, @opts $crate::gong_option_set_fixed!(), @cmds $crate::gong_command_set_fixed!())
    };
    ( $name:expr, @opts $opts:expr ) => {
        $crate::gong_command!($name, @opts $opts, @cmds $crate::gong_command_set_fixed!())
    };
    ( $name:expr, @cmds $sub_cmds:expr ) => {
        $crate::gong_command!($name, @opts $crate::gong_option_set_fixed!(), @cmds $sub_cmds)
    };
    ( $name:expr, @opts $opts:expr, @cmds $sub_cmds:expr ) => {
        $crate::commands::Command {
            name: $name,
            options: &$opts,
            sub_commands: $sub_cmds,
        }
    };
}
