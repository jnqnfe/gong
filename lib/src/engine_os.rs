// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! `AsRef<OsStr>` based variant of the engine
//!
//! This provides a variant of the standard parser which takes `AsRef<OsStr>` based arguments
//! instead of `ArRef<str>` based ones. It is to be used in situations where users want to handle
//! arguments taken from the environment via `std::end::os_args()` instead of `std::env::args()`.

/* Overview of solution:
 *
 *  1. Valid short and long options must be forbidden from using the unicode replacement char
 *     (`U+FFFD`) else incorrect matches could occur, which needs enforcing throughout the library.
 *  2. Do a lossy conversion of arguments to `Cow<str>`.
 *  3. Parse this with the normal `str` based parser (avoids unnecessarily duplicating logic).
 *  4. Convert the analysis item returned by the `str` based parser:
 *      a. Copy warn/error booleans
 *      b. Loop through analysis items, converting and adding each.
 *          - Some cases will be easy to handle, for instance with non-options and in-next-arg data
 *            values we just reference the original argument.
 *          - For matched long option names, we just use what was given, since valid option names
 *            are only permitted to be valid UTF-8 strings (being `&str`), and because in the case
 *            of `--foo=` we need to ignore the `=`, thus want the given name, not the original, and
 *            also because we want the full name in the returned item, not any given abbreviation.
 *          - Some will not be quite so simple:
 *             - When it comes to unknown or ambiguous long option names, we need to get the name
 *               from the original string; here we need to ignore any potential in-same-arg data
 *               value. This requires hunting for the equals (`=`) separator. (Note, this separator
 *               is not allowed in valid option names, so the first one should always be taken to
 *               be the separator).
 *             - For recognised long options, where in-same argument data has been supplied (whether
 *               the option expects it or not), the data value needs capturing from the original
 *               string. This is not so difficult, just requiring reconstructing the prefix portion
 *               with the long option name, which we know to be valid UTF-8 and thus gives no issues
 *               regarding byte length.
 *             - Short option sets are a little tricky, but manageable. We must understand the rules
 *               of how many unicode replacement characters result from any given invalid byte
 *               sequence. As we proceed through the items returned by the `str` based parser, we
 *               need to keep track of what we have processed so far, in order that we can correctly
 *               extract any in-same-arg data value. The tricky part is that where the `str` based
 *               parser signals that the current character is the unicode replacement character, we
 *               need to figure out how many bytes this represents in the original argument.
 *
 * So long as the unicode replacement char is forbidden in valid long option names and as a
 * short option char, this should work perfectly.
 */

use std;
use std::borrow::Cow;
use std::char::REPLACEMENT_CHARACTER;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::mem;
use std::ops::Range;
#[cfg(any(unix, target_os = "redox"))]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use self::windows::OsStrExt;
use super::parser::*;
use super::analysis::*;
use super::engine::{ParseIter, SINGLE_DASH_PREFIX, DOUBLE_DASH_PREFIX};

/// This brings clarity, ensuring correct starting value used
macro_rules! set_shortset_consumed_init { () => { SINGLE_DASH_PREFIX.len() } }

/// An argument list parsing iterator
///
/// Created by the [`parse_iter_os`] method of [`Parser`].
///
/// [`parse_iter_os`]: ../parser/struct.Parser.html#method.parse_iter_os
/// [`Parser`]: ../parser/struct.Parser.html
#[derive(Clone)]
pub struct ParseIterOs<'r, 's: 'r, A: 's + AsRef<OsStr>> {
    /// The original argument list
    args: &'s [A],
    /// The original argument list, lossy converted
    args_as_str: Vec<Cow<'s, str>>,
    /// Inner `AsRef<str>` based parsing iterator
    parse_iter: ParseIter<'r, 's, Cow<'s, str>>,
    /// Short option set tracking data
    short_set_tracking_data: ShortOptSetData,
    /// Cached prefix used by long options, depending upon parsing mode
    longopt_prefix: &'s OsStr,
}

