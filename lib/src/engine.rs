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
use std::slice;
use std::str::CharIndices;
use crate::analysis::*;
use crate::arguments::Args;
use crate::commands::CommandSet;
use crate::matching::{SearchResult, NameSearchResult, OsStrExt};
use crate::options::*;
use crate::parser::*;
use crate::positionals::{Quantity as PositionalsQuantity, Policy as PositionalsPolicy,
                         SimplePolicy as PositionalsSimplePolicy};

/// An argument list parsing iterator
///
/// Created by the [`parse_iter`] method of [`Parser`].
///
/// [`parse_iter`]: ../parser/struct.Parser.html#method.parse_iter
/// [`Parser`]: ../parser/struct.Parser.html
#[derive(Clone)]
pub struct ParseIter<'r, 'set, 'arg, A> where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r {
    /// Enumerated iterator over the argument list
    arg_iter: Enumerate<slice::Iter<'arg, A>>,
    /// The option set in use (will change on encountering a command)
    options: &'r OptionSet<'r, 'set>,
    /// Settings
    settings: Settings,
    /// Whether or not all remaining arguments should be interpreted as positionals (`true` if
    /// either an early terminator has been encountered, or “posixly correct” behaviour is required
    /// and a positional has been encountered).
    rest_are_positionals: bool,
    /// Whether or not to consider non-options as possible commands rather than just positionals
    ///
    /// Interpretation of non-options as commands should only occur if command parsing is actually
    /// being done (this being used through the command-based iterator); if we have not yet served a
    /// positional item (since positionals can only occur **after** commands); and if
    /// `rest_are_positionals` is `false` of course.
    try_command_matching: bool,
    /// Whether or not we have gone beyond the maximum number of positionals allowed per policy
    too_many_positionals: bool,
    /// Requirements of positionals
    positionals_policy: PositionalsSimplePolicy,
    /// Number of positionals encounterd
    positionals_count: PositionalsQuantity,
    /// Cached index of previous item
    last_index: usize,
    /// Cached data-location of previous item
    last_data_loc: Option<DataLocation>,
    /// Short option set argument iterator
    short_set_iter: Option<ShortSetIter<'r, 'arg>>,
}

/// A short option set string iterator
#[derive(Clone)]
struct ShortSetIter<'r, 'arg: 'r> {
    /// The short option set string being iterated over.
    /// We need to hold a copy of this at least for the purpose of extracting in-same-arg data.
    string: &'arg OsStr,
    /// A lossy UTF-8 conversion of the string
    string_utf8: Cow<'r, str>,
    /// Char iterator over the lossily converted UTF-8 string
    iter: CharIndices<'r>,
    /// Bytes consumed in the original `OsStr`, used for extraction of an in-same-arg data value.
    bytes_consumed: usize,
    /// For marking as fully consumed when remaining portion of the string has been consumed as the
    /// data value of a short option, bypassing the fact that the char iterator has not finished.
    consumed: bool,
}

/// An argument list parsing iterator, command based
///
/// Created by the [`parse_iter`] method of [`CmdParser`].
///
/// [`parse_iter`]: ../parser/struct.CmdParser.html#method.parse_iter
/// [`CmdParser`]: ../parser/struct.CmdParser.html
///
/// Note that methods are provided for changing the *option set* and *command set* used for
/// subsequent iterations. These are typically only applicable where you are using the iterative
/// parsing style with a command based program, where instead of describing the entire command
/// structure to the parser up front, you want to dynamically switch the sets used for subsequent
/// iterations (arguments) manually, after encountering a command.
#[derive(Clone)]
pub struct CmdParseIter<'r, 'set, 'arg, A> where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r {
    /// The command set in use (will change on encountering a command)
    commands: &'r CommandSet<'r, 'set>,
    /// Inner main iterator
    inner: ParseIter<'r, 'set, 'arg, A>,
}

/// An argument list parsing iterator, bundling extra data
///
/// This bundles items along with their respective argument indexes in a tuple, in a similar way to
/// how `enumerate()` on a generic iterator bundles a count index.
///
/// Created by the [`indexed`] method of [`ParseIter`].
///
/// [`indexed`]: struct.ParseIter.html#method.indexed
/// [`ParseIter`]: struct.ParseIter.html
#[derive(Clone)]
pub struct ParseIterIndexed<'r, 'set, 'arg, A> where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    inner: ParseIter<'r, 'set, 'arg, A>,
}

