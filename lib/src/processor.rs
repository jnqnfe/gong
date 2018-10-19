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
    use ItemClass as Class;

    /* NOTE: We deliberately do not perform validation of the provided `options` data within this
     * function; the burden to do so is left to the user, thus allowing this function to be called
     * multiple times if desired, without introducing inefficiency! */

    // Whether an argument of exactly `--` has been encountered, symbolising early termination of
    // option processing, meaning all remaining arguments are to be treated as non-options.
    let mut early_termination = false;

    let mode = options.mode;
    let mut results = Analysis::new(args.len());

    let mut arg_iter = args.iter().enumerate();
    while let Some((index, arg)) = arg_iter.next() {
        let arg = arg.as_ref();

        if early_termination {
            results.add(Class::Ok(Item::NonOption(index, arg)));
            continue;
        }

        // Get length of initial portion
        let start_len = arg.chars().take(3).count();

        // Early terminator
        if start_len == 2 && arg == "--" {
            // Yes, it may be valuable info to the caller to know that one was encountered and
            // where, so let's not leave it out of the results.
            results.add(Class::Ok(Item::EarlyTerminator(index)));
            early_termination = true;
            continue;
        }

        let has_double_dash_prefix = || {
            start_len > 2 && arg.starts_with("--")
        };
        let has_single_dash_prefix = || {
            start_len > 1 && arg.starts_with("-")
        };

        // Long option
        if (mode == OptionsMode::Standard && has_double_dash_prefix()) ||
            (mode == OptionsMode::Alternate && has_single_dash_prefix())
        {
            /* We need to deal with the fact that arg data may be supplied in the same argument,
             * separated by an `=`, and also that the user is allowed to supply an abbreviated form
             * of an available option, so long as it is unique, which requires checking for
             * ambiguity. (See documentation). */

            // Extract name, splitting from optional data arg
            let without_prefix = match mode {
                OptionsMode::Standard => arg.split_at(2).1, //"--" length
                OptionsMode::Alternate => arg.split_at(1).1, //"-" length
            };
            let mut parts_iter = without_prefix.splitn(2, '=');
            let name = parts_iter.next().unwrap(); /* Must exist */
            let data_included = parts_iter.next();

            let name_char_count = name.chars().count();

            // This occurs with `--=` or `--=foo` (`-=` or `-=foo` in alt mode)
            if name_char_count == 0 {
                results.add(Class::Warn(ItemW::LongWithNoName(index)));
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
                    // An exact match overrules a previously found partial match and ambiguity found
                    // with multiple partial matches.
                    matched = Some(&*candidate);
                    ambiguity = false;
                    break;
                }
                // Abbreviated
                else if options.allow_abbreviations &&
                    cand_char_count > name_char_count &&
                    candidate.name.starts_with(name)
                {
                    if matched.is_none() {
                        matched = Some(&*candidate);
                    }
                    else {
                        ambiguity = true;
                        break;
                    }
                }
            }

            if ambiguity {
                results.add(Class::Err(ItemE::AmbiguousLong(index, name)));
                results.error = true;
            }
            else if let Some(matched) = matched {
                // Use option's full name, not the possibly abbreviated user provided one
                let opt_name = &matched.name;

                if matched.expects_data {
                    // Data included in same argument
                    if let Some(data) = data_included {
                        results.add(Class::Ok(Item::LongWithData {
                            i: index, n: opt_name, d: data, l: DataLocation::SameArg }));
                    }
                    // Data included in next argument
                    else if let Some((_, next_arg)) = arg_iter.next() {
                        results.add(Class::Ok(Item::LongWithData {
                            i: index, n: opt_name, d: next_arg.as_ref(),
                            l: DataLocation::NextArg }));
                    }
                    // Data missing
                    else {
                        results.add(Class::Err(ItemE::LongMissingData(index, opt_name)));
                        results.error = true;
                    }
                }
                else {
                    let mut added_entry = false;
                    if let Some(data) = data_included {
                        // Ignore unexpected data if empty string
                        if data.len() > 0 {
                            results.add(Class::Warn(ItemW::LongWithUnexpectedData {
                                i: index, n: opt_name, d: data }));
                            results.warn = true;
                            added_entry = true;
                        }
                    }
                    if !added_entry {
                        results.add(Class::Ok(Item::Long(index, opt_name)));
                    }
                }
            }
            else {
                // Again, we ignore any possibly included data in the argument
                results.add(Class::Warn(ItemW::UnknownLong(index, name)));
                results.warn = true;
            }
            continue;
        }

        // Short option(s)
        // Note, lone `-` argument is considered a non-option!
        if mode == OptionsMode::Standard && has_single_dash_prefix() {
            let last_char_index = arg.chars().skip(1).count() - 1;
            for (i, (byte_pos, ch)) in arg.char_indices().skip(1).enumerate() {
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
                    results.add(Class::Warn(ItemW::UnknownShort(index, ch)));
                    results.warn = true;
                }
                else if !expects_data {
                    results.add(Class::Ok(Item::Short(index, ch)));
                }
                else {
                    // If not last char, remaining chars are our data
                    if i < last_char_index {
                        let next_char_byte_pos = byte_pos + ch.len_utf8();
                        let data = arg.split_at(next_char_byte_pos).1;
                        results.add(Class::Ok(Item::ShortWithData {
                            i: index, c: ch, d: data, l: DataLocation::SameArg }));
                        break;
                    }
                    // Data included in next argument
                    else if let Some((_, next_arg)) = arg_iter.next() {
                        results.add(Class::Ok(Item::ShortWithData {
                            i: index, c: ch, d: next_arg.as_ref(), l: DataLocation::NextArg }));
                    }
                    // Data missing
                    else {
                        results.add(Class::Err(ItemE::ShortMissingData(index, ch)));
                        results.error = true;
                    }
                }
            }
            continue;
        }

        results.add(Class::Ok(Item::NonOption(index, arg)));
    }
    results
}