/// Used for tracking short option set consumption, which is necessary for correctly extracting an
/// in-same-arg data value, if there is one, and in the face of possible invalid bytes in the option
/// set in the option set argument.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct ShortOptSetData {
    /// Index of argument that the previous short option came from, or zero
    last_arg: usize,
    /// How many bytes of the current option set have been consumed so far
    consumed: usize,
}

impl Default for ShortOptSetData {
    fn default() -> Self {
        Self { last_arg: 0, consumed: set_shortset_consumed_init!(), }
    }
}

impl<'r, 's, A> Iterator for ParseIterOs<'r, 's, A>
    where A: 's + AsRef<OsStr>, 's: 'r
{
    type Item = ItemClass<'s, OsStr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_iter.next().and_then(|i| Some(ParseIterOs::convert_item(self, i)))
    }
}

impl<'r, 's, A> ParseIterOs<'r, 's, A>
    where A: 's + AsRef<OsStr>, 's: 'r
{
    /// Create a new instance
    pub(crate) fn new(args: &'s [A], parser: &Parser<'r, 's>) -> Self {
        // Temporary lossy conversion
        let args_as_str: Vec<Cow<'s, str>> =
            args.iter().map(|s| s.as_ref().to_string_lossy()).collect();
        // HACK: We must adjust the lifetime for use with `parse`
        let args_as_str_slice = unsafe {
            mem::transmute::<&'_ [Cow<'s, str>], &'s [Cow<'s, str>]>(&args_as_str[..])
        };

        Self {
            args: args,
            args_as_str: args_as_str,
            parse_iter: ParseIter::new(args_as_str_slice, parser),
            short_set_tracking_data: ShortOptSetData::default(),
            longopt_prefix: Self::give_longopt_prefix(parser.settings.mode),
        }
    }

    #[inline]
    fn give_longopt_prefix(mode: OptionsMode) -> &'s OsStr {
        match mode {
            OptionsMode::Standard => OsStr::new(DOUBLE_DASH_PREFIX),
            OptionsMode::Alternate => OsStr::new(SINGLE_DASH_PREFIX),
        }
    }

    /// Convert an analysis item from `str` form to `OsStr` form from the original arguments. This
    /// requires access to an object for tracking short option set consumption, for correct
    /// extraction of an in-same-argument data value.
    fn convert_item(&mut self, item: ItemClass<'s, str>) -> ItemClass<'s, OsStr> {
        let short_set_data = &mut self.short_set_tracking_data;
        let opt_mode = self.parse_iter.parser_data.settings.mode;

        let args = self.args; //Get around borrow checker
        let track_short_opt = |data: &mut ShortOptSetData, i, c| {
            if data.last_arg != i {
                data.reset(i);
            }
            track_short_opt_set(args[i].as_ref(), &mut data.consumed, c);
        };

        // REMINDER: We **MUST** throw away all strings in the original items which have been
        // sourced from the `args` param we gave to the `str` based parser, since those were from
        // our local lossy `str` conversion. These must be replaced with respective slices/strings
        // from the originals.
        match item {
            /* These can all be copied directly */

            ItemClass::Ok(Item::EarlyTerminator(i)) =>       ItemClass::Ok(Item::EarlyTerminator(i)),
            ItemClass::Ok(Item::Long(i, n)) =>               ItemClass::Ok(Item::Long(i, n)),
            ItemClass::Ok(Item::Command(i, n)) =>            ItemClass::Ok(Item::Command(i, n)),
            ItemClass::Err(ItemE::LongMissingData(i, n)) =>  ItemClass::Err(ItemE::LongMissingData(i, n)),
            ItemClass::Err(ItemE::ShortMissingData(i, c)) => ItemClass::Err(ItemE::ShortMissingData(i, c)),
            ItemClass::Warn(ItemW::LongWithNoName(i)) =>     ItemClass::Warn(ItemW::LongWithNoName(i)),

            /* These can be copied directly, but we need to track the short option set */

            ItemClass::Ok(Item::Short(i, c)) => {
                track_short_opt(short_set_data, i, c);
                ItemClass::Ok(Item::Short(i, c))
            },
            ItemClass::Warn(ItemW::UnknownShort(i, c)) => {
                track_short_opt(short_set_data, i, c);
                ItemClass::Warn(ItemW::UnknownShort(i, c))
            },

            /* These need more work, capturing part or all of the original `OsStr` */

            ItemClass::Ok(Item::NonOption(i, _)) => {
                ItemClass::Ok(Item::NonOption(i, self.args[i].as_ref()))
            },
            ItemClass::Ok(Item::LongWithData{ i, n, l, .. }) => {
                let data = match l {
                    DataLocation::SameArg => {
                        let index = match opt_mode {
                            OptionsMode::Standard => DOUBLE_DASH_PREFIX.len(),
                            OptionsMode::Alternate => SINGLE_DASH_PREFIX.len(),
                        } + n.len() + "=".len();
                        get_osstr_suffix(self.args[i].as_ref(), index)
                    },
                    DataLocation::NextArg => self.args[i+1].as_ref(),
                };
                ItemClass::Ok(Item::LongWithData{ i, n, d: data, l })
            },
            ItemClass::Ok(Item::ShortWithData{ i, c, l, .. }) => {
                let data = match l {
                    DataLocation::SameArg => {
                        // NB: This works because both Unix and Windows OsStr implementations use
                        // less-string UTF-8 sequence based storage.
                        track_short_opt(short_set_data, i, c);
                        get_osstr_suffix(self.args[i].as_ref(), short_set_data.consumed)
                    },
                    DataLocation::NextArg => self.args[i+1].as_ref(),
                };
                ItemClass::Ok(Item::ShortWithData{ i, c, d: data, l })
            },
            ItemClass::Err(ItemE::AmbiguousLong(i, _)) => {
                let opt_name = get_osstr_longopt_name(self.args[i].as_ref(), self.longopt_prefix);
                ItemClass::Err(ItemE::AmbiguousLong(i, opt_name))
            },
            ItemClass::Warn(ItemW::UnknownLong(i, _)) => {
                let opt_name = get_osstr_longopt_name(self.args[i].as_ref(), self.longopt_prefix);
                ItemClass::Warn(ItemW::UnknownLong(i, opt_name))
            },
            // Reminder, this can obviously only occur with 'in-same-arg' data values.
            ItemClass::Warn(ItemW::LongWithUnexpectedData{ i, n, .. }) => {
                let index = match opt_mode {
                    OptionsMode::Standard => DOUBLE_DASH_PREFIX.len(),
                    OptionsMode::Alternate => SINGLE_DASH_PREFIX.len(),
                } + n.len() + "=".len();
                let data = get_osstr_suffix(self.args[i].as_ref(), index);
                ItemClass::Warn(ItemW::LongWithUnexpectedData{ i, n, d: data })
            },
        }
    }
}

