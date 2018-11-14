// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Base “available” option set used by most tests

use gong::options::OptionSet;

/// A base set of options for common usage in tests
static BASE_OPTS: OptionSet = gong_option_set_fixed!(
    [
        gong_longopt!("help"),
        gong_longopt!("foo"),
        gong_longopt!("version"),
        gong_longopt!("foobar"),
        gong_longopt!("hah", true),
        gong_longopt!("ábc"),       // Using a combinator char (accent)
        gong_longopt!("ƒƒ", true),  // For multi-byte with-data long option component split checking
        gong_longopt!("ƒo"),        // For multi-byte abbreviation/ambiguity
    ],
    [
        gong_shortopt!('h'),
        gong_shortopt!('❤'),
        gong_shortopt!('x'),
        gong_shortopt!('o', true),
        gong_shortopt!('\u{030a}'), // A lone combinator (“ring above”)
        gong_shortopt!('Ɛ', true),  // For multi-byte with-data calculation checking
    ]
);

/// Provides a base set of options for common usage in tests
pub fn get_base() -> &'static OptionSet<'static, 'static> {
    &BASE_OPTS
}
