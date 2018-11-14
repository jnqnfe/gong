// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! The parser & parser settings
//!
//! The `Parser` wraps a description of a collection of available program [*options*][options] and
//! [*command arguments*][commands], along with parser settings, and provides methods that take a
//! list of input arguments to parse, and returns an analysis.
//!
//! The `Parser` has methods for parsing input arguments in both of the following forms:
//!
//!  - `AsRef<str>` (`String` and `&str`)
//!  - `AsRef<OsStr>` (`OsString` and `&OsStr`)
//!
//! Most programs will only want the former, however there are cases where the latter is
//! wanted/needed.
//!
//! The `Parser` also has settings to control certain aspects of parsing, for instance to choose
//! which mode to parse options in (*standard* or *alternate*; the difference of which is discussed
//! in the [option support documentation][options]), and whether or not to allow abbreviated *long
//! option* name matching.
//!
//! [commands]: ../docs/commands/index.html
//! [options]: ../docs/options/index.html

use std::convert::AsRef;
use std::ffi::OsStr;
use super::commands::{CommandSet, CommandFlaw};
use super::options::{OptionSet, OptionFlaw};

/// Default abbreviation support state
pub(crate) const ABBR_SUP_DEFAULT: bool = true;
/// Default mode
pub(crate) const MODE_DEFAULT: OptionsMode = OptionsMode::Standard;

/// Parser
///
/// Holds the option set and command set used for parsing, along with parser settings, and provides
/// the parse method for parsing an argument set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser<'r, 's: 'r> {
    /* NOTE: these have been left public to allow efficient static creation */
    /// The main (top level) option set
    pub options: &'r OptionSet<'r, 's>,
    /// Command set
    pub commands: &'r CommandSet<'r, 's>,
    /// Settings
    pub settings: Settings,
}

impl<'r, 's: 'r> Default for Parser<'r, 's> {
    fn default() -> Self {
        Self {
            options: &gong_option_set_fixed!(),
            commands: &gong_command_set_fixed!(),
            settings: Settings::default(),
        }
    }
}

/// Settings for parser
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Option processing mode to use
    pub mode: OptionsMode,
    /// Whether or not to allow abbreviated longoption name matching
    pub allow_abbreviations: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: MODE_DEFAULT,
            allow_abbreviations: ABBR_SUP_DEFAULT,
        }
    }
}

/// Used to specify which option processing mode to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionsMode {
    /// Standard (default): Short (`-o`) and long (`--foo`) options, with single and double dash
    /// prefixes respectively.
    Standard,
    /// Alternate: Long options only, with single dash prefix.
    Alternate,
}

impl Default for OptionsMode {
    fn default() -> Self {
        MODE_DEFAULT
    }
}

impl Settings {
    /// Set mode
    #[inline(always)]
    pub fn set_mode(&mut self, mode: OptionsMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Enable/disable long option name abbreviated matching
    #[inline(always)]
    pub fn set_allow_abbreviations(&mut self, allow: bool) -> &mut Self {
        self.allow_abbreviations = allow;
        self
    }
}

impl<'r, 's: 'r> Parser<'r, 's> {
    /// Create a new parser
    pub fn new(options: &'r OptionSet<'r, 's>, commands: Option<&'r CommandSet<'r, 's>>) -> Self {
        Self {
            options: options,
            commands: commands.unwrap_or(&gong_command_set_fixed!()),
            settings: Settings::default(),
        }
    }

    /// Get a mutable reference to settings
    #[inline(always)]
    pub fn settings(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Checks validity of the option set and command set
    ///
    /// Returns `true` if valid.
    ///
    /// See also the [`validate`](#method.validate) method.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.options.is_valid() && self.commands.is_valid()
    }

    /// Checks validity of the option set and command set, returning details of any problems
    ///
    /// If any flaws are found, a tuple is returned, wrapped in `Err`. The first item in the tuple
    /// is the `Vec` of flaws for the main option set, and the second is the flaws for the command
    /// set.
    pub fn validate(&self) -> Result<(), (Vec<OptionFlaw<'s>>, Vec<CommandFlaw<'s>>)> {
        let option_set_flaws = self.options.validate();
        let command_set_flaws = self.commands.validate();
        match (option_set_flaws, command_set_flaws) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(f)) => Err((Vec::new(), f)),
            (Err(f), Ok(_)) => Err((f, Vec::new())),
            (Err(f1), Err(f2)) => Err((f1, f2)),
        }
    }

    /// Parses provided program arguments
    ///
    /// Returns an analysis describing the parsed argument list. This may include `&str` references
    /// to strings provided in the `args` parameter and in `self`. Take note of this with respect to
    /// object lifetimes.
    ///
    /// Expects `self` to be valid (see [`is_valid`](#method.is_valid)).
    #[inline(always)]
    pub fn parse<A>(&self, args: &'s [A]) -> super::analysis::Analysis<'s, str>
        where A: 's + AsRef<str>
    {
        super::engine::process(args, self)
    }

    /// Parses provided program arguments, given as `OsStr`
    ///
    /// Returns an analysis describing the parsed argument list. This may include `&str` references
    /// to strings provided in `self`, and `&OsStr` to those provided in the `args` parameter. Take
    /// note of this with respect to object lifetimes.
    ///
    /// Expects `self` to be valid (see [`is_valid`](#method.is_valid)).
    #[inline(always)]
    pub fn parse_os<A>(&self, args: &'s [A]) -> super::analysis::Analysis<'s, OsStr>
        where A: 's + AsRef<OsStr>
    {
        super::engine_os::process(args, self)
    }
}
