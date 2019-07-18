// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

use std::borrow::Cow;
use std::char::REPLACEMENT_CHARACTER;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::iter::Enumerate;
use std::mem;
#[cfg(any(unix, target_os = "redox"))]
use std::os::unix::ffi::OsStrExt;
use std::slice;
use std::str::CharIndices;
use crate::commands::CommandSet;
use crate::parser::*;
use crate::options::*;
use crate::analysis::*;

type ArgTypeAssessor = fn(&OsStr) -> ArgTypeBasic<'_>;

/// An argument list parsing iterator
///
/// Created by the [`parse_iter`] method of [`Parser`].
///
/// [`parse_iter`]: ../parser/struct.Parser.html#method.parse_iter
/// [`Parser`]: ../parser/struct.Parser.html
///
/// Note that methods are provided for changing the *option set* and *command set* used for
/// subsequent iterations. These are typically only applicable where you are using the iterative
/// parsing style with a command based program, where instead of describing the entire command
/// structure to the parser up front, you want to dynamically switch the sets used for subsequent
/// iterations (arguments) manually, after encountering a command.
#[derive(Clone)]
pub struct ParseIter<'r, 's: 'r, A: 's + AsRef<OsStr>> {
    /// Enumerated iterator over the argument list
    arg_iter: Enumerate<slice::Iter<'s, A>>,
    /// The parser data in use (will change on encountering a command)
    parser_data: Parser<'r, 's>,
    /// Whether or not all remaining arguments should be interpreted as positionals (`true` if
    /// either an early terminator has been encountered, or “posixly correct” behaviour is required
    /// and a positional has been encountered).
    rest_are_positionals: bool,
    /// A positional is only assessed as being a possible command if 1) it is the first encountered
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
struct ShortSetIter<'r, 's: 'r, A: 's + AsRef<OsStr>> {
    /// Enumerated iterator over the argument list
    arg_iter: Enumerate<slice::Iter<'s, A>>,
    /// The parser data in use
    parser_data: Parser<'r, 's>,
    /// The short option set string being iterated over.
    /// We need to hold a copy of this at least for the purpose of extracting in-same-arg data.
    string: &'s OsStr,
    /// A lossy UTF-8 conversion of the string
    string_utf8: Cow<'r, str>,
    /// Char iterator over the lossily converted UTF-8 string
    iter: CharIndices<'r>,
    /// Bytes consumed in the original `OsStr`, used for extraction of an in-same-arg data value.
    bytes_consumed: usize,
    /// Index of argument the set came from, for recording in items
    arg_index: usize,
    /// For marking as fully consumed when remaining portion of the string has been consumed as the
    /// data value of a short option, bypassing the fact that the char iterator has not finished.
    consumed: bool,
}

/// Basic argument type
///
/// Option variants should: include argument without prefix; include “in-same-arg” data values.
#[derive(Debug)]
enum ArgTypeBasic<'a> {
    NonOption,
    EarlyTerminator,
    LongOption(&'a OsStr),
    ShortOptionSet(&'a OsStr),
}

impl<'r, 's, A> Iterator for ParseIter<'r, 's, A>
    where A: 's + AsRef<OsStr>, 's: 'r
{
    type Item = ItemResult<'s>;

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
    where A: 's + AsRef<OsStr>, 's: 'r
{
    type Item = ItemResult<'s>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Do next short, if there is one
        self.get_next()
    }
}

impl<'r, 's, A> ParseIter<'r, 's, A>
    where A: 's + AsRef<OsStr>, 's: 'r
{
    /// Create a new instance
    pub(crate) fn new(args: &'s [A], parser: &Parser<'r, 's>) -> Self {
        Self {
            arg_iter: args.iter().enumerate(),
            parser_data: *parser,
            rest_are_positionals: false,
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

    /// Get the *option set* currently in use for parsing
    ///
    /// This is useful for suggestion matching of unknown options
    #[inline(always)]
    pub fn get_option_set(&self) -> &'r OptionSet<'r, 's> {
        self.parser_data.options
    }

    /// Change the *option set* used for parsing by subsequent iterations
    ///
    /// This is typically only applicable where you are using the iterative parsing style with a
    /// command based program, where instead of describing the entire command structure to the
    /// parser up front, you want to dynamically switch out the *option set* used for subsequent
    /// iterations (arguments) manually, after encountering a command.
    ///
    /// Note, it is undefined behaviour to set a non-valid option set.
    pub fn set_option_set(&mut self, opt_set: &'r OptionSet<'r, 's>) {
        self.parser_data.options = opt_set;
        if let Some(ref mut short_set_iter) = self.short_set_iter {
            short_set_iter.parser_data.options = opt_set;
        }
    }

    /// Get the *command set* currently in use for parsing
    ///
    /// This is useful for suggestion matching of an unknown command
    #[inline(always)]
    pub fn get_command_set(&self) -> &'r CommandSet<'r, 's> {
        self.parser_data.commands
    }

    /// Change the *command set* used for parsing by subsequent iterations
    ///
    /// This is typically only applicable where you are using the iterative parsing style with a
    /// command based program, where instead of describing the entire command structure to the
    /// parser up front, you want to dynamically switch out the *command set* used for subsequent
    /// iterations (arguments) manually, after encountering a command.
    ///
    /// Note, it is undefined behaviour to set a non-valid command set.
    pub fn set_command_set(&mut self, cmd_set: &'r CommandSet<'r, 's>) {
        self.parser_data.commands = cmd_set;
        if let Some(ref mut short_set_iter) = self.short_set_iter {
            short_set_iter.parser_data.commands = cmd_set;
        }
    }

    /// Get copy of parser settings in use
    ///
    /// The point of this is for use in situations where `set_parse_settings` might be used, where
    /// a copy of the original settings are wanted for modification before applying on the iterator,
    /// avoiding the need for a pointer to the original parser object.
    #[inline]
    pub fn get_parse_settings(&self) -> Settings {
        self.parser_data.settings
    }

    /// Change the settings used for parsing by subsequent iterations
    ///
    /// The use case for this method is similar to that of the methods for changing the *option
    /// set* and *command set* to be used, though more niche. It is thought unlikely that any
    /// program should have any need to change settings in the middle of parsing, but you can if you
    /// absolutely want to (there is no reason to prevent you from doing so).
    pub fn set_parse_settings(&mut self, settings: Settings) {
        self.parser_data.settings = settings;
        if let Some(ref mut short_set_iter) = self.short_set_iter {
            short_set_iter.parser_data.settings = settings;
        }
        self.get_basic_arg_type_fn = Self::get_type_assessor(settings.mode);
    }

    /// Parse next argument, if any
    fn get_next(&mut self) -> Option<ItemResult<'s>> {
        let (arg_index, arg) = self.arg_iter.next()?;
        let arg = arg.as_ref();

        let arg_type = match self.rest_are_positionals {
            true => ArgTypeBasic::NonOption,
            false => (self.get_basic_arg_type_fn)(arg),
        };

        match arg_type {
            ArgTypeBasic::NonOption => {
                // This may be a positional or a command
                if !self.rest_are_positionals {
                    if self.try_command_matching {
                        match find_name_match(arg, self.parser_data.commands.commands.iter(),
                            |&c| { c.name }, self.parser_data.settings.allow_cmd_abbreviations)
                        {
                            Err(_) => {
                                return Some(Err(ProblemItem::AmbiguousCmd(arg_index, arg)));
                            },
                            Ok(Some(matched)) => {
                                self.parser_data.options = matched.options;
                                self.parser_data.commands = &matched.sub_commands;
                                return Some(Ok(Item::Command(arg_index, matched.name)));
                            },
                            Ok(None) => { /* fall through */ },
                        }
                        self.try_command_matching = false;
                        if !self.parser_data.commands.commands.is_empty() {
                            return Some(Err(ProblemItem::UnknownCommand(arg_index, arg)));
                        }
                    }
                    if self.parser_data.settings.posixly_correct {
                        self.rest_are_positionals = true;
                    }
                }
                Some(Ok(Item::Positional(arg_index, arg)))
            },
            ArgTypeBasic::EarlyTerminator => {
                self.rest_are_positionals = true;
                // Yes, it may be valuable info to the caller to know that one was encountered and
                // where, so let’s not leave it out of the results.
                Some(Ok(Item::EarlyTerminator(arg_index)))
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
                let (name, data_included) = split_long_components(opt_string);

                // This occurs with `--=` or `--=foo` (`-=` or `-=foo` in alt mode)
                if name.is_empty() {
                    return Some(Err(ProblemItem::UnknownLong(arg_index, OsStr::new(""))));
                }

                let match_result = find_name_match(name, self.parser_data.options.long.iter(),
                    |&o| { o.name }, self.parser_data.settings.allow_opt_abbreviations);

                if match_result.is_err() {
                    Some(Err(ProblemItem::AmbiguousLong(arg_index, name)))
                }
                else if let Ok(Some(matched)) = match_result {
                    // Use option’s full name, not the possibly abbreviated user provided one
                    let opt_name = matched.name;

                    if matched.opt_type != OptionType::Flag {
                        // Data included in same argument
                        // We accept it even if it’s an empty string
                        if let Some(data) = data_included {
                            Some(Ok(Item::Long(
                                arg_index, opt_name, Some((data, DataLocation::SameArg))
                            )))
                        }
                        // Data consumption is optional
                        else if matched.opt_type == OptionType::OptionalData {
                            Some(Ok(Item::Long(
                                arg_index, opt_name, Some((OsStr::new(""), DataLocation::SameArg))
                            )))
                        }
                        // Data included in next argument
                        else if let Some((_, next_arg)) = self.arg_iter.next() {
                            Some(Ok(Item::Long(
                                arg_index, opt_name, Some((next_arg.as_ref(), DataLocation::NextArg))
                            )))
                        }
                        // Data missing
                        else {
                            Some(Err(ProblemItem::LongMissingData(arg_index, opt_name)))
                        }
                    }
                    // Ignore unexpected data if empty string
                    else if data_included.is_none() || data_included == Some(OsStr::new("")) {
                        Some(Ok(Item::Long(arg_index, opt_name, None)))
                    }
                    else {
                        let data = data_included.unwrap();
                        Some(Err(ProblemItem::LongWithUnexpectedData {
                            i: arg_index, n: opt_name, d: data
                        }))
                    }
                }
                else {
                    // Again, we ignore any possibly included data in the argument
                    Some(Err(ProblemItem::UnknownLong(arg_index, name)))
                }
            },
        }
    }
}

impl<'r, 's, A> ShortSetIter<'r, 's, A>
    where A: 's + AsRef<OsStr>, 's: 'r
{
    /// Create a new instance. Note, the provided string should **not** include the dash prefix.
    pub(crate) fn new(parse_iter: &ParseIter<'r, 's, A>, short_set_string: &'s OsStr,
        arg_index: usize) -> Self
    {
        // Note, both the lossy converted string and the char iterator over it will live within the
        // struct, and will have the same lifetime. We are forced however to transmute the lifetime
        // of the borrow in creating the iterator to achieve this, but we know there will be no
        // problem doing so.
        let lossy = short_set_string.to_string_lossy();
        let lossy_ref = unsafe { mem::transmute::<&'_ Cow<str>, &'r Cow<str>>(&lossy) };
        let iter = lossy_ref.char_indices();
        Self {
            arg_iter: parse_iter.arg_iter.clone(),
            parser_data: parse_iter.parser_data,
            string: short_set_string,
            string_utf8: lossy,
            iter: iter,
            bytes_consumed: 0,
            arg_index: arg_index,
            consumed: false,
        }
    }

    /// Get next item, if any
    fn get_next(&mut self) -> Option<ItemResult<'s>> {
        if self.consumed {
            return None;
        }

        let (byte_pos, ch) = self.iter.next()?;

        let mut match_found = false;
        let mut opt_type = OptionType::Flag;

        match ch {
            // If we encounter the Unicode replacement character (U+FFFD) then we must beware that
            // this may have come from either a real such character, or from a byte sequence that
            // cannot be converted to valid UTF-8 in the original `OsStr`. To handle the latter, the
            // former is not allowed as a valid short option character.
            REPLACEMENT_CHARACTER => {
                // Here all we do is update a byte index such that any in-same-arg data value can be
                // correctly extracted later.

                // Init tracking?
                if self.bytes_consumed == 0 {
                    self.bytes_consumed = byte_pos;
                }

                // On Unix, we need to put in some effort to figure out how many bytes to consume
                #[cfg(not(windows))] {
                    let slice = OsStr::from_bytes(&self.string.as_bytes()[self.bytes_consumed..]);
                    self.bytes_consumed += get_urc_bytes(slice);
                }
                // On Windows, the `OsStr` is also holding a UTF-8 based byte sequence (WTF-8
                // format), but lossy conversion is a WTF-8 to UTF-8 conversion, which does the
                // minimal amount of work (only swapping encodings of code points U+D800 to U+DFFF),
                // and such code points come out as a single replacement character, unlike on Unix
                // where there is one per byte (3) for such code points.
                //
                // Actual replacement characters are three bytes (note that as per the Unix
                // implementation we can ignore the possibility of the ‘overlong’ 4-byte form), and
                // the encodings of “unpaired surrogate” code points (U+D800 to U+DFFF) are also
                // three bytes, thus the answer here is always three.
                #[cfg(windows)] {
                    self.bytes_consumed += 3;
                }
            },
            // Not a Unicode replacement character, so lets try to match it
            _ => {
                for candidate in self.parser_data.options.short {
                    if candidate.ch == ch {
                        match_found = true;
                        opt_type = candidate.opt_type;
                        break;
                    }
                }

                // Tracking?
                if self.bytes_consumed != 0 {
                    self.bytes_consumed += ch.len_utf8();
                }
            }
        }

        if !match_found {
            Some(Err(ProblemItem::UnknownShort(self.arg_index, ch)))
        }
        else if opt_type == OptionType::Flag {
            Some(Ok(Item::Short(self.arg_index, ch, None)))
        }
        else {
            let bytes_consumed_updated = byte_pos + ch.len_utf8();

            // If not last char, remaining chars are our data
            if bytes_consumed_updated < self.string_utf8.len() {
                self.consumed = true;
                if self.bytes_consumed == 0 {
                    self.bytes_consumed = bytes_consumed_updated;
                }
                let data = OsStr::from_bytes(&self.string.as_bytes()[self.bytes_consumed..]);
                Some(Ok(Item::Short(
                    self.arg_index, ch, Some((data, DataLocation::SameArg))
                )))
            }
            // Data consumption is optional
            else if opt_type == OptionType::OptionalData {
                Some(Ok(Item::Short(
                    self.arg_index, ch, Some((OsStr::new(""), DataLocation::SameArg))
                )))
            }
            // Data included in next argument
            else if let Some((_, next_arg)) = self.arg_iter.next() {
                Some(Ok(Item::Short(
                    self.arg_index, ch, Some((next_arg.as_ref(), DataLocation::NextArg))
                )))
            }
            // Data missing
            else {
                Some(Err(ProblemItem::ShortMissingData(self.arg_index, ch)))
            }
        }
    }
}

