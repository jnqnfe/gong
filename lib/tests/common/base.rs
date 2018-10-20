// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Base "available" option set used by most tests

use gong::options::{Options, OptionsMode};

/// Provides a base set of options for common usage in tests
pub fn get_base<'a>() -> Options<'a> {
    // Note, the macro tests were written with the expectation that this function is constructing
    // this base set using the macros. Thus if this were changed (though there is no conceivable
    // reason to) to not use the macros, that needs addressing.
    gong_option_set!(
        vec![
            gong_longopt!("help"),
            gong_longopt!("foo"),
            gong_longopt!("version"),
            gong_longopt!("foobar"),
            gong_longopt!("hah", true),
            gong_longopt!("ábc"),       // Using a combinator char (accent)
            gong_longopt!("ƒƒ", true),  // For multi-byte with-data long option component split checking
        ],
        vec![
            gong_shortopt!('h'),
            gong_shortopt!('❤'),
            gong_shortopt!('x'),
            gong_shortopt!('o', true),
            gong_shortopt!('\u{030a}'), // A lone combinator ("ring above")
            gong_shortopt!('Ɛ', true),  // For multi-byte with-data calculation checking
        ],
        OptionsMode::Standard,
        true
    )
}
