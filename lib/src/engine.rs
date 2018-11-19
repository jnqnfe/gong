// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

use std::convert::AsRef;
use super::options::*;
use super::analysis::*;

const SINGLE_DASH_PREFIX: &str = "-";
const DOUBLE_DASH_PREFIX: &str = "--";
const EARLY_TERMINATOR: &str = "--";

/// Basic argument type
///
/// Option variants should: include argument without prefix; include "in-same-arg" data values.
enum ArgTypeBasic<'a> {
    NonOption,
    EarlyTerminator,
    LongOption(&'a str),
    ShortOptionSet(&'a str),
}

/// Analyses provided program arguments, using provided information about valid available options.
///
/// Returns a result set describing the result of the analysis. This may include `&str` references
/// to strings provided in the `args` and `options` parameter data. Take note of this with respect
/// to object lifetimes.
///
/// Expects available `options` data to have already been validated. (See
/// [`OptionSet::is_valid`](options/struct.OptionSet.html#method.is_valid)).
pub(crate) fn process<'o, 'r, 'a, A>(args: &'a [A], options: &'o OptionSet<'r, 'a>) -> Analysis<'a>
    where A: 'a + AsRef<str>,
          'r: 'o, 'a: 'r
{
    /* NOTE: We deliberately do not perform validation of the provided `options` data within this
     * function; the burden to do so is left to the user. The choice to not do this is for reasons
     * of efficiency - to not waste energy on known good sets, and to avoid waste of energy if this
     * function is called multiple times with the same set. */

    let get_basic_arg_type_fn = match options.mode {
        OptionsMode::Standard => get_basic_arg_type_standard,
        OptionsMode::Alternate => get_basic_arg_type_alternate,
    };

    let mut results = Analysis::new(args.len());
    let mut early_terminator_encountered = false;

    let mut arg_iter = args.iter().enumerate();

    while let Some((arg_index, arg_non_ref)) = arg_iter.next() {
        let arg_ref = arg_non_ref.as_ref();

        let arg_type = match early_terminator_encountered {
            true => ArgTypeBasic::NonOption,
            false => get_basic_arg_type_fn(arg_ref),
        };

        match arg_type {
            ArgTypeBasic::NonOption => {
                results.add(ItemClass::Ok(Item::NonOption(arg_index, arg_ref)));
            },
            ArgTypeBasic::EarlyTerminator => {
                early_terminator_encountered = true;
                // Yes, it may be valuable info to the caller to know that one was encountered and
                // where, so let's not leave it out of the results.
                results.add(ItemClass::Ok(Item::EarlyTerminator(arg_index)));
            },
            ArgTypeBasic::LongOption(opt_string) => {
                /* We need to deal with the fact that arg data may be supplied in the same argument,
                 * separated by an `=`, and also that the user is allowed to supply an abbreviated
                 * form of an available option, so long as it is unique, which requires checking for
                 * ambiguity. (See documentation). */

                // Extract name, splitting from optional "in-same-arg" data value
                let (name, data_included) = match opt_string.find('=') {
                    None => (opt_string, None),
                    Some(i) => {
                        let split = opt_string.split_at(i);
                        // We know that the '=' is encoded as just one byte and that it is
                        // definately there, so we can safely skip it unchecked.
                        let data_included = unsafe { split.1.get_unchecked(1..) };
                        (split.0, Some(data_included))
                    },
                };

                // This occurs with `--=` or `--=foo` (`-=` or `-=foo` in alt mode)
                if name.is_empty() {
                    results.add(ItemClass::Warn(ItemW::LongWithNoName(arg_index)));
                    results.warn = true;
                    continue;
                }

                let mut matched: Option<&LongOption> = None;
                let mut ambiguity = false;
                for candidate in options.long {
                    // Exact
                    if candidate.name == name {
                        // An exact match overrules a previously found partial match and ambiguity
                        // found with multiple partial matches.
                        matched = Some(candidate);
                        ambiguity = false;
                        break;
                    }
                    // Abbreviated
                    else if options.allow_abbreviations && !ambiguity
                        && candidate.name.starts_with(name)
                    {
                        match matched {
                            Some(_) => { ambiguity = true; },
                            None => { matched = Some(candidate); },
                        }
                    }
                }

                if ambiguity {
                    results.add(ItemClass::Err(ItemE::AmbiguousLong(arg_index, name)));
                    results.error = true;
                }
                else if let Some(matched) = matched {
                    // Use option's full name, not the possibly abbreviated user provided one
                    let opt_name = &matched.name;

                    if matched.expects_data {
                        // Data included in same argument
                        // We accept it even if it's an empty string
                        if let Some(data) = data_included {
                            results.add(ItemClass::Ok(Item::LongWithData {
                                i: arg_index, n: opt_name, d: data, l: DataLocation::SameArg }));
                        }
                        // Data included in next argument
                        else if let Some((_, next_arg)) = arg_iter.next() {
                            results.add(ItemClass::Ok(Item::LongWithData {
                                i: arg_index, n: opt_name, d: next_arg.as_ref(),
                                l: DataLocation::NextArg }));
                        }
                        // Data missing
                        else {
                            results.add(ItemClass::Err(ItemE::LongMissingData(arg_index, opt_name)));
                            results.error = true;
                        }
                    }
                    else {
                        match data_included {
                            None |
                            // Ignore unexpected data if empty string
                            Some("") => {
                                results.add(ItemClass::Ok(Item::Long(arg_index, opt_name)));
                            },
                            Some(data) => {
                                results.add(ItemClass::Warn(ItemW::LongWithUnexpectedData {
                                    i: arg_index, n: opt_name, d: data }));
                                results.warn = true;
                            },
                        }
                    }
                }
                else {
                    // Again, we ignore any possibly included data in the argument
                    results.add(ItemClass::Warn(ItemW::UnknownLong(arg_index, name)));
                    results.warn = true;
                }
            },
            ArgTypeBasic::ShortOptionSet(optset_string) => {
                let last_char_index = optset_string.chars().count() - 1;
                for (i, (byte_pos, ch)) in optset_string.char_indices().enumerate() {
                    let mut match_found = false;
                    let mut expects_data = false;
                    for candidate in options.short {
                        if candidate.ch == ch {
                            match_found = true;
                            expects_data = candidate.expects_data;
                            break;
                        }
                    }

                    if !match_found {
                        results.add(ItemClass::Warn(ItemW::UnknownShort(arg_index, ch)));
                        results.warn = true;
                    }
                    else if !expects_data {
                        results.add(ItemClass::Ok(Item::Short(arg_index, ch)));
                    }
                    else {
                        // If not last char, remaining chars are our data
                        if i < last_char_index {
                            let next_char_byte_pos = byte_pos + ch.len_utf8();
                            let data = optset_string.split_at(next_char_byte_pos).1;
                            results.add(ItemClass::Ok(Item::ShortWithData {
                                i: arg_index, c: ch, d: data, l: DataLocation::SameArg }));
                            break;
                        }
                        // Data included in next argument
                        else if let Some((_, next_arg)) = arg_iter.next() {
                            results.add(ItemClass::Ok(Item::ShortWithData {
                                i: arg_index, c: ch, d: next_arg.as_ref(),
                                l: DataLocation::NextArg }));
                        }
                        // Data missing
                        else {
                            results.add(ItemClass::Err(ItemE::ShortMissingData(arg_index, ch)));
                            results.error = true;
                        }
                    }
                }
            },
        }
    }
    results
}

// Check if `arg` has the given prefix.
//
// This is similar to a `starts_with` check, but the length must be longer than the prefix, equal
// length is no good.
#[inline(always)]
fn has_prefix(arg: &str, prefix: &str) -> bool {
    // Note, it is safe to index into `arg` in here; we don't care about char boundaries for the
    // simple byte-slice comparison. Doing this is optimally efficient, avoiding `start_with`'s
    // `>=` length comparison check, as well as utf-8 char boundary checks, etc.
    let prefix_len = prefix.len();
    arg.len() > prefix_len && &prefix.as_bytes()[..] == &arg.as_bytes()[..prefix_len]
}

/// Assess argument type, returning options without their prefix, for 'standard' mode
fn get_basic_arg_type_standard<'a>(arg: &'a str) -> ArgTypeBasic<'a> {
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

/// Assess argument type, returning options without their prefix, for 'alternate' mode
fn get_basic_arg_type_alternate<'a>(arg: &'a str) -> ArgTypeBasic<'a> {
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