// Check if `arg` has the given prefix.
//
// This is similar to a `starts_with` check, but the length must be longer than the prefix, equal
// length is no good.
#[inline(always)]
fn has_prefix(arg: &OsStr, prefix: &OsStr) -> bool {
    // Note, it is safe to index into `arg` in here; we don’t care about char boundaries for the
    // simple byte-slice comparison. Doing this is optimally efficient, avoiding `start_with`’s
    // `>=` length comparison check, as well as utf-8 char boundary checks, etc.
    let prefix_len = prefix.len();
    arg.len() > prefix_len && &prefix.as_bytes()[..] == &arg.as_bytes()[..prefix_len]
}

/// Assess argument type, returning options without their prefix, for “standard” mode
fn get_basic_arg_type_standard(arg: &OsStr) -> ArgTypeBasic<'_> {
    if arg == OsStr::new("--") {
        ArgTypeBasic::EarlyTerminator
    }
    else if has_prefix(arg, OsStr::new("--")) {
        ArgTypeBasic::LongOption(OsStr::from_bytes(&arg.as_bytes()[2..]))
    }
    else if has_prefix(arg, OsStr::new("-")) {
        ArgTypeBasic::ShortOptionSet(OsStr::from_bytes(&arg.as_bytes()[1..]))
    }
    else {
        ArgTypeBasic::NonOption
    }
}