/// An argument list parsing iterator, bundling extra data
///
/// This bundles items along with their respective argument indexes in a tuple, in a similar way to
/// how `enumerate()` on a generic iterator bundles a count index.
///
/// Created by the [`indexed`] method of [`CmdParseIter`].
///
/// [`indexed`]: struct.CmdParseIter.html#method.indexed
/// [`CmdParseIter`]: struct.CmdParseIter.html
#[derive(Clone)]
pub struct CmdParseIterIndexed<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    inner: CmdParseIter<'r, 'set, 'arg, A>,
}

/// Basic argument type
///
/// Option variants should: include argument without prefix; include “in-same-arg” data values.
enum ArgTypeBasic<'a> {
    NonOption,
    EarlyTerminator,
    LongOption(&'a OsStr),
    ShortOptionSet(&'a OsStr),
}

impl<'r, 'set, 'arg, A> Iterator for ParseIter<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    type Item = ItemResult<'set, 'arg>;

    fn next(&mut self) -> Option<Self::Item> {
        // Continue from where we left off for a short option set?
        if self.short_set_iter.is_some() {
            let mut short_set_iter = self.short_set_iter.take().unwrap();
            match short_set_iter.get_next(self) {
                Some(result) => {
                    self.short_set_iter = Some(short_set_iter); // Move it back
                    return Some(result);
                },
                None => self.short_set_iter = None,
            }
        }
        // Do next argument, if there is one
        // Ensure that we issue a missing-positionals item if necessary
        match self.get_next() {
            Some(item) => Some(item),
            None => match self.positionals_policy.get_remaining_min(self.positionals_count) {
                0 => None,
                r => {
                    // Note, we can set an appropriate data-location value, but we cannot set an
                    // appropriate index, since there is no index to associate with this, and it is
                    // not worth adding an `Option` wrapper for setting `None` as would be
                    // appropriate, so we just allow that to remain as per previous item.
                    self.last_data_loc = None;
                    // Clear such that next iteration actually produces `None`
                    self.positionals_policy = PositionalsPolicy::Min(0).into();
                    // Report too-few positionals difference as missing
                    Some(Err(ProblemItem::MissingPositionals(r)))
                },
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Note that the `Iterator` documentation for implementation of this method states that
        // although the hint should not be absolutely relied upon to be accurate and there is no
        // enforcement that it gives up the stated number, an iterator would be considered to be
        // buggy if it did not conform to its stated bounds.
        //
        // Hence, while it might be nice to just return the arg-iter lower bound as our lower bound
        // here, as a best-estimate hint (for collecting into a `Vec`), assuming most arguments will
        // not consume an in-next-arg data value argument, this would go against the specification.

        let arg_iter_size_hint = self.arg_iter.size_hint();
        // The absolute minimum lower bound is influenced by the fact that option arguments can
        // potentially consume a single additional argument as an in-next-arg data value; we can
        // thus be certain that it will be no lower than (n/2 + n%2).
        let lower = (arg_iter_size_hint.0 / 2) + (arg_iter_size_hint.0 % 2);
        // We cannot give any sort of reliable upper bound since short option set arguments can
        // expand potentially to very large numbers of short options, which is completely
        // unpredictable prior to actual parsing. (Also missing-positionals adds one).
        let upper = None;
        (lower, upper)
    }
}

impl<'r, 'set, 'arg, A> Iterator for CmdParseIter<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    type Item = ItemResult<'set, 'arg>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next();
        if self.inner.try_command_matching {
            if let Some(Ok(Item::Positional(arg))) = next {
                let lookup = match self.inner.settings.allow_cmd_abbreviations {
                    false => crate::matching::find_by_name(arg,
                        self.commands.commands.iter(), |&c| { c.name }).into(),
                    true => crate::matching::find_by_abbrev_name(arg,
                        self.commands.commands.iter(), |&c| { c.name }),
                };
                match lookup {
                    NameSearchResult::Match(matched) |
                    NameSearchResult::AbbreviatedMatch(matched) => {
                        self.commands = &matched.sub_commands;
                        self.inner.options = matched.options;
                        self.inner.positionals_policy = matched.positionals_policy.into();
                        return Some(Ok(Item::Command(matched.name)));
                    },
                    NameSearchResult::AmbiguousMatch => {
                        return Some(Err(ProblemItem::AmbiguousCmd(arg)));
                    },
                    NameSearchResult::NoMatch => {
                        self.inner.try_command_matching = false;
                        if !self.commands.commands.is_empty() &&
                            self.inner.positionals_policy.max == 0
                        {
                            let suggestion = match self.inner.serve_suggestions() {
                                #[cfg(feature = "suggestions")]
                                true => self.commands.suggest(arg),
                                _ => None,
                            };
                            return Some(Err(ProblemItem::UnknownCommand(arg, suggestion)));
                        }
                        /* fall through */
                    },
                }
                self.inner.register_positional();
                if self.inner.too_many_positionals {
                    return Some(Err(ProblemItem::UnexpectedPositional(arg)));
                }
            }
        }
        next
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'r, 'set, 'arg, A> Iterator for ParseIterIndexed<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    type Item = ItemResultIndexed<'set, 'arg>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| (self.inner.get_last_index(), item, self.inner.get_last_dataloc()))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'r, 'set, 'arg, A> Iterator for CmdParseIterIndexed<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    type Item = ItemResultIndexed<'set, 'arg>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| (self.inner.get_last_index(), item, self.inner.get_last_dataloc()))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'r, 'set, 'arg, A> ParseIter<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    /// Create a new instance
    #[inline(always)]
    pub(crate) fn new(args: &'arg Args<A>, parser: &Parser<'r, 'set>) -> Self {
        Self::new_inner(args, parser, false)
    }

    /// Create a new instance
    fn new_inner(args: &'arg Args<A>, parser: &Parser<'r, 'set>, command_mode: bool) -> Self {
        parser.positionals_policy.assert_valid();
        Self {
            arg_iter: args.as_slice().iter().enumerate(),
            options: parser.options,
            settings: parser.settings,
            rest_are_positionals: false,
            try_command_matching: command_mode,
            too_many_positionals: false,
            positionals_policy: parser.positionals_policy.into(),
            positionals_count: 0,
            last_index: 0,
            last_data_loc: None,
            short_set_iter: None,
        }
    }

    /// Wraps the iterator in one which also returns extra data
    ///
    /// Specifically this extra data amounts to argument index number and, where applicable, the
    /// location that option data was obtained from (same argument or next).
    #[inline(always)]
    pub fn indexed(self) -> ParseIterIndexed<'r, 'set, 'arg, A> {
        ParseIterIndexed { inner: self }
    }

    /// Get the input argument index of the previous item
    ///
    /// Note, if `next()` has not yet been called, zero will be returned (we do not bother with an
    /// `Option` wrapper). If the iterator has been fully consumed, it will continue to return the
    /// index of the last item.
    #[inline(always)]
    pub fn get_last_index(&self) -> usize {
        self.last_index
    }

    /// Get the data-location of the previous item, if any
    ///
    /// This is used to query whether a data value provided with an option was supplied within the
    /// same argument or in the next argument, should you wish to know.
    ///
    /// `None` will be returned if this does not apply to the previous item, and similarly if
    /// `next()` has not yet been called. If the iterator has been fully consumed, it will continue
    /// to return the last value seen.
    #[inline(always)]
    pub fn get_last_dataloc(&self) -> Option<DataLocation> {
        self.last_data_loc
    }

    /// Get the count of the number of positionals so far
    ///
    /// The counter is incremented on serving each positional, but **not** when serving an
    /// unexpected-positional problem item.
    #[inline(always)]
    pub fn get_positionals_count(&self) -> PositionalsQuantity {
        self.positionals_count
    }

    /// Update the policy for positionals
    ///
    /// This is provided to assist in situations where the policy needs to change dynamically, for
    /// instance in response to use of a particular option.
    ///
    /// This replaces the policy against which the number of positionals encountered is assessed
    /// upon encountering each positional item, so numbers need to be adjusted **without**
    /// accounting for the number returned so far. Thus, if your original policy was say `Fixed(3)`
    /// and you need to allow for an additional instance, and one has already been returned, you
    /// ignore the last fact and set to `Fixed(4)`.
    ///
    /// This will fail and return `Err` if used after an unexpected-positional item has already been
    /// issued; otherwise it will return `Ok`. You can freely change otherwise, even if lowering
    /// the number of positionals that should be accepted below the number already returned (though
    /// it would make no sense to do so).
    ///
    /// Panics on invalid policy.
    #[inline(always)]
    pub fn set_positionals_policy(&mut self, policy: PositionalsPolicy) -> Result<(), ()> {
        policy.assert_valid();
        // Check that it is acceptable to make a change. If the number of positionals has already
        // gone over the max-limit, if there was one, then we cannot allow this to be changed, else
        // it just messes with the correctness of the output; it could allow subsequent positionals
        // to start being issued in good (expected) form again, which would be wrong, or if lowered
        // then it just has no impact and makes no sense. Ultimately it just comes down to there
        // being no valid case of making an update in such circumstances.
        match self.too_many_positionals {
            false => { self.positionals_policy = policy.into(); Ok(()) },
            true => Err(()),
        }
    }

    // Used by creation of `ItemSet` from iterator only
    #[inline(always)]
    pub(crate) fn get_parse_settings(&self) -> &Settings {
        &self.settings
    }

    /// Parse next argument, if any
    fn get_next(&mut self) -> Option<ItemResult<'set, 'arg>> {
        let (arg_index, arg) = self.arg_iter.next()?;
        let arg = arg.as_ref();

        self.last_index = arg_index;
        self.last_data_loc = None;

        let arg_type = match (self.rest_are_positionals, self.settings.mode) {
            (true, _) => ArgTypeBasic::NonOption,
            (false, OptionsMode::Standard) => get_basic_arg_type_standard(arg),
            (false, OptionsMode::Alternate) => get_basic_arg_type_alternate(arg),
        };

        match arg_type {
            ArgTypeBasic::NonOption => {
                match self.try_command_matching {
                    true => Some(Ok(Item::Positional(arg))), // Defer to command handling wrapper
                    false => {
                        self.register_positional();
                        match self.too_many_positionals {
                            false => Some(Ok(Item::Positional(arg))),
                            true => Some(Err(ProblemItem::UnexpectedPositional(arg))),
                        }
                    },
                }
            },
            ArgTypeBasic::EarlyTerminator => {
                self.rest_are_positionals = true;
                self.try_command_matching = false;
                match self.settings.report_earlyterm {
                    true => Some(Ok(Item::EarlyTerminator)),
                    false => self.get_next(),
                }
            },
            ArgTypeBasic::ShortOptionSet(optset_string) => {
                // For **very** simple cases we can handle much more efficiently with an optimised
                // handler, e.g. for `-h` where we have just a single byte (ASCII) character.
                let optset_string_bytes = optset_string.as_bytes();
                if optset_string_bytes.len() == 1 {
                    Some(self.short_quick(optset_string_bytes[0]))
                }
                // Otherwise, we defer to a sub-iterator-like object specific to iterating over the
                // short option set (with dash prefix stripped). We will save the object in the main
                // iterator, and return its first `next()` result here.
                else {
                    let mut short_set_iter = ShortSetIter::new(optset_string);
                    let first = short_set_iter.get_next(self);
                    self.short_set_iter = Some(short_set_iter);
                    first
                }
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
                    return Some(Err(ProblemItem::UnknownOption(OptID::Long(OsStr::new("")), None)));
                }

                let lookup = match self.settings.allow_opt_abbreviations {
                    false => crate::matching::find_by_name(name,
                        self.options.long.iter(), |&o| { o.ident() }).into(),
                    true => crate::matching::find_by_abbrev_name(name,
                        self.options.long.iter(), |&o| { o.ident() }),
                };

                match lookup {
                    NameSearchResult::Match(matched) |
                    NameSearchResult::AbbreviatedMatch(matched) => {
                        // Use option’s full name, not the possibly abbreviated user provided one
                        let opt_name = matched.ident();

                        if matched.ty() != OptionType::Flag {
                            // Data included in same argument
                            // We accept it even if it’s an empty string
                            if let Some(data) = data_included {
                                self.last_data_loc = Some(DataLocation::SameArg);
                                Some(Ok(Item::Option(OptID::Long(opt_name), Some(data))))
                            }
                            // Data consumption is optional
                            else if matched.ty() == OptionType::Mixed {
                                self.last_data_loc = Some(DataLocation::SameArg);
                                Some(Ok(Item::Option(OptID::Long(opt_name), None)))
                            }
                            // Data included in next argument
                            else if let Some((_, next_arg)) = self.arg_iter.next() {
                                self.last_data_loc = Some(DataLocation::NextArg);
                                Some(Ok(Item::Option(OptID::Long(opt_name), Some(next_arg.as_ref()))))
                            }
                            // Data missing
                            else {
                                Some(Err(ProblemItem::MissingOptionData(OptID::Long(opt_name))))
                            }
                        }
                        // Ignore unexpected data if empty string
                        else if data_included.is_none() || data_included == Some(OsStr::new("")) {
                            Some(Ok(Item::Option(OptID::Long(opt_name), None)))
                        }
                        else {
                            let data = data_included.unwrap();
                            Some(Err(ProblemItem::LongWithUnexpectedData(opt_name, data)))
                        }
                    },
                    NameSearchResult::NoMatch => {
                        let suggestion = match self.serve_suggestions() {
                            #[cfg(feature = "suggestions")]
                            true => self.options.suggest(name),
                            _ => None,
                        };
                        // Again, we ignore any possibly included data in the argument
                        Some(Err(ProblemItem::UnknownOption(OptID::Long(name), suggestion)))
                    },
                    NameSearchResult::AmbiguousMatch => {
                        Some(Err(ProblemItem::AmbiguousLong(name)))
                    },
                }
            },
        }
    }

    #[inline(always)]
    fn serve_suggestions(&self) -> bool {
        #[cfg(not(feature = "suggestions"))]
        { false }
        #[cfg(feature = "suggestions")]
        { self.settings.serve_suggestions }
    }

    #[inline]
    fn register_positional(&mut self) {
        // Do not bother to assess things if we have already determined that we have encountered too
        // many positionals; that would mean that we have already been through this once, and we do
        // not want to increment the counter for those unexpected.
        //
        // Note that policy cannot be updated once an unexpected-positional item has been served,
        // and that once one has been served, all remaining must also therefore be unexpected.
        if !self.too_many_positionals {
            if self.settings.posixly_correct {
                self.rest_are_positionals = true;
            }
            self.too_many_positionals =
                self.positionals_policy.is_next_unexpected(self.positionals_count);
            if !self.too_many_positionals {
                self.positionals_count += 1;
            }
        }
    }

    /// Handles very simple short option args more efficiently than with `ShortSetIter`.
    /// Specifically this is designed to handle cases like `-h` where we just have a single ASCII
    /// byte.
    fn short_quick(&mut self, byte: u8) -> ItemResult<'set, 'arg> {
        let ch = char::from(byte);

        let lookup: SearchResult<ShortOption> = crate::matching::find_by_char(ch,
            self.options.short.iter(), |&o| { o.ident() }).into();

        match lookup {
            SearchResult::NoMatch => Err(ProblemItem::UnknownOption(OptID::Short(ch), None)),
            SearchResult::Match(matched) => {
                match matched.ty() {
                    OptionType::Flag => Ok(Item::Option(OptID::Short(ch), None)),
                    OptionType::Mixed => {
                        self.last_data_loc = Some(DataLocation::SameArg);
                        Ok(Item::Option(OptID::Short(ch), None))
                    },
                    OptionType::Data => {
                        if let Some((_, next_arg)) = self.arg_iter.next()
                        {
                            self.last_data_loc = Some(DataLocation::NextArg);
                            Ok(Item::Option(OptID::Short(ch), Some(next_arg.as_ref())))
                        }
                        // Data missing
                        else {
                            Err(ProblemItem::MissingOptionData(OptID::Short(ch)))
                        }
                    }
                }
            },
        }
    }
}

