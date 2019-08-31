// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Command description components
//!
//! This module contains components to do with describing the *command* arguments “available” within
//! a given program, i.e. those that an argument list will be parsed against. There are components
//! for describing both individual *commands* and sets of *commands*.
//!
//! See the separate [*commands* support discussion][commands] for details on what *command*
//! arguments are, and the details on how parsing works with respect to them.
//!
//! [commands]: ../docs/ch4_commands/index.html

#[cfg(feature = "suggestions")]
use std::ffi::OsStr;
use crate::option_set;
use crate::options::{self, OptionSet, OptionFlaw};
use crate::positionals::Policy as PositionalsPolicy;

/// Description of an available command
///
/// The `options` and `sub_commands` attributes are used to specify the sets to be used for parsing
/// arguments that follow use of a specific command in an argument list.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Command<'r, 's: 'r> {
    /* NOTE: these have been left public to allow efficient static creation of options */
    /// Command name
    pub name: &'s str,
    /// Options
    pub options: &'r OptionSet<'r, 's>,
    /// Sub-commands
    pub sub_commands: CommandSet<'r, 's>,
    /// Policy for *positionals*
    pub positionals_policy: PositionalsPolicy,
}

/// Extendible command set
///
/// Used to supply the set of information about available commands to match against
///
/// This is the “extendible” variant which uses `Vec`s to hold the command lists and thus is
/// flexible in allowing addition of commands, and may re-allocate as necessary.
///
/// Note, certain add option methods panic with invalid identifiers, as documented, however you must
/// understand that the validation checks only do the bare minimum of checking for the most crucial
/// problems that could cause issues when parsing. It is up to you to otherwise ensure that
/// identifiers are sensibly chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSetEx<'r, 's: 'r> {
    /* NOTE: these have been left public to allow creation via macros */
    pub commands: Vec<Command<'r, 's>>,
}

impl<'r, 's: 'r> Default for CommandSetEx<'r, 's> {
    fn default() -> Self {
        CommandSetEx::new()
    }
}

/// Command set
///
/// Used to supply the set of information about available commands to match against
///
/// This is the non-“extendible” variant. Unlike its cousin `CommandSetEx`, this holds the command
/// list as a slice reference rather than a `Vec`, and thus cannot be extended in size (hence no
/// `add_*` methods). This is particularly useful in efficient creation of static/const command
/// sets.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct CommandSet<'r, 's: 'r> {
    /* NOTE: these have been left public to allow efficient static creation of commands */
    pub commands: &'r [Command<'r, 's>],
}

impl<'r, 's: 'r> From<&'r CommandSetEx<'r, 's>> for CommandSet<'r, 's> {
    /// Create an `CommandSet` referencing an `CommandSetEx`’s vectors as slices
    fn from(r: &'r CommandSetEx<'r, 's>) -> Self {
        Self {
            commands: &r.commands[..],
        }
    }
}

impl<'r, 's: 'r> From<CommandSet<'r, 's>> for CommandSetEx<'r, 's> {
    /// Create an `CommandSetEx` from an `CommandSet`
    fn from(c: CommandSet<'r, 's>) -> Self {
        Self { commands: c.commands.iter().cloned().collect() }
    }
}

impl<'r, 's: 'r> PartialEq<CommandSet<'r, 's>> for CommandSetEx<'r, 's> {
    fn eq(&self, rhs: &CommandSet<'r, 's>) -> bool {
        CommandSet::from(self).eq(rhs)
    }
}

impl<'r, 's: 'r> PartialEq<CommandSetEx<'r, 's>> for CommandSet<'r, 's> {
    fn eq(&self, rhs: &CommandSetEx<'r, 's>) -> bool {
        self.eq(&CommandSet::from(rhs))
    }
}

