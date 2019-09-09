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
//! A “parser” object wraps a description of a collection of available program [*options*][options]
//! and [*command arguments*][commands] (where applicable), along with parser settings, and provides
//! the parse methods that parse a given set of input arguments.
//!
//! Two parsers are provided; [`Parser`] is suitable for use cases that do not involve command
//! arguments, while the alternative [`CmdParser`] is suitable for those that do.
//!
//! As just mentioned, the parser holds the settings that control certain aspects of parsing, which
//! you should configure correctly as needed *before* performing parsing.
//!
//! # Parsing style
//!
//! Parsing an argument list can be done in two different ways, depending upon your preferences or
//! application design requirements:
//!
//! - The “iterative” (“one at a time”) model: The [`parse_iter`] method returns an iterator, with
//!   which you can simply handle each analysed “item” one at a time.
//! - The “data-mining” (“all in one”) model: The [`parse`] method of [`Parser`] returns an
//!   [`ItemSet`] object, which wraps a `Vec` of items, collected from use of the above mentioned
//!   iterator. It contains a quick indication of whether or not any problems were found, and
//!   importantly, it provides a set of “data mining” methods for retrieving information from the
//!   item set. With [`CmdParser`] an [`CommandAnalysis`] is returned, which breaks up the items as
//!   appropriate per use of command arguments.
//!
//! # Command-based parsing
//!
//! Note that the iterator objects have methods that allow you to change the *option set* and
//! *command set* used for subsequent iterations. The purpose behind these methods is for situations
//! where you have a [*command argument*][commands] based program, but do not wish to describe the
//! full command structure up front, with these methods giving you the ability to manually set the
//! *option* and *command* sets to use in subsequent iterations after encountering a *command*.
//!
//! [`ItemSet`]: ../analysis/struct.ItemSet.html
//! [`Parser`]: struct.Parser.html
//! [`CmdParser`]: struct.CmdParser.html
//! [`parse`]: struct.Parser.html#method.parse
//! [`parse_iter`]: struct.Parser.html#method.parse_iter
//! [`CommandAnalysis`]: ../analysis/struct.CommandAnalysis.html
//! [commands]: ../docs/ch4_commands/index.html
//! [options]: ../docs/ch3_options/index.html

use std::convert::AsRef;
use std::ffi::OsStr;
use crate::{option_set, command_set};
use crate::analysis::{ItemSet, CommandAnalysis};
use crate::commands::{CommandSet, CommandFlaw};
use crate::options::{OptionSet, OptionFlaw};
use crate::positionals::Policy as PositionalsPolicy;

// NB: We export this in the public API here (the `engine` mod is private)
pub use crate::engine::{ParseIter, ParseIterIndexed, CmdParseIter, CmdParseIterIndexed};

/// The parser
///
/// Holds the option set and command set descriptions used for parsing input arguments, along with
/// parser settings, and provides methods for parsing.
///
/// If your program is built upon use of “command” arguments then use [`CmdParser`] instead.
///
/// [`CmdParser`]: struct.CmdParser.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Parser<'r, 'set: 'r> {
    /* NOTE: these have been left public to allow efficient static creation */
    /// The main (top level) option set
    pub options: &'r OptionSet<'r, 'set>,
    /// Policy for *positionals*
    pub positionals_policy: PositionalsPolicy,
    /// Settings
    pub settings: Settings,
}

/// The parser, command based
///
/// Holds the option set and command set descriptions used for parsing input arguments, along with
/// parser settings, and provides methods for parsing.
///
/// Use this instead of [`Parser`] if your program is built upon use of “command” arguments.
///
/// [`Parser`]: struct.Parser.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CmdParser<'r, 'set: 'r> {
    /* NOTE: these have been left public to allow efficient static creation */
    /// Command set
    pub commands: &'r CommandSet<'r, 'set>,
    /// Normal parser
    pub inner: Parser<'r, 'set>,
}

impl<'r, 'set: 'r> Default for Parser<'r, 'set> {
    fn default() -> Self {
        Self {
            options: &option_set!(),
            positionals_policy: PositionalsPolicy::default(),
            settings: Settings::default(),
        }
    }
}

impl<'r, 'set: 'r> Default for CmdParser<'r, 'set> {
    fn default() -> Self {
        Self {
            commands: &command_set!(),
            inner: Parser::default(),
        }
    }
}

