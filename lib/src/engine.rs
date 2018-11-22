// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

use std::convert::AsRef;
use std::iter::Enumerate;
use std::slice;
use std::str::CharIndices;
use super::parser::*;
use super::options::*;
use super::analysis::*;

pub(crate) const SINGLE_DASH_PREFIX: &str = "-";
pub(crate) const DOUBLE_DASH_PREFIX: &str = "--";
const EARLY_TERMINATOR: &str = "--";

type ArgTypeAssessor = fn(&str) -> ArgTypeBasic<'_>;

/// An argument list parsing iterator
///
/// Created by the [`parse_iter`] method of [`Parser`].
///
/// [`parse_iter`]: ../parser/struct.Parser.html#method.parse_iter
/// [`Parser`]: ../parser/struct.Parser.html
#[derive(Clone)]
pub struct ParseIter<'r, 's: 'r, A: 's + AsRef<str>> {
    /// Enumerated iterator over the argument list
    arg_iter: Enumerate<slice::Iter<'s, A>>,
    /// The parser data in use (will change on encountering a command)
    pub(crate) parser_data: Parser<'r, 's>,
    /// If an early terminator has been encountered, all subsequent arguments are non-options
    early_terminator_encountered: bool,
    /// A non-option is only assessed as being a possible command if 1) it is the first encountered
    /// for each option-set analysis (reset for each command identified), and 2) we have not
    /// encountered an early terminator.
    try_command_matching: bool,
    /// Function for determining basic argument type (different function per option mode)
    get_basic_arg_type_fn: ArgTypeAssessor,
    /// Short option set argument iterator
    short_set_iter: Option<ShortSetIter<'r, 's, A>>,
}

/// A short option set string iterator
#[derive(Debug, Clone)]
struct ShortSetIter<'r, 's: 'r, A: 's + AsRef<str>> {
    /// Enumerated iterator over the argument list
    arg_iter: Enumerate<slice::Iter<'s, A>>,
    /// The parser data in use
    parser_data: Parser<'r, 's>,
    /// The short option set string being iterated over.
    /// We need to hold a copy of this at least for the purpose of extracting in-same-arg data.
    string: &'s str,
    /// Char iterator
    iter: CharIndices<'s>,
    /// Index of argument the set came from, for recording in items
    arg_index: usize,
    /// For marking as fully consumed when remaining portion of the string has been consumed as the
    /// data value of a short option, bypassing the fact that the char iterator has not finished.
    consumed: bool,
}

/// Basic argument type
///
/// Option variants should: include argument without prefix; include “in-same-arg” data values.
enum ArgTypeBasic<'a> {
    NonOption,
    EarlyTerminator,
    LongOption(&'a str),
    ShortOptionSet(&'a str),
}

impl<'r, 's, A> Iterator for ParseIter<'r, 's, A>
    where A: 's + AsRef<str>, 's: 'r
{
    type Item = ItemClass<'s, str>;

    fn next(&mut self) -> Option<Self::Item> {
        // Continue from where we left off for a short option set?
        if self.short_set_iter.is_some() {
            let mut set_iter = self.short_set_iter.take().unwrap();
            let result = set_iter.get_next();
            if result.is_some() {
                self.short_set_iter = Some(set_iter); // Move it back
                return result;
            }
            else {
                // Update our iterator based on short option set progress (it may have consumed an
                // extra one for an in-next-arg data value, and it’s working on a copy of our
                // iterator, not a reference).
                self.arg_iter = set_iter.arg_iter;
            }
        }
        // Do next argument, if there is one
        self.get_next()
    }
}

impl<'r, 's, A> Iterator for ShortSetIter<'r, 's, A>
    where A: 's + AsRef<str>, 's: 'r
{
    type Item = ItemClass<'s, str>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Do next short, if there is one
        self.get_next()
    }
}

