// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! The parser & parser settings
//!
//! A [`Parser`] wraps a description of a collection of available program [*options*][options] and
//! [*command arguments*][commands], along with parser settings, and provides the parser methods
//! that parse a given set of input arguments.
//!
//! # Parsing style
//!
//! Parsing an argument list can be done in two different ways, depending upon your preferences or
//! application design requirements:
//!
//! - “All in one” style: The [`parse`] and [`parse_os`] methods return an [`Analysis`] object,
//!   which contains a `Vec` holding descriptions from parsing the entire argument list in one go.
//! - “Iterative” style: The [`parse_iter`] and [`parse_iter_os`] methods return an iterator, which
//!   returns analysis items one at a time.
//!
//! Note that the iterator objects have methods that allow you to change the *option set* and
//! *command set* used for subsequent iterations. The purpose behind these methods is for situations
//! where you have a [*command argument*][commands] based program, but do not wish to describe the
//! full command structure up front, with these methods giving you the ability to manually set the
//! *option* and *command* sets to use in subsequent iterations after encountering a *command*.
//!
//! # Input arguments
//!
//! A [`Parser`] can parse input arguments in both of the following forms:
//!
//!  - `AsRef<str>` (`String` and `&str`)
//!  - `AsRef<OsStr>` (`OsString` and `&OsStr`)
//!
//! Most programs will only want the former, however there may be cases where the latter is
//! wanted or needed. The [unicode documentation][unicode] may also be of interest here.
//!
//! Note that the [`parse`] and [`parse_iter`] methods are both `AsRef<str>` based, while
//! [`parse_os`] and [`parse_iter_os`] are the `AsRef<OsStr>` based alternatives.
//!
//! # Settings
//!
//! A [`Parser`] has settings to control certain aspects of parsing, for instance to choose which
//! mode to parse options in (*standard* or *alternate*; the difference of which is discussed in the
//! [option support documentation][options]), and whether or not to allow abbreviated *long option*
//! name matching.
//!
//! [`Analysis`]: ../analysis/struct.Analysis.html
//! [`Parser`]: struct.Parser.html
//! [`parse`]: struct.Parser.html#method.parse
//! [`parse_iter`]: struct.Parser.html#method.parse_iter
//! [`parse_os`]: struct.Parser.html#method.parse_os
//! [`parse_iter_os`]: struct.Parser.html#method.parse_iter_os
//! [commands]: ../docs/commands/index.html
//! [options]: ../docs/options/index.html
//! [unicode]: ../docs/unicode/index.html

use std::convert::AsRef;
use std::ffi::OsStr;
use super::analysis::{Analysis, ItemClass};
use super::commands::{CommandSet, CommandFlaw};
use super::options::{OptionSet, OptionFlaw};

// NB: We export this in the public API here (the `engine` and `engine_os` mods are private)
pub use super::engine::ParseIter;
pub use super::engine_os::ParseIterOs;

/// Default abbreviation support state
pub(crate) const ABBR_SUP_DEFAULT: bool = true;
/// Default mode
pub(crate) const MODE_DEFAULT: OptionsMode = OptionsMode::Standard;

/// The parser
///
/// Holds the option set and command set descriptions used for parsing input arguments, along with
/// parser settings, and provides methods for parsing.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
            options: &gong_option_set!(),
            commands: &gong_command_set!(),
            settings: Settings::default(),
        }
    }
}

/// Settings for parser
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Option parsing mode to use
    pub mode: OptionsMode,
    /// Whether or not to allow abbreviated long option name matching
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

