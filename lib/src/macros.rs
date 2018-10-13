// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

/// Construct an [`Options`](struct.Options.html)
///
/// Takes:
///
/// 1. A `Vec` of long options
/// 2. A `Vec` of short options
/// 3. Mode
/// 4. Abbreviation support state
///
/// The last two can be left off if both defaults.
///
/// Example:
///
/// ```rust
/// # #[macro_use]
/// # extern crate gong;
/// # fn main() {
/// // Without modes, empty option lists
/// let _ = gong_option_set!(vec![], vec![]);
/// // With modes, empty option lists
/// let _ = gong_option_set!(vec![], vec![], gong::OptionsMode::Standard, true);
/// # }
/// ```
#[macro_export]
macro_rules! gong_option_set {
    ( $long:expr, $short:expr, $mode:expr, $abbr:expr ) => {
        $crate::Options { long: $long, short: $short, mode: $mode, allow_abbreviations: $abbr }
    };
    ( $long:expr, $short:expr ) => {
        $crate::Options {
            long: $long, short: $short, mode: $crate::OptionsMode::Standard,
            allow_abbreviations: true
        }
    };
}

/// Construct a [`LongOption`](struct.LongOption.html)
///
/// Takes:
///
/// 1. Option name
/// 2. Boolean indicating whether or not it takes a data arg (optional, defaults to false)
#[macro_export]
macro_rules! gong_longopt {
    ( $name:expr, $data:expr ) => { $crate::LongOption { name: $name, expects_data: $data } };
    ( $name:expr ) => { $crate::LongOption { name: $name, expects_data: false } };
}

/// Construct a [`ShortOption`](struct.ShortOption.html)
///
/// Takes:
///
/// 1. Option char
/// 2. Boolean indicating whether or not it takes a data arg (optional, defaults to false)
#[macro_export]
macro_rules! gong_shortopt {
    ( $ch:expr, $data:expr ) => { $crate::ShortOption { ch: $ch, expects_data: $data } };
    ( $ch:expr ) => { $crate::ShortOption { ch: $ch, expects_data: false } };
}