impl<'r, 'arg: 'r> ShortSetIter<'r, 'arg> {
    /// Create a new instance. Note, the provided string should **not** include the dash prefix.
    pub(crate) fn new(short_set_string: &'arg OsStr) -> Self {
        // Note, both the lossy converted string and the char iterator over it will live within the
        // struct, and will have the same lifetime. We are forced however to transmute the lifetime
        // of the borrow in creating the iterator to achieve this, but we know there will be no
        // problem doing so.
        let lossy = short_set_string.to_string_lossy();
        let lossy_ref = unsafe { mem::transmute::<&'_ Cow<str>, &'r Cow<str>>(&lossy) };
        let iter = lossy_ref.char_indices();
        Self {
            string: short_set_string,
            string_utf8: lossy,
            iter: iter,
            bytes_consumed: 0,
            consumed: false,
        }
    }

    /// Get next item, if any
    fn get_next<'set, A>(&mut self, parent: &mut ParseIter<'r, 'set, 'arg, A>)
        -> Option<ItemResult<'set, 'arg>>
        where A: AsRef<OsStr> + 'arg, 'set: 'r
    {
        if self.consumed {
            return None;
        }

        let (byte_pos, ch) = self.iter.next()?;

        let lookup: SearchResult<ShortOption>;

        match ch {
            // If we encounter the Unicode replacement character (U+FFFD) then we must beware that
            // this may have come from either a real such character, or from a byte sequence that
            // cannot be converted to valid UTF-8 in the original `OsStr`. To handle the latter, the
            // former is not allowed as a valid short option character.
            REPLACEMENT_CHARACTER => {
                lookup = SearchResult::NoMatch;

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
                lookup = crate::matching::find_by_char(ch,
                    parent.options.short.iter(), |&o| { o.ident() }).into();

                // Tracking?
                if self.bytes_consumed != 0 {
                    self.bytes_consumed += ch.len_utf8();
                }
            }
        }

        match lookup {
            SearchResult::NoMatch => Some(Err(ProblemItem::UnknownOption(OptID::Short(ch), None))),
            SearchResult::Match(matched) => {
                match matched.ty() {
                    OptionType::Flag => Some(Ok(Item::Option(OptID::Short(ch), None))),
                    _ => {
                        let bytes_consumed_updated = byte_pos + ch.len_utf8();

                        // If not last char, remaining chars are our data
                        if bytes_consumed_updated < self.string_utf8.len() {
                            self.consumed = true;
                            if self.bytes_consumed == 0 {
                                self.bytes_consumed = bytes_consumed_updated;
                            }
                            let data = OsStr::from_bytes(
                                &self.string.as_bytes()[self.bytes_consumed..]);
                            parent.last_data_loc = Some(DataLocation::SameArg);
                            Some(Ok(Item::Option(OptID::Short(ch), Some(data))))
                        }
                        // Data consumption is optional
                        else if matched.ty() == OptionType::Mixed {
                            parent.last_data_loc = Some(DataLocation::SameArg);
                            Some(Ok(Item::Option(OptID::Short(ch), None)))
                        }
                        // Data included in next argument
                        else if let Some((_, next_arg)) = parent.arg_iter.next() {
                            parent.last_data_loc = Some(DataLocation::NextArg);
                            Some(Ok(Item::Option(OptID::Short(ch), Some(next_arg.as_ref()))))
                        }
                        // Data missing
                        else {
                            Some(Err(ProblemItem::MissingOptionData(OptID::Short(ch))))
                        }
                    },
                }
            },
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

impl<'r, 'set, 'arg, A> ParseIterIndexed<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    /// Get the count of the number of positionals so far
    ///
    /// The counter is incremented on serving each positional, but **not** when serving an
    /// unexpected-positional problem item.
    #[inline(always)]
    pub fn get_positionals_count(&self) -> PositionalsQuantity {
        self.inner.get_positionals_count()
    }

    /// Update the policy for positionals
    ///
    /// This is provided to assist in situations where the policy needs to change dynamically, for
    /// instance in response to use of a particular option.
    ///
    /// This replaces the policy against which the number of positionals encountered is assessed
    /// upon encountering each positional item, so numbers need to be adjusted **without**
    /// accounting for the number returned so far. Thus, if your original policy was say `Fixed(3)`
    /// and you need to allow for an additional instance, and one has already been returned, you
    /// ignore the last fact and set to `Fixed(4)`.
    ///
    /// This will fail and return `Err` if used after an unexpected-positional item has already been
    /// issued; otherwise it will return `Ok`. You can freely change otherwise, even if lowering
    /// the number of positionals that should be accepted below the number already returned (though
    /// it would make no sense to do so).
    ///
    /// Panics on invalid policy.
    #[inline(always)]
    pub fn set_positionals_policy(&mut self, policy: PositionalsPolicy) -> Result<(), ()> {
        self.inner.set_positionals_policy(policy)
    }

    // Used by creation of `ItemSet` from iterator only
    #[inline(always)]
    pub(crate) fn get_parse_settings(&self) -> &Settings {
        self.inner.get_parse_settings()
    }
}

impl<'r, 'set, 'arg, A> CmdParseIter<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    /// Create a new instance
    #[inline]
    pub(crate) fn new(args: &'arg Args<A>, parser: &CmdParser<'r, 'set>) -> Self {
        Self {
            commands: parser.commands,
            inner: ParseIter::new_inner(args, &parser.inner, true),
        }
    }

    /// Wraps the iterator in one which also returns extra data
    ///
    /// Specifically this extra data amounts to argument index number and, where applicable, the
    /// location that option data was obtained from (same argument or next).
    #[inline(always)]
    pub fn indexed(self) -> CmdParseIterIndexed<'r, 'set, 'arg, A> {
        CmdParseIterIndexed { inner: self }
    }

    /// Get the input argument index of the previous item
    ///
    /// Note, if `next()` has not yet been called, zero will be returned (we do not bother with an
    /// `Option` wrapper). If the iterator has been fully consumed, it will continue to return the
    /// index of the last item.
    #[inline(always)]
    pub fn get_last_index(&self) -> usize {
        self.inner.last_index
    }

    /// Get the data-location of the previous item, if any
    ///
    /// This is used to query whether a data value provided with an option was supplied within the
    /// same argument or in the next argument, should you wish to know.
    ///
    /// `None` will be returned if this does not apply to the previous item, and similarly if
    /// `next()` has not yet been called. If the iterator has been fully consumed, it will continue
    /// to return the last value seen.
    #[inline(always)]
    pub fn get_last_dataloc(&self) -> Option<DataLocation> {
        self.inner.last_data_loc
    }

    /// Get the count of the number of positionals so far
    ///
    /// The counter is incremented on serving each positional, but **not** when serving an
    /// unexpected-positional problem item.
    #[inline(always)]
    pub fn get_positionals_count(&self) -> PositionalsQuantity {
        self.inner.get_positionals_count()
    }

    /// Update the policy for positionals
    ///
    /// This is provided to assist in situations where the policy needs to change dynamically, for
    /// instance in response to use of a particular option.
    ///
    /// This replaces the policy against which the number of positionals encountered is assessed
    /// upon encountering each positional item, so numbers need to be adjusted **without**
    /// accounting for the number returned so far. Thus, if your original policy was say `Fixed(3)`
    /// and you need to allow for an additional instance, and one has already been returned, you
    /// ignore the last fact and set to `Fixed(4)`.
    ///
    /// This will fail and return `Err` if used after an unexpected-positional item has already been
    /// issued; otherwise it will return `Ok`. You can freely change otherwise, even if lowering
    /// the number of positionals that should be accepted below the number already returned (though
    /// it would make no sense to do so).
    ///
    /// Panics on invalid policy.
    #[inline(always)]
    pub fn set_positionals_policy(&mut self, policy: PositionalsPolicy) -> Result<(), ()> {
        self.inner.set_positionals_policy(policy)
    }

    /// Get the *option set* currently in use for parsing
    ///
    /// This is useful for suggestion matching of unknown options
    #[inline(always)]
    pub fn get_option_set(&self) -> &'r OptionSet<'r, 'set> {
        self.inner.options
    }

    /// Change the *option set* used for parsing by subsequent iterations
    ///
    /// This is typically only applicable where you are using the iterative parsing style with a
    /// command based program, where instead of describing the entire command structure to the
    /// parser up front, you want to dynamically switch out the *option set* used for subsequent
    /// iterations (arguments) manually, after encountering a command.
    ///
    /// Note, it is undefined behaviour to set a non-valid option set.
    pub fn set_option_set(&mut self, opt_set: &'r OptionSet<'r, 'set>) {
        self.inner.options = opt_set;
    }

    /// Get the *command set* currently in use for parsing
    ///
    /// This is useful for suggestion matching of an unknown command
    #[inline(always)]
    pub fn get_command_set(&self) -> &'r CommandSet<'r, 'set> {
        self.commands
    }

    /// Change the *command set* used for parsing by subsequent iterations
    ///
    /// This is typically only applicable where you are using the iterative parsing style with a
    /// command based program, where instead of describing the entire command structure to the
    /// parser up front, you want to dynamically switch out the *command set* used for subsequent
    /// iterations (arguments) manually, after encountering a command.
    ///
    /// Note, it is undefined behaviour to set a non-valid command set.
    pub fn set_command_set(&mut self, cmd_set: &'r CommandSet<'r, 'set>) {
        self.commands = cmd_set;
    }

    /// Get a mutable reference to the parser settings
    ///
    /// The use case for this method is similar to that of the methods for changing the *option
    /// set* and *command set* to be used, though more niche. It is thought unlikely that any
    /// program should have any need to change settings in the middle of parsing, but you can if you
    /// absolutely want to (there is no reason to prevent you from doing so).
    #[inline]
    pub fn get_parse_settings(&mut self) -> &mut Settings {
        &mut self.inner.settings
    }
}