/// Description of a validation issue within a command in a [`CommandSet`](struct.CommandSet.html)
/// or [`CommandSetEx`](struct.CommandSetEx.html) set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandFlaw<'a> {
    /// Command name is an empty string
    EmptyName,
    /// Command name contains a forbidden `char`
    NameHasForbiddenChar(&'a str, char),
    /// Duplicate command found
    Duplicated(&'a str),
    /// Flaws for the option set belonging to a command
    NestedOptSetFlaws(&'a str, Vec<OptionFlaw<'a>>),
    /// Flaws for the sub-command set belonging to a command
    NestedSubCmdFlaws(&'a str, Vec<CommandFlaw<'a>>),
}

impl<'r, 's: 'r> Command<'r, 's> {
    /// Create a new command descriptor
    ///
    /// Panics (debug only) if the given name is invalid.
    fn new(name: &'s str, options: Option<&'r OptionSet<'r, 's>>,
        sub_commands: CommandSet<'r, 's>) -> Self
    {
        debug_assert!(Self::validate(name).is_ok());
        let opts_actual = options.unwrap_or(&option_set!());
        Self { name, options: opts_actual, sub_commands, positionals_policy: PositionalsPolicy::default() }
    }

    /// Set the policy for positionals
    #[inline]
    pub fn set_positional_reqs(&mut self, policy: PositionalsPolicy) {
        self.positionals_policy = policy;
    }

    /// Validate a given name as a possible command
    ///
    /// Returns the first flaw identified, if any
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    #[must_use]
    fn validate(name: &str) -> Result<(), CommandFlaw> {
        if name.is_empty() {
            return Err(CommandFlaw::EmptyName);
        }
        // Would cause problems with correct `OsStr` based parsing
        if name.contains('\u{FFFD}') {
            return Err(CommandFlaw::NameHasForbiddenChar(name, '\u{FFFD}'));
        }
        Ok(())
    }
}

impl<'r, 's: 'r> CommandSetEx<'r, 's> {
    /// Create a new object
    ///
    /// You can alternatively use [`with_capacity`](#method.with_capacity) for more efficient `Vec`
    /// creation.
    #[inline]
    pub fn new() -> Self {
        Self { commands: Vec::new(), }
    }

    /// Create a new object, with size estimation
    ///
    /// Takes estimations of the number of commands to expect to be added (for efficient vector
    /// allocation).
    #[inline]
    pub fn with_capacity(count_est: usize) -> Self {
        Self { commands: Vec::with_capacity(count_est), }
    }

    /// Create a [`CommandSet`](struct.CommandSet.html) referencing `self`’s vectors as slices
    #[inline]
    pub fn as_fixed(&'r self) -> CommandSet<'r, 's> {
        self.into()
    }

    /// Checks if empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Add a command
    ///
    /// Panics (debug only) on invalid name.
    #[inline]
    pub fn add_command(&mut self, name: &'s str, options: Option<&'r OptionSet<'r, 's>>,
        sub_commands: CommandSet<'r, 's>) -> &mut Self
    {
        self.commands.push(Command::new(name, options, sub_commands));
        self
    }

    /// Add an existing (ready-made) command
    #[inline]
    pub fn add_existing_command(&mut self, command: Command<'r, 's>) -> &mut Self {
        self.commands.push(command);
        self
    }

    /// Checks validity of command set
    ///
    /// Returns `true` if valid.
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    ///
    /// See also the [`validate`](#method.validate) method.
    #[inline]
    pub fn is_valid(&self) -> bool {
        validation::validate_set(&self.as_fixed(), false).is_ok()
    }

    /// Checks validity of command set, returning details of any problems
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    #[inline]
    pub fn validate(&self) -> Result<(), Vec<CommandFlaw<'s>>> {
        validation::validate_set(&self.as_fixed(), true)
    }

    /// Find the best matching command for the given string
    ///
    /// This is intended to be used when a command argument was expected, and a *positional*
    /// argument was given, but it was not matched against any available command, and you want to
    /// report an “unrecognised command” error, indicating the most likely option the user may have
    /// meant, if a suitable suggestion can be found. E.g.
    ///
    /// > “Error: Unknown command ‘x’, did you mean ‘y’”
    ///
    /// Specifically, this uses the `jaro_winkler` algorithm from the `strsim` crate; It filters
    /// out any commands with a metric calculated as less than `0.8`, and returns the first command
    /// with the highest metric.
    #[cfg(feature = "suggestions")]
    #[inline]
    pub fn suggest(&self, unknown: &OsStr) -> Option<&'s str> {
        self.as_fixed().suggest(unknown)
    }
}