/// Assess argument type, returning options without their prefix, for “alternate” mode
fn get_basic_arg_type_alternate(arg: &OsStr) -> ArgTypeBasic<'_> {
    if arg == OsStr::new("--") {
        ArgTypeBasic::EarlyTerminator
    }
    else if has_prefix(arg, OsStr::new("-")) {
        ArgTypeBasic::LongOption(OsStr::from_bytes(&arg.as_bytes()[1..]))
    }
    else {
        ArgTypeBasic::NonOption
    }
}

/// Splits the name and optional in-same-argument data value component from a long option (with
/// prefix already stripped).
#[inline]
fn split_long_components(string: &OsStr) -> (&'_ OsStr, Option<&'_ OsStr>) {
    let bytes = string.as_bytes();

    let mut separator = None;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'=' {
            separator = Some(i);
            break;
        }
    }
    match separator {
        Some(i) => (OsStr::from_bytes(&bytes[..i]), Some(OsStr::from_bytes(&bytes[i+1..]))),
        None => (string, None),
    }
}

/// Returns the number of bytes that would result in a single Unicode replacement character, from
/// the start of the string, if the string went through a lossy conversion to `str`.
///
/// This assumes that the start of the string is a point from which lossy conversion would
/// definitely result in a replacement character, whether from a real replacement character, or from
/// one or more invalid bytes.
#[cfg(not(windows))]
#[cold]
fn get_urc_bytes(string: &OsStr) -> usize {
    let as_bytes = string.as_bytes();

    // Did a replacement character come from a real replacement character?
    //
    // A real replacement character (U+FFFD) encoded is 0xEFBFBD (3 bytes).
    //
    // Note, we do not need to worry about the ‘overlong’ 4-byte form (0xF08FBFBD). This is a
    // correctly formed encoding of the code point, but is invalid UTF-8 since valid UTF-8 only
    // allows a one-to-one mapping between code-points and encoding form, thus with a requirement
    // that decoders reject ‘overlong’ forms, as the Rust `std` library does.
    match as_bytes.get(..3) == Some(&[0xef, 0xbf, 0xbd]) {
        true => 3,
        false => {
            // On Unix/Redox, the `OsStr` is holding a UTF-8 based byte sequence, which is lossily
            // converted to `str` via the same code as `std::str::from_utf8`, thus we can use that.
            //
            // Note, while the function calls here potentially processes the entire string to check
            // validity, which we don’t want that to happen for efficiency; we know here that it
            // will only be run in the case of invalid bytes, and considering how invalid byte
            // sequences are converted to unicode replacement characters, we know it will only
            // process 1-4 bytes, and there is no point in trying to enforce that with the slice
            // that we give it.
            match std::str::from_utf8(as_bytes) {
                Err(e) => match e.error_len() {
                    Some(i) => i,
                    None => as_bytes.len() - e.valid_up_to(),
                },
                Ok(_) => unreachable!(),
            }
        },
    }
}

