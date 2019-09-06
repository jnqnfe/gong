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
use gong::positionals::Policy;

/// A base set of options for common usage in tests
static BASE_OPTS: OptionSet = option_set!(
    @long [
        longopt!(@flag "help"),
        longopt!(@flag "foo"),
        longopt!(@flag "version"),
        longopt!(@flag "foobar"),
        longopt!(@data "hah"),
        longopt!(@flag "ábc"), // Using a combinator char (accent)
        longopt!(@data "ƒƒ"),  // For multi-byte with-data long option component split checking
        longopt!(@flag "ƒo"),  // For multi-byte abbreviation/ambiguity
        longopt!(@mixed "delay"),
        longopt!(@mixed "ǝƃ"),
        longopt!(@flag "color"),
        longopt!(@flag "no-color"),
    ],
    @short [
        shortopt!(@flag 'h'),
        shortopt!(@flag 'v'),
        shortopt!(@flag '❤'),
        shortopt!(@flag 'x'),
        shortopt!(@data 'o'),
        shortopt!(@flag '\u{030a}'), // A lone combinator (“ring above”)
        shortopt!(@data 'Ɛ'),        // For multi-byte with-data calculation checking
        shortopt!(@flag 'C'),        // For analysis data mining, using capital to avoid test conflicts
        shortopt!(@mixed '💧'),
        shortopt!(@mixed 'p'),
    ]
);

/// A base set of commands for common usage in tests
static BASE_CMDS: CommandSet = command_set!([
    command!("foo", @pp Policy::Unlimited), // For command/option name clash testing
    command!("add", @pp Policy::Unlimited),
    command!("commit", @pp Policy::Unlimited),
    command!("push",
        @opts &option_set!(
            @long [
                longopt!(@flag "help"),
                longopt!(@flag "tags"),
            ],
            @short [
                shortopt!(@flag 'h'),
            ]
        ),
        @cmds command_set!([
            command!("origin",
                @opts &option_set!(
                    @long [
                        longopt!(@flag "help"),
                        longopt!(@flag "force"),
                        longopt!(@flag "foo"),
                    ]
                ),
                @pp Policy::Unlimited
            ),
            command!("remote", @pp Policy::Unlimited),
        ]),
        @pp Policy::Fixed(0)
    ),
    command!("branch",
        @opts &option_set!(
            @long [
                longopt!(@flag "help"),
                longopt!(@flag "sorted"),
            ],
            @short [
                shortopt!(@flag 'h'),
            ]
        ),
        @cmds command_set!([
            command!("add", @pp Policy::Unlimited),
            command!("del",
                @opts &option_set!(),
                @cmds command_set!([
                    // Note, the names here are chosen to be different to those below for greater
                    // assurance that a match is made from this set, not the sibling below.
                    command!("locally", @pp Policy::Unlimited),
                    command!("remotely", @pp Policy::Unlimited),
                ]),
                @pp Policy::Fixed(0)
            ),
            command!("list",
                @opts &option_set!(
                    @long [
                        longopt!(@flag "help"),
                        longopt!(@flag "show-current"),
                        longopt!(@flag "foo"),
                    ]
                ),
                @cmds command_set!([
                    command!("local", @pp Policy::Unlimited),
                    command!("remote", @pp Policy::Unlimited),
                ]),
                @pp Policy::Fixed(0)
            ),
        ]),
        @pp Policy::Fixed(0)
    ),
    // For abbreviation ambiguity
    command!("put",
        @opts &option_set!(),
        @cmds command_set!([
            command!("beep", @pp Policy::Unlimited),
            command!("boop", @pp Policy::Unlimited),
        ]),
        @pp Policy::Fixed(0)
    ),
    command!("pull", @pp Policy::Unlimited),
]);

/// Provides a base set of options for common usage in tests
pub fn get_base_opts() -> &'static OptionSet<'static, 'static> {
    &BASE_OPTS
}

/// Provides a base set of commands for common usage in tests
pub fn get_base_cmds() -> &'static CommandSet<'static, 'static> {
    &BASE_CMDS
}