impl<'r, 's: 'r> CommandSet<'r, 's> {
    /// Converts to an “extendible” copy of `self`
    ///
    /// This duplicates the options in `self` into a [`CommandSetEx`](struct.CommandSetEx.html).
    #[inline]
    pub fn to_extendible(self) -> CommandSetEx<'r, 's> {
        CommandSetEx::from(self)
    }

    /// Checks if empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Checks validity of command set
    ///
    /// Returns `true` if valid.
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    ///
    /// See also the [`validate`](#method.validate) method.
    #[inline(always)]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        validation::validate_set(self, false).is_ok()
    }

    /// Checks validity of command set, returning details of any problems
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    #[inline(always)]
    pub fn validate(&self) -> Result<(), Vec<CommandFlaw<'s>>> {
        validation::validate_set(self, true)
    }

    /// Find the best matching command for the given string
    ///
    /// This is intended to be used when a command argument was expected, and a *positional*
    /// argument was given, but it was not matched against any available command, and you want to
    /// report an “unrecognised command” error, indicating the most likely option the user may have
    /// meant, if a suitable suggestion can be found. E.g.
    ///
    /// > “Error: Unknown command ‘x’, did you mean ‘y’”
    ///
    /// Specifically, this uses the `jaro_winkler` algorithm from the `strsim` crate; It filters
    /// out any commands with a metric calculated as less than `0.8`, and returns the first command
    /// with the highest metric.
    #[cfg(feature = "suggestions")]
    pub fn suggest(&self, unknown: &OsStr) -> Option<&'s str> {
        let unknown_lossy = unknown.to_string_lossy();
        crate::matching::suggest(&unknown_lossy, self.commands.iter(), |&c| c.name)
    }
}

/// Command set validation
mod validation {
    use super::{options, Command, CommandSet, CommandFlaw};

    /// Checks validity of command set, optionally returning details of any problems
    ///
    /// If no problems are found, it returns `Ok(())`, otherwise `Err(_)`.
    ///
    /// If `detail` is `false`, it returns early on encountering a problem (with an empty `Vec`),
    /// useful for quick `is_valid` checks. Otherwise it builds up and provides a complete list of
    /// flaws.
    #[must_use]
    pub fn validate_set<'r, 's: 'r>(set: &CommandSet<'r, 's>, detail: bool)
        -> Result<(), Vec<CommandFlaw<'s>>>
    {
        let mut flaws: Vec<CommandFlaw<'s>> = Vec::new();

        // Validate command names
        for command in set.commands {
            if let Err(f) = Command::validate(command.name) {
                match detail {
                    true => { flaws.push(f); },
                    false => { return Err(flaws); },
                }
            }
        }

        // Check for name duplicates
        let mut dupes: bool = false;
        find_duplicates(set, &mut flaws, detail, &mut dupes);
        if !detail && dupes {
            return Err(flaws);
        }

        // Check the sub_commands and option sets of commands
        for command in set.commands {
            if let Err(f) = options::validation::validate_set(command.options, detail) {
                match detail {
                    true => { flaws.push(CommandFlaw::NestedOptSetFlaws(command.name, f)); },
                    false => { return Err(flaws); },
                }
            }
            if let Err(f) = validate_set(&command.sub_commands, detail) {
                match detail {
                    true => { flaws.push(CommandFlaw::NestedSubCmdFlaws(command.name, f)); },
                    false => { return Err(flaws); },
                }
            }
        }

        match flaws.is_empty() {
            true => Ok(()),
            false => Err(flaws),
        }
    }

    fn find_duplicates<'r, 's: 'r>(set: &CommandSet<'r, 's>,
        flaws: &mut Vec<CommandFlaw<'s>>, detail: bool, found: &mut bool)
    {
        let cmds = set.commands;
        if cmds.is_empty() { return; }
        let mut duplicates = Vec::new();
        for (i, cmd) in cmds[..cmds.len()-1].iter().enumerate() {
            let name = cmd.name.clone();
            if !duplicates.contains(&CommandFlaw::Duplicated(name)) {
                for cmd2 in cmds[i+1..].iter() {
                    if name == cmd2.name {
                        match detail {
                            true => {
                                duplicates.push(CommandFlaw::Duplicated(name));
                                break;
                            },
                            false => { *found = true; return; },
                        }
                    }
                }
            }
        }
        if !duplicates.is_empty() {
            flaws.append(&mut duplicates);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_set;

    /// Check `Command::new` rejects empty string
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_cmd_no_name() {
        let _cmd = Command::new("", None, command_set!()); // Should panic here in debug mode!
    }

    /* Command names cannot contain the unicode replacement char (`\u{FFFD}`). Support for handling
     * `OsStr` based argument sets involves a temporary lossy conversion to `str`, and if the
     * replacement char was allowed, this could result in incorrect matches.
     */

    /// Check `Command::new` rejects unicode replacement char (`\u{FFFD}`)
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_cmd_with_rep_char() {
        let _cmd = Command::new("a\u{FFFD}b", None, command_set!()); // Should panic here in debug mode!
    }
}
