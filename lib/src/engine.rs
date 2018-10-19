// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

use std::convert::AsRef;
use super::options::*;
use super::analysis::*;

const SINGLE_DASH_PREFIX: &str = "-";
const DOUBLE_DASH_PREFIX: &str = "--";
const EARLY_TERMINATOR: &str = "--";
const SINGLE_DASH_PREFIX_LEN: usize = 1;
const DOUBLE_DASH_PREFIX_LEN: usize = 2;
const EARLY_TERMINATOR_LEN: usize = 2;
/// Minimum num chars of string start needed for assessing option prefixing (widest prefix + 1)
const PREFIX_ASSESS_CHARS: usize = 3;

#[cfg(test)]
#[test]
fn constants() {
    assert_eq!(SINGLE_DASH_PREFIX_LEN, SINGLE_DASH_PREFIX.len());
    assert_eq!(DOUBLE_DASH_PREFIX_LEN, DOUBLE_DASH_PREFIX.len());
    assert_eq!(EARLY_TERMINATOR_LEN, EARLY_TERMINATOR.len());
    assert!(PREFIX_ASSESS_CHARS > SINGLE_DASH_PREFIX_LEN);
    assert!(PREFIX_ASSESS_CHARS > DOUBLE_DASH_PREFIX_LEN);
    assert!(PREFIX_ASSESS_CHARS > EARLY_TERMINATOR_LEN);
}

/// Basic argument type
///
/// Option variants should: include argument without prefix; include "in-same-arg" data values.
enum ArgTypeBasic<'a> {
    NonOption,
    EarlyTerminator,
    LongOption(&'a str),
    ShortOptionSet(&'a str),
}

//TODO: This implementation was originally designed with some degree of caution against the
// possibility of argument strings not necessarily being valid utf-8, although this was not
// thoroughly evaluated, nor such situations tested for, with such situations being unlikely. This
// could do with being reassessed.

/// Analyses provided program arguments, using provided information about valid available options.
///
/// Returns a result set describing the result of the analysis. This may include `&str` references
/// to strings provided in the `args` and `options` parameter data. Take note of this with respect
/// to object lifetimes.
///
/// Expects available `options` data to have already been validated. (See
/// [`Options::is_valid`](options/struct.Options.html#method.is_valid)).
pub fn process<'a, T>(args: &'a [T], options: &Options<'a>) -> Analysis<'a>
    where T: AsRef<str>
{
    /* NOTE: We deliberately do not perform validation of the provided `options` data within this
     * function; the burden to do so is left to the user. The choice to not do this is for reasons
     * of efficiency - to not waste energy on known good sets, and to avoid waste of energy if this
     * function is called multiple times with the same set. */

    let mut results = Analysis::new(args.len());
    let mut early_terminator_encountered = false;

    let mut arg_iter = args.iter().enumerate();

    while let Some((arg_index, arg_non_ref)) = arg_iter.next() {
        let arg_ref = arg_non_ref.as_ref();

        let arg_type = match early_terminator_encountered {
            true => ArgTypeBasic::NonOption,
            false => get_basic_arg_type(arg_ref, options.mode),
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
                let mut parts_iter = opt_string.splitn(2, '=');
                let name = parts_iter.next().unwrap(); /* Must exist */
                let data_included = parts_iter.next();

                let name_char_count = name.chars().count();

                // This occurs with `--=` or `--=foo` (`-=` or `-=foo` in alt mode)
                if name_char_count == 0 {
                    results.add(ItemClass::Warn(ItemW::LongWithNoName(arg_index)));
                    results.warn = true;
                    continue;
                }

                let mut matched: Option<&LongOption> = None;
                let mut ambiguity = false;
                for candidate in &options.long {
                    let cand_char_count = candidate.name.chars().count();
                    // Exact
                    if cand_char_count == name_char_count &&
                        candidate.name == name
                    {
                        // An exact match overrules a previously found partial match and ambiguity
                        // found with multiple partial matches.
                        matched = Some(candidate);
                        ambiguity = false;
                        break;
                    }
                    // Abbreviated
                    else if options.allow_abbreviations &&
                        !ambiguity &&
                        cand_char_count > name_char_count &&
                        candidate.name.starts_with(name)
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
                        let mut added_entry = false;
                        if let Some(data) = data_included {
                            // Ignore unexpected data if empty string
                            if data.len() > 0 {
                                results.add(ItemClass::Warn(ItemW::LongWithUnexpectedData {
                                    i: arg_index, n: opt_name, d: data }));
                                results.warn = true;
                                added_entry = true;
                            }
                        }
                        if !added_entry {
                            results.add(ItemClass::Ok(Item::Long(arg_index, opt_name)));
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
                    for candidate in &options.short {
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

/// Assess argument type, returning options without their prefix
#[inline(always)]
fn get_basic_arg_type<'a>(arg: &'a str, mode: OptionsMode) -> ArgTypeBasic<'a> {
    // Get length of initial portion
    let start_len = arg.chars().take(PREFIX_ASSESS_CHARS).count();

    let check_double = match mode {
        OptionsMode::Standard => true,
        OptionsMode::Alternate => false,
    };

    macro_rules! has_prefix_double_dash {
        ( $arg:expr, $start_len:expr ) => {
            has_prefix!($arg, DOUBLE_DASH_PREFIX, DOUBLE_DASH_PREFIX_LEN, $start_len)
        }
    }
    macro_rules! has_prefix_single_dash {
        ( $arg:expr, $start_len:expr ) => {
            has_prefix!($arg, SINGLE_DASH_PREFIX, SINGLE_DASH_PREFIX_LEN, $start_len)
        }
    }
    macro_rules! has_prefix {
        ( $arg:expr, $prefix:expr, $prefix_len:expr, $start_len:expr ) => {
            // The length must be longer than the prefix
            ($start_len > $prefix_len && $arg.starts_with($prefix))
        }
    }

    match start_len {
        EARLY_TERMINATOR_LEN if arg == EARLY_TERMINATOR => ArgTypeBasic::EarlyTerminator,
        i if check_double && has_prefix_double_dash!(arg, i) => {
            ArgTypeBasic::LongOption(&arg[DOUBLE_DASH_PREFIX_LEN..])
        },
        i if has_prefix_single_dash!(arg, i) => {
            let prefix_stripped = &arg[SINGLE_DASH_PREFIX_LEN..];
            match mode {
                OptionsMode::Standard => ArgTypeBasic::ShortOptionSet(prefix_stripped),
                OptionsMode::Alternate => ArgTypeBasic::LongOption(prefix_stripped),
            }
        },
        _ => ArgTypeBasic::NonOption,
    }
}
