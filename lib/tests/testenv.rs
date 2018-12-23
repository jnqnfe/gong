// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Testing common components of the test environment

extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
mod common;

use gong::parser::{Settings, OptionsMode};
use self::common::{get_parser, get_base_opts, get_base_cmds};

/// Checks default settings match those expected. If they change, we need to update the test suite.
#[test]
fn check_default_settings() {
    let mut expected = Settings::default();
    expected.set_mode(OptionsMode::Standard);
    expected.set_allow_abbreviations(true);
    assert_eq!(expected, Default::default());
}

/// Check common base “available options” set validity
///
/// Doing it once here allows avoiding inefficiently doing so in every test using it (without
/// modification).
mod base_opt_set {
    use super::*;

    #[test]
    fn is_valid() {
        let opts = get_base_opts();
        assert!(opts.is_valid());
        assert_eq!(opts.validate(), Ok(()));
    }

    /// Double check inserting a problem does break it
    #[test]
    #[should_panic]
    fn can_break() {
        let mut opts = get_base_opts().to_extendible();
        opts.add_long("foo"); // Duplicate - should panic here in debug mode!
        assert!(opts.is_valid());
        assert_eq!(opts.validate(), Ok(()));
    }
}

/// Check common base “available command” set validity
///
/// Doing it once here allows avoiding inefficiently doing so in every test using it (without
/// modification).
mod base_cmd_set {
    use super::*;

    #[test]
    fn is_valid() {
        let cmds = get_base_cmds();
        assert!(cmds.is_valid());
        assert_eq!(cmds.validate(), Ok(()));
    }

    /// Double check inserting a problem does break it
    #[test]
    #[should_panic]
    fn can_break() {
        let mut cmds = get_base_cmds().to_extendible();
        cmds.add_command("foo", None, Default::default()); // Duplicate - should panic here in debug mode!
        assert!(cmds.is_valid());
        assert_eq!(cmds.validate(), Ok(()));
    }
}

/// Check a `Parser` combining the base option set and the base command set is valid (not reason
/// for it not to be if they are individually).
#[test]
fn parser() {
    assert!(get_parser().is_valid());
    assert_eq!(get_parser().validate(), Ok(()));
}
