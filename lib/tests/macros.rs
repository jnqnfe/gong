// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
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
use common::*;

/// Tests of “available options set” construction macros
mod available_options {
    use super::*;

    /// Compare the two option set building macros
    #[test]
    fn cmp_non_fixed() {
        // The common base set is already constructed with the “fixed” (`OptionSet` based) macro
        let fixed: &OptionSet = common::get_base();

        // Re-build with “non-fixed” (`OptionSetEx` based) macro
        let non_fixed: OptionSetEx = gong_option_set!(
            vec![
                gong_longopt!("help"),
                gong_longopt!("foo"),
                gong_longopt!("version"),
                gong_longopt!("foobar"),
                gong_longopt!("hah", true),
                gong_longopt!("ábc"),
                gong_longopt!("ƒƒ", true),
            ],
            vec![
                gong_shortopt!('h'),
                gong_shortopt!('❤'),
                gong_shortopt!('x'),
                gong_shortopt!('o', true),
                gong_shortopt!('\u{030a}'),
                gong_shortopt!('Ɛ', true),
            ],
            OptionsMode::Standard,
            true
        );
        assert_eq!(*fixed, non_fixed);
    }

    /// Compare macro-built with hand-built “available options” set
    #[test]
    fn cmp_hand_built() {
        // The common base set is already constructed with a macro
        let macro_built = common::get_base();

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
            ],
            short: &[
                ShortOption { ch: 'h', expects_data: false },
                ShortOption { ch: '❤', expects_data: false },
                ShortOption { ch: 'x', expects_data: false },
                ShortOption { ch: 'o', expects_data: true },
                ShortOption { ch: '\u{030A}', expects_data: false },
                ShortOption { ch: 'Ɛ', expects_data: true },
            ],
            mode: MODE_DEFAULT,
            allow_abbreviations: ABBR_SUP_DEFAULT,
        };

        assert_eq!(*macro_built, hand_built);
    }

    /// Compare macro-built with method-built “available options” set
    #[test]
    fn cmp_method_built() {
        // The common base set is already constructed with a macro
        let macro_built = common::get_base();

        // Re-build it with methods for comparison
        let mut method_built = OptionSetEx::new(6, 5);
        method_built
            .add_long("help")
            .add_short('h')
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
            .add_short_data('Ɛ');

        assert_eq!(*macro_built, method_built);
    }

    /// Check capability to specify with and without modes
    #[test]
    fn modes() {
        // With modes
        let opts = gong_option_set_fixed!([], [], OptionsMode::Alternate, false);
        let opts_ex = gong_option_set!(vec![], vec![], OptionsMode::Alternate, false);
        let cmp = OptionSet {
            long: &[],
            short: &[],
            mode: OptionsMode::Alternate,
            allow_abbreviations: false,
        };
        assert_eq!(opts, cmp);
        assert_eq!(opts_ex, cmp);

        // Without modes
        let opts = gong_option_set_fixed!([], []);
        let opts_ex = gong_option_set!(vec![], vec![]);
        let cmp = OptionSet {
            long: &[],
            short: &[],
            mode: MODE_DEFAULT,
            allow_abbreviations: ABBR_SUP_DEFAULT,
        };
        assert_eq!(opts, cmp);
        assert_eq!(opts_ex, cmp);
    }
}