/// Settings for parser
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Option parsing mode to use
    pub mode: OptionsMode,
    /// Whether or not to allow abbreviated long option name matching
    pub allow_opt_abbreviations: bool,
    /// Whether or not to allow abbreviated command name matching
    pub allow_cmd_abbreviations: bool,
    /// Whether or not to stop interpretation of arguments as possible options/commands upon
    /// encountering a positional argument, similar to encountering an early terminator, i.e.
    /// “posixly correct” behaviour. See [the respective setter method][set_posixly_correct]
    /// documentation for details.
    ///
    /// [set_posixly_correct]: #method.set_posixly_correct
    pub posixly_correct: bool,
    /// Whether or not to stop parsing when a problem is encountered.
    ///
    /// This only applies to the non-iterative form of parsing (with iterative you can just stop
    /// iterating). When a problem is encountered, there typically is no guarantee that remaining
    /// arguments will be interpreted correctly; this, when `true`, allows parsing to immediately
    /// stop, allowing you to avoid potentially wasted effort.
    pub stop_on_problem: bool,
    /// Whether or not to automatically include mismatch suggestion results in unknown long option
    /// and unknown command item variants.
    #[cfg(feature = "suggestions")]
    pub serve_suggestions: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: OptionsMode::Standard,
            allow_opt_abbreviations: true,
            allow_cmd_abbreviations: false,
            posixly_correct: false,
            stop_on_problem: true,
            #[cfg(feature = "suggestions")]
            serve_suggestions: true,
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
        OptionsMode::Standard
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
    pub fn set_allow_opt_abbreviations(&mut self, allow: bool) -> &mut Self {
        self.allow_opt_abbreviations = allow;
        self
    }

    /// Control matching of abbreviated command names (set to `true` to allow)
    #[inline(always)]
    pub fn set_allow_cmd_abbreviations(&mut self, allow: bool) -> &mut Self {
        self.allow_cmd_abbreviations = allow;
        self
    }

    /// Control whether or not “posixly correct” mode is enabled
    ///
    /// “Posixly correct” mode means that upon encountering a *positional* argument all subsequent
    /// arguments must also be considered to be *positionals*.
    ///
    /// This mode is disabled by default.
    ///
    /// # Background
    ///
    /// As discussed in `Appendix B` of `The Linux Programming Interface`, `2010` by `Michael
    /// Kerrisk`, the POSIX/SUS standards relating to command line argument parsing do not allow
    /// mixing of *option* and *positional* arguments, requiring that all *options* must come before
    /// *positional*, and on encountering a *positional*, all subsequent arguments should be
    /// interpreted as being *positionals*.
    ///
    /// Also discussed is the fact that the `glibc` (standard GNU C library) implementation by
    /// default does not conform to this requirement, allowing mixing, though will change its
    /// behaviour to conform in the presence of an environment variable named `POSIXLY_CORRECT`.
    ///
    /// # Notes
    ///
    /// The design of this library **optionally** allows free inter-mixing of *option* and
    /// *positional* arguments. The default state is to allow such free mixing. If you are building
    /// a program that for any reason needs to conform to the above requirement of the mentioned
    /// standards then this setting is very relevant to you; you can achieve such conformance simply
    /// by configuring this setting to `true` via this method.
    ///
    /// Note that this library does **not** itself pay any attention to the mentioned
    /// `POSIXLY_CORRECT` environment variable. You of course can freely do so in your application
    /// however, changing this setting in response as appropriate.
    #[inline(always)]
    pub fn set_posixly_correct(&mut self, enable: bool) -> &mut Self {
        self.posixly_correct = enable;
        self
    }

    /// Control whether or not non-iterative parsing stops on finding a problem
    #[inline(always)]
    pub fn set_stop_on_problem(&mut self, enable: bool) -> &mut Self {
        self.stop_on_problem = enable;
        self
    }

    /// Control whether or not mismatch suggestions are included
    #[cfg(feature = "suggestions")]
    #[inline(always)]
    pub fn set_serve_suggestions(&mut self, serve: bool) -> &mut Self {
        self.serve_suggestions = serve;
        self
    }
}

