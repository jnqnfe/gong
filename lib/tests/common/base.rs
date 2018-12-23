// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Base “available” option set used by most tests

use gong::{longopt, shortopt, command, command_set, option_set};
use gong::options::OptionSet;
use gong::commands::CommandSet;

/// A base set of options for common usage in tests
static BASE_OPTS: OptionSet = option_set!(
    @long [
        longopt!("help"),
        longopt!("foo"),
        longopt!("version"),
        longopt!("foobar"),
        longopt!(@data "hah"),
        longopt!("ábc"),       // Using a combinator char (accent)
        longopt!(@data "ƒƒ"),  // For multi-byte with-data long option component split checking
        longopt!("ƒo"),        // For multi-byte abbreviation/ambiguity
        longopt!("color"),
        longopt!("no-color"),
    ],
    @short [
        shortopt!('h'),
        shortopt!('v'),
        shortopt!('❤'),
        shortopt!('x'),
        shortopt!(@data 'o'),
        shortopt!('\u{030a}'), // A lone combinator (“ring above”)
        shortopt!(@data 'Ɛ'),  // For multi-byte with-data calculation checking
        shortopt!('C'),        // For analysis data mining, using capital to avoid test conflicts
    ]
);

/// A base set of commands for common usage in tests
static BASE_CMDS: CommandSet = command_set!([
    command!("foo"), // For command/option name clash testing
    command!("add"),
    command!("commit"),
    command!("push",
        @opts &option_set!(
            @long [
                longopt!("help"),
                longopt!("tags"),
            ],
            @short [
                shortopt!('h'),
            ]
        ),
        @cmds command_set!([
            command!("origin",
                @opts &option_set!(
                    @long [
                        longopt!("help"),
                        longopt!("force"),
                        longopt!("foo"),
                    ]
                )
            ),
            command!("remote"),
        ])
    ),
    command!("branch",
        @opts &option_set!(
            @long [
                longopt!("help"),
                longopt!("sorted"),
            ],
            @short [
                shortopt!('h'),
            ]
        ),
        @cmds command_set!([
            command!("add"),
            command!("del",
                @opts &option_set!(),
                @cmds command_set!([
                    // Note, the names here are chosen to be different to those below for greater
                    // assurance that a match is made from this set, not the sibling below.
                    command!("locally"),
                    command!("remotely"),
                ])
            ),
            command!("list",
                @opts &option_set!(
                    @long [
                        longopt!("help"),
                        longopt!("show-current"),
                        longopt!("foo"),
                    ]
                ),
                @cmds command_set!([
                    command!("local"),
                    command!("remote"),
                ])
            ),
        ])
    ),
]);

/// Provides a base set of options for common usage in tests
pub fn get_base_opts() -> &'static OptionSet<'static, 'static> {
    &BASE_OPTS
}

/// Provides a base set of commands for common usage in tests
pub fn get_base_cmds() -> &'static CommandSet<'static, 'static> {
    &BASE_CMDS
}
