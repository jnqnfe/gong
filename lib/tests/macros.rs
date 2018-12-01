// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
mod common;

use gong::options::*;

/// Tests of “available options set” construction macros
mod available_options {
    use super::*;

    /// Compare the two option set building macros
    #[test]
    fn cmp_non_fixed() {
        // The common base set is already constructed with the “fixed” (`OptionSet` based) macro
        let fixed: &OptionSet = common::get_base_opts();

        // Re-build with “non-fixed” (`OptionSetEx` based) macro
        let non_fixed = OptionSetEx {
            long: vec![
                gong_longopt!("help"),
                gong_longopt!("foo"),
                gong_longopt!("version"),
                gong_longopt!("foobar"),
                gong_longopt!(@data "hah"),
                gong_longopt!("ábc"),
                gong_longopt!(@data "ƒƒ"),
                gong_longopt!("ƒo"),
                gong_longopt!("color"),
                gong_longopt!("no-color"),
            ],
            short: vec![
                gong_shortopt!('h'),
                gong_shortopt!('v'),
                gong_shortopt!('❤'),
                gong_shortopt!('x'),
                gong_shortopt!(@data 'o'),
                gong_shortopt!('\u{030a}'),
                gong_shortopt!(@data 'Ɛ'),
                gong_shortopt!('C'),
            ]
        };
        assert_eq!(*fixed, non_fixed);
    }

    /// Compare macro-built with hand-built “available options” set
    #[test]
    fn cmp_hand_built() {
        // The common base set is already constructed with a macro
        let macro_built = common::get_base_opts();

        // Re-build it by hand for comparison
        let hand_built = OptionSet {
            long: &[
                LongOption { name: "help", expects_data: false },
                LongOption { name: "foo", expects_data: false },
                LongOption { name: "version", expects_data: false },
                LongOption { name: "foobar", expects_data: false },
                LongOption { name: "hah", expects_data: true },
                LongOption { name: "ábc", expects_data: false },
                LongOption { name: "ƒƒ", expects_data: true },
                LongOption { name: "ƒo", expects_data: false },
                LongOption { name: "color", expects_data: false },
                LongOption { name: "no-color", expects_data: false },
            ],
            short: &[
                ShortOption { ch: 'h', expects_data: false },
                ShortOption { ch: 'v', expects_data: false },
                ShortOption { ch: '❤', expects_data: false },
                ShortOption { ch: 'x', expects_data: false },
                ShortOption { ch: 'o', expects_data: true },
                ShortOption { ch: '\u{030A}', expects_data: false },
                ShortOption { ch: 'Ɛ', expects_data: true },
                ShortOption { ch: 'C', expects_data: false },
            ]
        };

        assert_eq!(*macro_built, hand_built);
    }

    /// Compare macro-built with method-built “available options” set
    #[test]
    fn cmp_method_built() {
        // The common base set is already constructed with a macro
        let macro_built = common::get_base_opts();

        // Re-build it with methods for comparison
        let mut method_built = OptionSetEx::with_capacity(6, 5);
        method_built
            .add_long("help")
            .add_short('h')
            .add_short('v')
            .add_long("foo")
            .add_long("version")
            .add_long("foobar")
            .add_long_data("hah")
            .add_long("ábc")
            .add_short('❤')
            .add_short('x')
            .add_short_data('o')
            .add_short('\u{030A}')
            .add_long_data("ƒƒ")
            .add_long("ƒo")
            .add_short_data('Ɛ')
            .add_long("color")
            .add_long("no-color")
            .add_short('C');

        assert_eq!(*macro_built, method_built);
    }
}