impl<'r, 'set: 'r, 'arg: 'r> Parser<'r, 'set> {
    /// Create a new parser
    pub fn new(options: &'r OptionSet<'r, 'set>) -> Self {
        Self {
            options: options,
            .. Self::default()
        }
    }

    /// Get a mutable reference to settings
    #[inline(always)]
    pub fn settings(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Set the policy for positionals
    ///
    /// Panics on invalid policy.
    #[inline]
    pub fn set_positionals_policy(&mut self, policy: PositionalsPolicy) {
        policy.assert_valid();
        self.positionals_policy = policy;
    }

    /// Checks validity of the option set and positionals policy
    ///
    /// Returns `true` if valid.
    ///
    /// Note, only the most crucial option identifier (name or character) problems that could cause
    /// issues when parsing are checked for; Passing validation is not a confirmation that the
    /// option identifiers used are all sensible or otherwise entirely free of issues.
    ///
    /// See also the [`validate`](#method.validate) method.
    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.options.is_valid() && self.positionals_policy.is_valid()
    }

    /// Checks validity of the option set, returning details of any problems
    ///
    /// Note, only the most crucial option identifier (name or character) problems that could cause
    /// issues when parsing are checked for; Passing validation is not a confirmation that the
    /// option identifiers used are all sensible or otherwise entirely free of issues.
    pub fn validate(&self) -> Result<(), Vec<OptionFlaw<'set>>> {
        self.options.validate()
    }

    /// Gives an iterator for parsing the provided program arguments
    ///
    /// Returns an iterator. Each iteration consumes one (or sometimes two) input arguments (except
    /// with a *short option set* where one short option in the set is consumed), returning a single
    /// analysis item.
    ///
    /// The returned analysis item may include `&str` references to strings provided in `self`,
    /// and/or `&OsStr` to those provided in the `args` parameter. Take note of this with respect to
    /// object lifetimes.
    ///
    /// Note, it is undefined behaviour to perform parsing with a non-valid option and/or command
    /// set. See [`is_valid`] and [`validate`] for validation checking methods.
    ///
    /// [`is_valid`]: #method.is_valid
    /// [`validate`]: #method.validate
    #[inline(always)]
    #[must_use]
    pub fn parse_iter<A>(&'r self, args: &'arg [A]) -> ParseIter<'r, 'set, 'arg, A>
        where A: AsRef<OsStr> + 'arg
    {
        ParseIter::new(args, self)
    }

    /// Parses the provided program arguments
    ///
    /// Returns an analysis describing the parsed argument list. While the [`parse_iter`] method is
    /// aimed at iterative based parsing, this alternative provides a “parse-and-data-mine”
    /// alternative; it basically uses an iterator, collecting all of the results into an object
    /// which has a set of methods for mining the result set for information.
    ///
    /// The returned analysis item may include `&str` references to strings provided in `self`,
    /// and/or `&OsStr` to those provided in the `args` parameter. Take note of this with respect to
    /// object lifetimes.
    ///
    /// Note, it is undefined behaviour to perform parsing with a non-valid option and/or command
    /// set. See [`is_valid`] and [`validate`] for validation checking methods.
    ///
    /// [`is_valid`]: #method.is_valid
    /// [`validate`]: #method.validate
    /// [`parse_iter`]: #method.parse_iter
    #[inline(always)]
    #[must_use]
    pub fn parse<A>(&self, args: &'arg [A]) -> ItemSet<'r, 'set, 'arg>
        where A: AsRef<OsStr> + 'arg
    {
        ItemSet::from(ParseIter::new(args, self))
    }
}

impl<'r, 'set: 'r, 'arg: 'r> CmdParser<'r, 'set> {
    /// Create a new parser
    pub fn new(options: &'r OptionSet<'r, 'set>, commands: &'r CommandSet<'r, 'set>) -> Self {
        Self {
            commands: commands,
            inner: Parser::new(options),
        }
    }

    /// Get a mutable reference to settings
    #[inline(always)]
    pub fn settings(&mut self) -> &mut Settings {
        self.inner.settings()
    }

    /// Set the policy for positionals
    ///
    /// Panics on invalid policy.
    #[inline]
    pub fn set_positionals_policy(&mut self, policy: PositionalsPolicy) {
        self.inner.set_positionals_policy(policy);
    }

    /// Checks validity of the option set, command set and positionals policy
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
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.inner.is_valid() && self.commands.is_valid()
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
    pub fn validate(&self) -> Result<(), (Vec<OptionFlaw<'set>>, Vec<CommandFlaw<'set>>)> {
        let option_set_flaws = self.inner.options.validate();
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
    /// The returned analysis item may include `&str` references to strings provided in `self`,
    /// and/or `&OsStr` to those provided in the `args` parameter. Take note of this with respect to
    /// object lifetimes.
    ///
    /// Note, it is undefined behaviour to perform parsing with a non-valid option and/or command
    /// set. See [`is_valid`] and [`validate`] for validation checking methods.
    ///
    /// [`is_valid`]: #method.is_valid
    /// [`validate`]: #method.validate
    #[inline(always)]
    #[must_use]
    pub fn parse_iter<A>(&'r self, args: &'arg [A]) -> CmdParseIter<'r, 'set, 'arg, A>
        where A: AsRef<OsStr> + 'arg
    {
        CmdParseIter::new(args, self)
    }

    /// Parses the provided program arguments
    ///
    /// Returns an analysis describing the parsed argument list. While the [`parse_iter`] method is
    /// aimed at iterative based parsing, this alternative provides a “parse-and-data-mine”
    /// alternative.
    ///
    /// This is similar to [`Parser::parse`] except of course being more suited to programs built
    /// upon use of “command” arguments. The primary difference is that rather than returning one
    /// `ItemSet`, it breaks up (partitions) the analysis items by command use, thus the object
    /// returned wraps a list of `ItemSet`s and command names. It also provides a reference to the
    /// relevant `CommandSet` for use with unknown-command suggestion matching.
    ///
    /// The returned analysis item may include `&str` references to strings provided in `self`,
    /// and/or `&OsStr` to those provided in the `args` parameter. Take note of this with respect to
    /// object lifetimes.
    ///
    /// Note, it is undefined behaviour to perform parsing with a non-valid option and/or command
    /// set. See [`is_valid`] and [`validate`] for validation checking methods.
    ///
    /// [`is_valid`]: #method.is_valid
    /// [`validate`]: #method.validate
    /// [`parse_iter`]: #method.parse_iter
    /// [`Parser::parse`]: struct.Parser.html#method.parse
    #[inline(always)]
    #[must_use]
    pub fn parse<A>(&self, args: &'arg [A]) -> CommandAnalysis<'r, 'set, 'arg>
        where A: AsRef<OsStr> + 'arg
    {
        CommandAnalysis::from(CmdParseIter::new(args, self))
    }
}
