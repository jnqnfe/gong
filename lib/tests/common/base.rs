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

use gong::options::OptionSet;
use gong::commands::CommandSet;

/// A base set of options for common usage in tests
static BASE_OPTS: OptionSet = gong_option_set!(
    @long [
        gong_longopt!("help"),
        gong_longopt!("foo"),
        gong_longopt!("version"),
        gong_longopt!("foobar"),
        gong_longopt!("hah", true),
        gong_longopt!("ábc"),       // Using a combinator char (accent)
        gong_longopt!("ƒƒ", true),  // For multi-byte with-data long option component split checking
        gong_longopt!("ƒo"),        // For multi-byte abbreviation/ambiguity
    ],
    @short [
        gong_shortopt!('h'),
        gong_shortopt!('❤'),
        gong_shortopt!('x'),
        gong_shortopt!('o', true),
        gong_shortopt!('\u{030a}'), // A lone combinator (“ring above”)
        gong_shortopt!('Ɛ', true),  // For multi-byte with-data calculation checking
    ]
);

/// A base set of commands for common usage in tests
static BASE_CMDS: CommandSet = gong_command_set!([
    gong_command!("foo"), // For command/option name clash testing
    gong_command!("add"),
    gong_command!("commit"),
    gong_command!("push",
        @opts &gong_option_set!(
            @long [
                gong_longopt!("help"),
                gong_longopt!("tags"),
            ],
            @short [
                gong_shortopt!('h'),
            ]
        ),
        @cmds gong_command_set!([
            gong_command!("origin",
                @opts &gong_option_set!(
                    @long [
                        gong_longopt!("help"),
                        gong_longopt!("force"),
                        gong_longopt!("foo"),
                    ]
                )
            ),
            gong_command!("remote"),
        ])
    ),
    gong_command!("branch",
        @opts &gong_option_set!(
            @long [
                gong_longopt!("help"),
                gong_longopt!("sorted"),
            ],
            @short [
                gong_shortopt!('h'),
            ]
        ),
        @cmds gong_command_set!([
            gong_command!("add"),
            gong_command!("del",
                @opts &gong_option_set!(),
                @cmds gong_command_set!([
                    // Note, the names here are chosen to be different to those below for greater
                    // assurance that a match is made from this set, not the sibling below.
                    gong_command!("locally"),
                    gong_command!("remotely"),
                ])
            ),
            gong_command!("list",
                @opts &gong_option_set!(
                    @long [
                        gong_longopt!("help"),
                        gong_longopt!("show-current"),
                        gong_longopt!("foo"),
                    ]
                ),
                @cmds gong_command_set!([
                    gong_command!("local"),
                    gong_command!("remote"),
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