#[cfg(windows)]
pub trait OsStrExt {
    fn from_bytes(slice: &[u8]) -> &Self;
    fn as_bytes(&self) -> &[u8];
}

#[cfg(windows)]
impl OsStrExt for OsStr {
    #[inline(always)]
    fn from_bytes(slice: &[u8]) -> &OsStr {
        unsafe { mem::transmute(slice) }
    }
    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        unsafe { mem::transmute(self) }
    }
}

/// Find a match for something with a name (long option or command), optionally allowing for
/// abbreviations
fn find_name_match<'a, T>(needle: &OsStr, haystack: impl Iterator<Item = &'a T>,
    get_name: fn(&'a T) -> &'a str, abbreviations: bool) -> Result<Option<&'a T>, ()>
{
    let mut matched: Option<&T> = None;
    let mut ambiguity = false;
    for candidate in haystack {
        let cand_name = get_name(candidate);
        // Exact
        if cand_name == needle {
            // An exact match overrules a previously found partial match and ambiguity found with
            // multiple partial matches.
            matched = Some(candidate);
            ambiguity = false;
            break;
        }
        // Abbreviated
        else if abbreviations && !ambiguity {
            let cand_name_osstr = OsStr::new(cand_name);
            if needle.len() < cand_name_osstr.len() {
                if &cand_name_osstr.as_bytes()[..needle.len()] == needle.as_bytes() {
                    match matched {
                        Some(_) => { ambiguity = true; },
                        None => { matched = Some(candidate); },
                    }
                }
            }
        }
    }
    match ambiguity {
        true => Err(()),
        false => Ok(matched),
    }
}