impl<'r, 'set, 'arg, A> CmdParseIterIndexed<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    /// Get the count of the number of positionals so far
    ///
    /// The counter is incremented on serving each positional, but **not** when serving an
    /// unexpected-positional problem item.
    #[inline(always)]
    pub fn get_positionals_count(&self) -> PositionalsQuantity {
        self.inner.get_positionals_count()
    }

    /// Update the policy for positionals
    ///
    /// This is provided to assist in situations where the policy needs to change dynamically, for
    /// instance in response to use of a particular option.
    ///
    /// This replaces the policy against which the number of positionals encountered is assessed
    /// upon encountering each positional item, so numbers need to be adjusted **without**
    /// accounting for the number returned so far. Thus, if your original policy was say `Fixed(3)`
    /// and you need to allow for an additional instance, and one has already been returned, you
    /// ignore the last fact and set to `Fixed(4)`.
    ///
    /// This will fail and return `Err` if used after an unexpected-positional item has already been
    /// issued; otherwise it will return `Ok`. You can freely change otherwise, even if lowering
    /// the number of positionals that should be accepted below the number already returned (though
    /// it would make no sense to do so).
    ///
    /// Panics on invalid policy.
    #[inline(always)]
    pub fn set_positionals_policy(&mut self, policy: PositionalsPolicy) -> Result<(), ()> {
        self.inner.set_positionals_policy(policy)
    }

    /// Get the *option set* currently in use for parsing
    ///
    /// This is useful for suggestion matching of unknown options
    #[inline(always)]
    pub fn get_option_set(&self) -> &'r OptionSet<'r, 'set> {
        self.inner.get_option_set()
    }

    /// Change the *option set* used for parsing by subsequent iterations
    ///
    /// This is typically only applicable where you are using the iterative parsing style with a
    /// command based program, where instead of describing the entire command structure to the
    /// parser up front, you want to dynamically switch out the *option set* used for subsequent
    /// iterations (arguments) manually, after encountering a command.
    ///
    /// Note, it is undefined behaviour to set a non-valid option set.
    #[inline(always)]
    pub fn set_option_set(&mut self, opt_set: &'r OptionSet<'r, 'set>) {
        self.inner.set_option_set(opt_set);
    }

    /// Get the *command set* currently in use for parsing
    ///
    /// This is useful for suggestion matching of an unknown command
    #[inline(always)]
    pub fn get_command_set(&self) -> &'r CommandSet<'r, 'set> {
        self.inner.get_command_set()
    }

    /// Change the *command set* used for parsing by subsequent iterations
    ///
    /// This is typically only applicable where you are using the iterative parsing style with a
    /// command based program, where instead of describing the entire command structure to the
    /// parser up front, you want to dynamically switch out the *command set* used for subsequent
    /// iterations (arguments) manually, after encountering a command.
    ///
    /// Note, it is undefined behaviour to set a non-valid command set.
    #[inline(always)]
    pub fn set_command_set(&mut self, cmd_set: &'r CommandSet<'r, 'set>) {
        self.inner.set_command_set(cmd_set);
    }

    /// Get a mutable reference to the parser settings
    ///
    /// The use case for this method is similar to that of the methods for changing the *option
    /// set* and *command set* to be used, though more niche. It is thought unlikely that any
    /// program should have any need to change settings in the middle of parsing, but you can if you
    /// absolutely want to (there is no reason to prevent you from doing so).
    #[inline(always)]
    pub fn get_parse_settings(&mut self) -> &mut Settings {
        self.inner.get_parse_settings()
    }
}

impl<'r, 'set, 'arg, A> From<ParseIter<'r, 'set, 'arg, A>>
    for ParseIterIndexed<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    fn from(iter: ParseIter<'r, 'set, 'arg, A>) -> Self {
        iter.indexed()
    }
}

impl<'r, 'set, 'arg, A> From<CmdParseIter<'r, 'set, 'arg, A>>
    for CmdParseIterIndexed<'r, 'set, 'arg, A>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    fn from(iter: CmdParseIter<'r, 'set, 'arg, A>) -> Self {
        iter.indexed()
    }
}
