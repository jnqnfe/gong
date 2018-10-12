// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Unit tests

use super::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Available options validation
////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod available_options {
    use super::*;

    /* Dash ('-') is an invalid short option (clashes with early terminator if it were given on its
     * own (`--`), and would be misinterpreted as a long option if given as the first in a short
     * option set (`--abc`)). */

    /// Check `ShortOption::new` rejects '-'
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_short_dash() {
        let _opt = ShortOption::new('-', false); // Should panic here in debug mode!
    }

    /// Check `LongOption::new` rejects empty string
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_no_name() {
        let _opt = LongOption::new("", false); // Should panic here in debug mode!
    }

    /* Long option names cannot contain an '=' (used for declaring a data sub-argument in the same
     * argument; if names could contain an '=', as data can, we would not know where to do the
     * split, complicating matching. */

    /// Check `LongOption::new` rejects equals ('=') char
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_with_equals() {
        let _opt = LongOption::new("a=b", false); // Should panic here in debug mode!
    }
}
