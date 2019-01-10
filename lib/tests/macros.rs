// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
mod common;

use gong::{longopt, shortopt};
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
                longopt!(@flag "help"),
                longopt!(@flag "foo"),
                longopt!(@flag "version"),
                longopt!(@flag "foobar"),
                longopt!(@data "hah"),
                longopt!(@flag "ábc"),
                longopt!(@data "ƒƒ"),
                longopt!(@flag "ƒo"),
                longopt!(@opt_data "delay"),
                longopt!(@opt_data "ǝƃ"),
                longopt!(@flag "color"),
                longopt!(@flag "no-color"),
            ],
            short: vec![
                shortopt!(@flag 'h'),
                shortopt!(@flag 'v'),
                shortopt!(@flag '❤'),
                shortopt!(@flag 'x'),
                shortopt!(@data 'o'),
                shortopt!(@flag '\u{030a}'),
                shortopt!(@data 'Ɛ'),
                shortopt!(@flag 'C'),
                shortopt!(@opt_data '💧'),
                shortopt!(@opt_data 'p'),
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
                LongOption { name: "help", opt_type: OptionType::Flag },
                LongOption { name: "foo", opt_type: OptionType::Flag },
                LongOption { name: "version", opt_type: OptionType::Flag },
                LongOption { name: "foobar", opt_type: OptionType::Flag },
                LongOption { name: "hah", opt_type: OptionType::Data },
                LongOption { name: "ábc", opt_type: OptionType::Flag },
                LongOption { name: "ƒƒ", opt_type: OptionType::Data },
                LongOption { name: "ƒo", opt_type: OptionType::Flag },
                LongOption { name: "delay", opt_type: OptionType::OptionalData },
                LongOption { name: "ǝƃ", opt_type: OptionType::OptionalData },
                LongOption { name: "color", opt_type: OptionType::Flag },
                LongOption { name: "no-color", opt_type: OptionType::Flag },
            ],
            short: &[
                ShortOption { ch: 'h', opt_type: OptionType::Flag },
                ShortOption { ch: 'v', opt_type: OptionType::Flag },
                ShortOption { ch: '❤', opt_type: OptionType::Flag },
                ShortOption { ch: 'x', opt_type: OptionType::Flag },
                ShortOption { ch: 'o', opt_type: OptionType::Data },
                ShortOption { ch: '\u{030A}', opt_type: OptionType::Flag },
                ShortOption { ch: 'Ɛ', opt_type: OptionType::Data },
                ShortOption { ch: 'C', opt_type: OptionType::Flag },
                ShortOption { ch: '💧', opt_type: OptionType::OptionalData },
                ShortOption { ch: 'p', opt_type: OptionType::OptionalData },
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
            .add_long_data_optional("delay")
            .add_long_data_optional("ǝƃ")
            .add_long("color")
            .add_long("no-color")
            .add_short('C')
            .add_short_data_optional('💧')
            .add_short_data_optional('p');

        assert_eq!(*macro_built, method_built);
    }
}