/// Used to specify which option parsing mode to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionsMode {
    /// Standard (default): Short (e.g. `-o`) and long (e.g. `--foo`) options, with single and
    /// double dash prefixes respectively.
    Standard,
    /// Alternate: Long options only, with single dash prefix (e.g. `-foo`).
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

    /// Control matching of abbreviated long option names (set to `true` to allow)
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
            commands: commands.unwrap_or(&gong_command_set!()),
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
    /// Note, only the most crucial command and option identifier (name or character) problems that
    /// could cause issues when parsing are checked for; Passing validation is not a confirmation
    /// that the command or option identifiers used are all sensible or otherwise entirely free of
    /// issues.
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
    ///
    /// Note, only the most crucial command and option identifier (name or character) problems that
    /// could cause issues when parsing are checked for; Passing validation is not a confirmation
    /// that the command or option identifiers used are all sensible or otherwise entirely free of
    /// issues.
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

    /// Gives an iterator for parsing the provided program arguments
    ///
    /// Returns an iterator. Each iteration consumes one (or sometimes two) input arguments (except
    /// with a *short option set* where one short option in the set is consumed), returning a single
    /// analysis item.
    ///
    /// The returned analysis item may include `&str` references to strings provided in the `args`
    /// parameter and/or in `self`. Take note of this with respect to object lifetimes.
    ///
    /// Expects `self` to be valid (see [`is_valid`](#method.is_valid)).
    #[inline(always)]
    pub fn parse_iter<A>(&'r self, args: &'s [A]) -> ParseIter<'r, 's, A>
        where A: 's + AsRef<str>
    {
        ParseIter::new(args, self)
    }

    /// Gives an iterator for parsing the provided program arguments, given as `OsStr`
    ///
    /// Returns an iterator. Each iteration consumes one (or sometimes two) input arguments (except
    /// with a *short option set* where one short option in the set is consumed), returning a single
    /// analysis item.
    ///
    /// The returned analysis item may include `&str` references to strings provided in `self`,
    /// and/or `&OsStr` to those provided in the `args` parameter. Take note of this with respect to
    /// object lifetimes.
    ///
    /// Expects `self` to be valid (see [`is_valid`](#method.is_valid)).
    #[inline(always)]
    pub fn parse_iter_os<A>(&'r self, args: &'s [A]) -> ParseIterOs<'r, 's, A>
        where A: 's + AsRef<OsStr>
    {
        ParseIterOs::new(args, self)
    }

    /// Parses the provided program arguments
    ///
    /// Returns an analysis describing the parsed argument list.
    ///
    /// The returned analysis item may include `&str` references to strings provided in the `args`
    /// parameter and/or in `self`. Take note of this with respect to object lifetimes.
    ///
    /// Expects `self` to be valid (see [`is_valid`](#method.is_valid)).
    pub fn parse<A>(&self, args: &'s [A]) -> Analysis<'s, str>
        where A: 's + AsRef<str>
    {
        let mut analysis = Analysis::new(args.len());
        let parse_iter = ParseIter::new(args, self);
        let items = parse_iter.inspect(|item| {
            match item {
                ItemClass::Err(_) => analysis.error = true,
                ItemClass::Warn(_) => analysis.warn = true,
                ItemClass::Ok(_) => {},
            }
        }).collect();
        analysis.items = items;
        analysis
    }

    /// Parses the provided program arguments, given as `OsStr`
    ///
    /// Returns an analysis describing the parsed argument list.
    ///
    /// The returned analysis item may include `&str` references to strings provided in `self`,
    /// and/or `&OsStr` to those provided in the `args` parameter. Take note of this with respect to
    /// object lifetimes.
    ///
    /// Expects `self` to be valid (see [`is_valid`](#method.is_valid)).
    pub fn parse_os<A>(&self, args: &'s [A]) -> Analysis<'s, OsStr>
        where A: 's + AsRef<OsStr>
    {
        let mut analysis = Analysis::new(args.len());
        let parse_iter = ParseIterOs::new(args, self);
        let items = parse_iter.inspect(|item| {
            match item {
                ItemClass::Err(_) => analysis.error = true,
                ItemClass::Warn(_) => analysis.warn = true,
                ItemClass::Ok(_) => {},
            }
        }).collect();
        analysis.items = items;
        analysis
    }
}
