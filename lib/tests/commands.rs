// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Testing “command argument” construction

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use gong::commands::*;

/// An empty string is not a valid command name property
mod no_name {
    use super::*;

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add() {
        let mut cmds = CommandSetEx::new();
        cmds.add_command("", None, Default::default()); // Should panic here in debug mode!
    }

    #[test]
    fn add_existing() {
        let mut cmds = CommandSetEx::new();
        cmds.add_existing_command(gong_command!("")); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    fn invalid_set() {
        let cmds = gong_command_set_fixed!([
            gong_command!("foo"),
            gong_command!(""),
            gong_command!("bar"),
        ]);
        assert_eq!(false, cmds.is_valid());
        assert_eq!(cmds.validate(), Err(vec![ CommandFlaw::EmptyName ]));
    }
}

/// Command names cannot contain the unicode replacement character (`\u{FFFD}`). If it were allowed,
/// it would allow incorrect analysis with `OsStr` based processing.
mod rep_char {
    use super::*;

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add() {
        let mut cmds = CommandSetEx::new();
        cmds.add_command("a\u{FFFD}b", None, Default::default()); // Should panic here in debug mode!
    }

    #[test]
    fn add_existing() {
        let mut cmds = CommandSetEx::new();
        cmds.add_existing_command(gong_command!("\u{FFFD}")); // Should work, no validation done
    }

    /// Bypassing add methods, check validation fails
    #[test]
    fn invalid_set() {
        let cmds = gong_command_set_fixed!([
            gong_command!("foo"),
            gong_command!("a\u{FFFD}b"),
            gong_command!("bar"),
        ]);
        assert_eq!(false, cmds.is_valid());
        assert_eq!(cmds.validate(), Err(vec![ CommandFlaw::NameIncludesRepChar("a\u{FFFD}b") ]));
    }
}

/// Duplicates pose a potential problem due to potential for confusion over differing `sub-command`
/// and `command-option` attributes. They also can result from option name-clashing bugs with
/// programs that dynamically generate (large) option sets. An option set containing duplicates is
/// thus considered invalid.
#[test]
fn duplicates() {
    let mut cmds = CommandSetEx::new();
    cmds.add_command("aaa", None, Default::default())
        .add_command("bbb", None, Default::default())
        .add_command("ccc", None, Default::default())
        .add_command("ccc", None, Default::default())      // dup
        .add_command("eee", None, Default::default())
        .add_command("bbb", None, Default::default());     // dup
    assert_eq!(false, cmds.is_valid());
    assert_eq!(cmds.validate(), Err(vec![
        CommandFlaw::Dup("ccc"),
        CommandFlaw::Dup("bbb"),
    ]));
}
