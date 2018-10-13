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

// NB: These are not publically accessible from the crate, so we duplicate them. We do have a test
// to ensure they are correct however!
pub const ABBR_SUP_DEFAULT: bool = true;
pub const MODE_DEFAULT: OptionsMode = OptionsMode::Standard;

/// Provides a base set of options for common usage in tests
pub fn get_base<'a>() -> Options<'a> {
    let mut opts = Options::new(6, 5);
    opts.add_long("help")
        .add_short('h')
        .add_long("foo")
        .add_long("version")
        .add_long("foobar")
        .add_long_data("hah")
        .add_long("ábc")        // Using a combinator char (accent)
        .add_short('❤')
        .add_short('x')
        .add_short_data('o')
        .add_short('\u{030A}'); // A lone combinator ("ring above")
    opts
}
