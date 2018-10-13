// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

#[macro_use]
extern crate gong;

mod common;

use gong::*;
use common::*;

/// Tests of "available options set" construction macros
mod available_options {
    use super::*;

    /// Compare macro-built with hand-built "available options" set
    #[test]
    fn cmp_hand_built() {
        let macro_built = gong_option_set!(
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
            ]
        );

        let hand_built = Options {
            long: vec![
                LongOption { name: "help", expects_data: false },
                LongOption { name: "foo", expects_data: false },
                LongOption { name: "version", expects_data: false },
                LongOption { name: "foobar", expects_data: false },
                LongOption { name: "hah", expects_data: true },
                LongOption { name: "ábc", expects_data: false },
            ],
            short: vec![
                ShortOption { ch: 'h', expects_data: false },
                ShortOption { ch: '❤', expects_data: false },
                ShortOption { ch: 'x', expects_data: false },
                ShortOption { ch: 'o', expects_data: true },
                ShortOption { ch: '\u{030A}', expects_data: false },
            ],
            mode: MODE_DEFAULT,
            allow_abbreviations: ABBR_SUP_DEFAULT,
        };

        assert_eq!(macro_built, hand_built);
    }

    /// Compare macro-built with method-built "available options" set
    #[test]
    fn cmp_method_built() {
        let macro_built = gong_option_set!(
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
            ]
        );

        // The common base set is (currently) constructed with the `add_*` methods
        let method_built = common::get_base();

        assert_eq!(macro_built, method_built);
    }

    /// Check capability to specify with and without modes
    #[test]
    fn modes() {
        // With modes
        let opts = gong_option_set!(vec![], vec![], OptionsMode::Alternate, false);
        let cmp = Options {
            long: vec![],
            short: vec![],
            mode: OptionsMode::Alternate,
            allow_abbreviations: false,
        };
        assert_eq!(opts, cmp);

        // Without modes
        let opts = gong_option_set!(vec![], vec![]);
        let cmp = Options {
            long: vec![],
            short: vec![],
            mode: MODE_DEFAULT,
            allow_abbreviations: ABBR_SUP_DEFAULT,
        };
        assert_eq!(opts, cmp);
    }
}