impl<'r, 's, A> ParseIter<'r, 's, A>
    where A: 's + AsRef<str>, 's: 'r
{
    /// Create a new instance
    pub(crate) fn new(args: &'s [A], parser: &Parser<'r, 's>) -> Self {
        Self {
            arg_iter: args.iter().enumerate(),
            parser_data: *parser,
            early_terminator_encountered: false,
            try_command_matching: true,
            get_basic_arg_type_fn: Self::get_type_assessor(parser.settings.mode),
            short_set_iter: None,
        }
    }

    #[inline]
    fn get_type_assessor(mode: OptionsMode) -> ArgTypeAssessor {
        match mode {
            OptionsMode::Standard => get_basic_arg_type_standard,
            OptionsMode::Alternate => get_basic_arg_type_alternate,
        }
    }

    /// Parse next argument, if any
    fn get_next(&mut self) -> Option<ItemClass<'s, str>> {
        let (arg_index, arg) = self.arg_iter.next()?;
        let arg = arg.as_ref();

        let arg_type = match self.early_terminator_encountered {
            true => ArgTypeBasic::NonOption,
            false => (self.get_basic_arg_type_fn)(arg),
        };

        match arg_type {
            ArgTypeBasic::NonOption => {
                if self.try_command_matching && !self.early_terminator_encountered {
                    for candidate in self.parser_data.commands.commands {
                        if candidate.name == arg {
                            self.parser_data.options = candidate.options;
                            self.parser_data.commands = &candidate.sub_commands;
                            return Some(ItemClass::Ok(Item::Command(arg_index, arg)));
                        }
                    }
                }
                self.try_command_matching = false;
                Some(ItemClass::Ok(Item::NonOption(arg_index, arg)))
            },
            ArgTypeBasic::EarlyTerminator => {
                self.early_terminator_encountered = true;
                // Yes, it may be valuable info to the caller to know that one was encountered and
                // where, so let’s not leave it out of the results.
                Some(ItemClass::Ok(Item::EarlyTerminator(arg_index)))
            },
            ArgTypeBasic::ShortOptionSet(optset_string) => {
                // Here we defer to an iterator specific to iterating over the short option set in
                // the non-prefixed portion of the current argument. We will save the iterator in
                // the main iterator, and just return its first `next()` result here.
                let mut short_set_iter = ShortSetIter::new(self, optset_string, arg_index);
                let first = short_set_iter.next();
                self.short_set_iter = Some(short_set_iter);
                first
            },
            ArgTypeBasic::LongOption(opt_string) => {
                /* We need to deal with the fact that arg data may be supplied in the same argument,
                 * separated by an `=`, and also that the user is allowed to supply an abbreviated
                 * form of an available option, so long as it is unique, which requires checking for
                 * ambiguity. (See documentation).
                 */

                // Extract name, splitting from optional “in-same-arg” data value
                let (name, data_included) = match opt_string.find('=') {
                    None => (opt_string, None),
                    Some(i) => {
                        let split = opt_string.split_at(i);
                        // We know that the `=` is encoded as just one byte and that it is
                        // definitely there, so we can safely skip it unchecked.
                        let data_included = unsafe { split.1.get_unchecked(1..) };
                        (split.0, Some(data_included))
                    },
                };

                // This occurs with `--=` or `--=foo` (`-=` or `-=foo` in alt mode)
                if name.is_empty() {
                    return Some(ItemClass::Warn(ItemW::LongWithNoName(arg_index)));
                }

                let mut matched: Option<&LongOption> = None;
                let mut ambiguity = false;
                'l_candidates: for candidate in self.parser_data.options.long {
                    // Exact
                    if candidate.name == name {
                        // An exact match overrules a previously found partial match and
                        // ambiguity found with multiple partial matches.
                        matched = Some(candidate);
                        ambiguity = false;
                        break 'l_candidates;
                    }
                    // Abbreviated
                    else if self.parser_data.settings.allow_abbreviations && !ambiguity
                        && name.len() < candidate.name.len()
                        && candidate.name.starts_with(name)
                    {
                        match matched {
                            Some(_) => { ambiguity = true; },
                            None => { matched = Some(candidate); },
                        }
                    }
                }

                if ambiguity {
                    Some(ItemClass::Err(ItemE::AmbiguousLong(arg_index, name)))
                }
                else if let Some(matched) = matched {
                    // Use option’s full name, not the possibly abbreviated user provided one
                    let opt_name = &matched.name;

                    if matched.expects_data {
                        // Data included in same argument
                        // We accept it even if it’s an empty string
                        if let Some(data) = data_included {
                            Some(ItemClass::Ok(Item::LongWithData {
                                i: arg_index, n: opt_name, d: data, l: DataLocation::SameArg }))
                        }
                        // Data included in next argument
                        else if let Some((_, next_arg)) = self.arg_iter.next() {
                            Some(ItemClass::Ok(Item::LongWithData {
                                i: arg_index, n: opt_name, d: next_arg.as_ref(),
                                l: DataLocation::NextArg }))
                        }
                        // Data missing
                        else {
                            Some(ItemClass::Err(ItemE::LongMissingData(arg_index, opt_name)))
                        }
                    }
                    else {
                        match data_included {
                            None |
                            // Ignore unexpected data if empty string
                            Some("") => {
                                Some(ItemClass::Ok(Item::Long(arg_index, opt_name)))
                            },
                            Some(data) => {
                                Some(ItemClass::Warn(ItemW::LongWithUnexpectedData {
                                    i: arg_index, n: opt_name, d: data }))
                            },
                        }
                    }
                }
                else {
                    // Again, we ignore any possibly included data in the argument
                    Some(ItemClass::Warn(ItemW::UnknownLong(arg_index, name)))
                }
            },
        }
    }
}