impl ShortOptSetData {
    /// Reset for new argument short option set
    #[inline]
    fn reset(&mut self, arg_index: usize) {
        self.last_arg = arg_index;
        self.consumed = set_shortset_consumed_init!();
    }
}

//TODO: THIS HELPER IS A NECESSARY HACK DUE TO LACK OF OSSTR SLICING IN STD
// `std` currently provides no proper clean & safe way to slice an `OSStr`. On both Unix and Windows
// `OsStr` holds the string in a `u8` slice. On Unix, this *may* or may not be valid UTF-8, but is
// an arbitrary byte sequence. On Windows, again it is an arbitrary sequence, where the input is
// decoded (very simply) from a `u16` sequence, encoding it as a WTF-8 (loose UTF-8) encoding, thus
// is possibly valid UTF-8, but may be arbritary.
#[inline]
fn get_osstr_suffix(os_str: &OsStr, offset: usize) -> &OsStr {
    OsStr::from_bytes(&os_str.as_bytes()[offset..])
}

//TODO: THIS HELPER IS A NECESSARY HACK DUE TO LACK OF OSSTR SLICING IN STD
// Similar to `get_osstr_suffix`, `std` currently provides no proper clean & safe way to split an
// `OsStr` based on a search pattern. For retrieving a long name, we have some complications. For
// matches we want to use the full option name, from the option set, in case we matched on an
// abbreviation, but that's `str` not `OsStr`, unless we can use the conversion with a correct
// lifetime. For unknown options we need to use the original string, but need to beware the possible
// in-same-arg value and its '=' separator. To split there, avoiding issues with lossy conversions
// in the name, we need to transmute as a byte slice, then find the position, if any, of the
// separator byte. We also need to skip the option prefix.
fn get_osstr_longopt_name<'a>(os_str: &'a OsStr, longopt_prefix_osstr: &OsStr) -> &'a OsStr {
    let os_str_bytes = os_str.as_bytes();

    let longopt_prefix_len = longopt_prefix_osstr.as_bytes().len();

    // Hunt for 'in-same-arg' data value separator
    let mut end = None;
    for (i, b) in os_str_bytes.iter().enumerate() {
        if *b == b'=' {
            end = Some(i);
            break;
        }
    }
    let end = end.unwrap_or(os_str_bytes.len());
    let range = Range { start: longopt_prefix_len, end };

    OsStr::from_bytes(&os_str_bytes[range])
}

