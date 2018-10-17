// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! This file holds no tests itself, it is used as a submodule for other test files, providing a
//! common base set of options to work with.

use gong::*;

//TODO: now that processing accepts both `String` and `&str` type "available option" sets, cover both somehow
/// Used for cleaner creation of set of test arguments
///
/// Note, arguments will normally be obtained from the environment, and Rust provides these to use
/// as String objects, not &str, hence why we create a vector of Strings!
#[macro_export]
macro_rules! arg_list {
    ( $($e:expr),+ ) => {
        vec![$(String::from($e)),+]
    };
    ( $($e:expr,)+ ) => {
        vec![$(String::from($e)),+]
    };
}

// NB: These are not publically accessible from the crate, so we duplicate them. We do have a test
// to ensure they are correct however!
pub const ABBR_SUP_DEFAULT: bool = true;
pub const MODE_DEFAULT: OptionsMode = OptionsMode::Standard;

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
            gong_longopt!("ábc"),        // Using a combinator char (accent)
        ],
        vec![
            gong_shortopt!('h'),
            gong_shortopt!('❤'),
            gong_shortopt!('x'),
            gong_shortopt!('o', true),
            gong_shortopt!('\u{030A}'),   // A lone combinator ("ring above")
        ],
        OptionsMode::Standard,
        true
    )
}