impl<'r, 's, A> ShortSetIter<'r, 's, A>
    where A: 's + AsRef<str>, 's: 'r
{
    /// Create a new instance. Note, the provided string should **not** include the dash prefix.
    pub(crate) fn new(parse_iter: &ParseIter<'r, 's, A>, short_set_string: &'s str,
        arg_index: usize) -> Self
    {
        Self {
            arg_iter: parse_iter.arg_iter.clone(),
            parser_data: parse_iter.parser_data,
            string: short_set_string,
            iter: short_set_string.char_indices(),
            arg_index: arg_index,
            consumed: false,
        }
    }

    /// Get next item, if any
    fn get_next(&mut self) -> Option<ItemClass<'s, str>> {
        if self.consumed {
            return None;
        }

        let (byte_pos, ch) = self.iter.next()?;

        let mut match_found = false;
        let mut expects_data = false;
        's_candidates: for candidate in self.parser_data.options.short {
            if candidate.ch == ch {
                match_found = true;
                expects_data = candidate.expects_data;
                break 's_candidates;
            }
        }

        if !match_found {
            Some(ItemClass::Warn(ItemW::UnknownShort(self.arg_index, ch)))
        }
        else if !expects_data {
            Some(ItemClass::Ok(Item::Short(self.arg_index, ch)))
        }
        else {
            let next_char_byte_pos = byte_pos + ch.len_utf8();

            // If not last char, remaining chars are our data
            if next_char_byte_pos < self.string.len() {
                self.consumed = true;
                let data = self.string.split_at(next_char_byte_pos).1;
                Some(ItemClass::Ok(Item::ShortWithData {
                    i: self.arg_index, c: ch, d: data, l: DataLocation::SameArg }))
            }
            // Data included in next argument
            else if let Some((_, next_arg)) = self.arg_iter.next() {
                Some(ItemClass::Ok(Item::ShortWithData {
                    i: self.arg_index, c: ch, d: next_arg.as_ref(),
                    l: DataLocation::NextArg }))
            }
            // Data missing
            else {
                Some(ItemClass::Err(ItemE::ShortMissingData(self.arg_index, ch)))
            }
        }
    }
}

// Check if `arg` has the given prefix.
//
// This is similar to a `starts_with` check, but the length must be longer than the prefix, equal
// length is no good.
#[inline(always)]
fn has_prefix(arg: &str, prefix: &str) -> bool {
    // Note, it is safe to index into `arg` in here; we don’t care about char boundaries for the
    // simple byte-slice comparison. Doing this is optimally efficient, avoiding `start_with`’s
    // `>=` length comparison check, as well as utf-8 char boundary checks, etc.
    let prefix_len = prefix.len();
    arg.len() > prefix_len && &prefix.as_bytes()[..] == &arg.as_bytes()[..prefix_len]
}

/// Assess argument type, returning options without their prefix, for “standard” mode
fn get_basic_arg_type_standard(arg: &str) -> ArgTypeBasic<'_> {
    if arg == EARLY_TERMINATOR {
        ArgTypeBasic::EarlyTerminator
    }
    else if has_prefix(arg, DOUBLE_DASH_PREFIX) {
        ArgTypeBasic::LongOption(unsafe { arg.get_unchecked(DOUBLE_DASH_PREFIX.len()..) })
    }
    else if has_prefix(arg, SINGLE_DASH_PREFIX) {
        ArgTypeBasic::ShortOptionSet(unsafe { arg.get_unchecked(SINGLE_DASH_PREFIX.len()..) })
    }
    else {
        ArgTypeBasic::NonOption
    }
}

/// Assess argument type, returning options without their prefix, for “alternate” mode
fn get_basic_arg_type_alternate(arg: &str) -> ArgTypeBasic<'_> {
    if arg == EARLY_TERMINATOR {
        ArgTypeBasic::EarlyTerminator
    }
    else if has_prefix(arg, SINGLE_DASH_PREFIX) {
        ArgTypeBasic::LongOption(unsafe { arg.get_unchecked(SINGLE_DASH_PREFIX.len()..) })
    }
    else {
        ArgTypeBasic::NonOption
    }
}