/// Helper for short option set item conversion.
///
/// The `bytes_consumed` parameter is a pointer to an integer in which we will track the number of
/// bytes of the short option set argument string (`arg`) consumed so far. This function is given a
/// new short option character returned from the `str` based parser, the UTF-8 encoded byte size of
/// which it should increase the tracker integer by.
///
/// The point of tracking this is for use in extracting an in-same-arg data value.
///
/// A complication is encountering a unicode replacement character; this could occur either from a
/// user actually giving one in the input (which should not match a real short option because we
/// forbid its use), or from a sequence of invalid bytes in the lossy conversion of the arg given to
/// the `str` based parser, in which case we need to determine just exactly how many bytes this
/// replaced in the original string. Note, the rules of unicode replacement character replacement
/// are, as determined from experimentation (the following `std` documentation pages helped, but did
/// not explain this properly: [1], [2], [3]), sequences of invalid bytes that are related (e.g.
/// the first three bytes of a prematurely ended four-byte character) result in a single replacement
/// character, while those unrelated each result in one. Naturally a sequence including related and
/// unrelated will result in multiple, one per related set, one per non-related. An ‘overlong’
/// sequence will result in one replacement character per byte.
///
/// [1]: https://doc.rust-lang.org/nightly/std/str/fn.from_utf8.html
/// [2]: https://doc.rust-lang.org/nightly/std/str/struct.Utf8Error.html
/// [3]: https://doc.rust-lang.org/nightly/std/string/struct.String.html#method.from_utf8_lossy
fn track_short_opt_set(arg: &OsStr, bytes_consumed: &mut usize, c: char) {
    *bytes_consumed += match c {
        #[cfg(not(windows))]
        REPLACEMENT_CHARACTER => {
            let slice = &arg.as_bytes()[*bytes_consumed..];
            get_urc_bytes(OsStr::from_bytes(slice))
        },
        #[cfg(windows)]
        REPLACEMENT_CHARACTER => {
            // On Windows, the `OsStr` is also holding a UTF-8 based byte sequence (WTF-8 format),
            // but lossy conversion is a WTF-8 to UTF-8 conversion, which does the minimal amount of
            // work (only swapping encodings of code points U+D800 to U+DFFF), and such code points
            // come out as a single replacement character, unlike on Unix where there is one per
            // byte (3).
            //
            // Actual replacement characters are three bytes (note that as per the Unix
            // implementation we can ignore the possibility of the ‘overlong’ 4-byte form), and the
            // encodings of “unpaired surrogate” code points (U+D800 to U+DFFF) are also three
            // bytes, thus the answer here is always three.
            3
        },
        _ => c.len_utf8(),
    };
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
