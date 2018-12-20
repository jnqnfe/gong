// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//TODO: Merge remaining tests here into the general `parsing` set

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use std::ffi::OsStr;
use gong::analysis::*;
use common::{get_parser, Actual, Expected, check_result};

/// This tests most of the variations that need to be converted from an `str` based `Analysis` to an
/// `OsStr` based one. The things missing are long and short options with missing data, and a
/// command example.
#[test]
fn basic() {
    let args = arg_list!(
        "abc",              // Positional
        "--xxx",            // Unknown long option
        "--help",           // Known long option
        "--hah=abc",        // Known long, with in-same-arg data
        "--hah", "abc2",    // Known long, with in-next-arg data
        "--hah=",           // Known long option, taking data, which is empty string, in-same-arg
        "--foo=",           // Known long option, does not take data, empty data ignored
        "--foo=xyz",        // Known long option, does not take data, unexpected data
        "--fo",             // Ambiguous long option
        "--=",              // Long with no name, and empty data
        "--=xyz",           // Long with no name, and non-empty data
        "-m",               // Unknown short option
        "-h",               // Known short option
        "-oarg",            // Known short option, with in-same-arg data
        "-o", "arg2",       // Known short option, with in-next-arg data
        "--",               // Early terminator
    );
    let expected = expected!(
        error: true,
        warn: true,
        [
            expected_item!(0, Positional, "abc"),
            expected_item!(1, UnknownLong, "xxx"),
            expected_item!(2, Long, "help"),
            expected_item!(3, LongWithData, "hah", "abc", DataLocation::SameArg),
            expected_item!(4, LongWithData, "hah", "abc2", DataLocation::NextArg),
            expected_item!(6, LongWithData, "hah", "", DataLocation::SameArg),
            expected_item!(7, Long, "foo"),
            expected_item!(8, LongWithUnexpectedData, "foo", "xyz"),
            expected_item!(9, AmbiguousLong, "fo"),
            expected_item!(10, LongWithNoName),
            expected_item!(11, LongWithNoName),
            expected_item!(12, UnknownShort, 'm'),
            expected_item!(13, Short, 'h'),
            expected_item!(14, ShortWithData, 'o', "arg", DataLocation::SameArg),
            expected_item!(15, ShortWithData, 'o', "arg2", DataLocation::NextArg),
            expected_item!(17, EarlyTerminator),
        ]
    );
    check_result(&Actual(get_parser().parse(&args)), &expected);
}

/// This tests some example items using multi-byte characters, where slicing of the input arguments
/// is involved.
#[test]
fn multi_byte() {
    let args = arg_list!(
        "--x❤x",            // Unknown long option
        "--ábc",           // Known long option
        "--ƒƒ=💖abc",        // Known long, with in-same-arg data
        "--ƒƒ", "abc💖",     // Known long, with in-next-arg data
        "--ƒƒ=",            // Known long option, taking data, which is empty string, in-same-arg
        "--ábc=",          // Known long option, does not take data, empty data ignored
        "--ábc=x💖z",       // Known long option, does not take data, unexpected data
        "--ƒ",              // Ambiguous long option
        "--=",              // Long with no name, and empty data
        "--=x💖z",           // Long with no name, and non-empty data
        "-ă",               // Unknown short option
        "-❤",               // Known short option
        "-Ɛaşrg",           // Known short option, with in-same-arg data
        "-Ɛ", "argş",       // Known short option, with in-next-arg data
    );
    let expected = expected!(
        error: true,
        warn: true,
        [
            expected_item!(0, UnknownLong, "x❤x"),
            expected_item!(1, Long, "ábc"),
            expected_item!(2, LongWithData, "ƒƒ", "💖abc", DataLocation::SameArg),
            expected_item!(3, LongWithData, "ƒƒ", "abc💖", DataLocation::NextArg),
            expected_item!(5, LongWithData, "ƒƒ", "", DataLocation::SameArg),
            expected_item!(6, Long, "ábc"),
            expected_item!(7, LongWithUnexpectedData, "ábc", "x💖z"),
            expected_item!(8, AmbiguousLong, "ƒ"),
            expected_item!(9, LongWithNoName),
            expected_item!(10, LongWithNoName),
            expected_item!(11, UnknownShort, 'ă'),
            expected_item!(12, Short, '❤'),
            expected_item!(13, ShortWithData, 'Ɛ', "aşrg", DataLocation::SameArg),
            expected_item!(14, ShortWithData, 'Ɛ', "argş", DataLocation::NextArg),
        ]
    );
    check_result(&Actual(get_parser().parse(&args)), &expected);
}

/// These test some example items using invalid UTF-8 byte sequences, where the invalid sequences
/// will be changed to unicode replacement characters in the inner `str` based parsing engine, and
/// we need to correctly map those to the original bytes for correct `OsStr` based parsing results.
///
/// Note, we assume that available option/command names are always valid UTF-8, as we require.
mod invalid_byte_sequences {
    use super::*;

    /*
     * On Unix, UTF-8 is used for OS strings, though without proper enforcement of all of the rules
     * of valid encodings. In Rust, an `OsStr`/`OsString` uses a simple `u8` sequence to represent
     * an OS string, and a lossy conversion to `str` form (truly valid UTF-8) simply involves a full
     * validation check, replacing invalid sequences with a unicode replacement character (U+FFFD).
     *
     * On Windows, UTF-16 is used for OS Strings, though again, not enforcing all of the rules
     * (which are more simple than with UTF-8, simply forbidding unpaired surrogates). In Rust, an
     * `OSStr`/`OsString` uses WTF-8 (wobbly transformation format, 8-bit) to represent as OS
     * string. Storing an OS string in an `OsStr`/`OsString` simply involves a basic UTF-16 decoding
     * to code points, then re-encoding in UTF-8 form, storing in a WTF-8 based type, which does
     * lossy conversion to `str` form by simply replacing sequences that represent unpaired UTF-16
     * surrogates with the replacement character. The way the original UTF-16 is transformed into
     * WTF-8 ensures that it is always valid WTF-8.
     *
     * To create invalid strings for testing:
     *
     *  - On Unix, there are many single bytes that are invalid, for instance `0x80`, a continuation
     *    byte, and since the OS string byte sequence is simply stored in `OsStr/`OsString`, we can
     *    simply just store such a byte in one.
     *  - On Windows, a byte like `0x80` directly injected into an `OsStr`/`OsString`, e.g. via a
     *    transmute, creates an invalid WTF-8, which then transforms lossily into an invalid `str`
     *    form. As mentioned above, the conversion of an OS string into WTF-8 would not allow such
     *    a lone byte to end up in the WTF-8 string. We must use something else which is valid in
     *    WTF-8, which gets swapped with a replacement character in lossy `str` conversion, for
     *    instance a code point like `0xD800`, i.e. [ 0xed, 0xa0, 0x80 ].
     *
     * Hence the below test is different for Unix vs. Windows.
     *
     * Note, the only way to create invalid strings here is by specifying them as a byte sequence.
     * Rust does not allow creating literals with invalid bytes. (Naturally users of a program
     * using this library would not have any trouble providing strings containing invalid byte
     * sequences, for instance with a Unix command line, a user could simply enter an argument like
     * `--a$'\x80'b` to inject an invalid 0x80 continuation byte).
     */

    #[cfg(any(unix, target_os = "redox"))]
    #[test]
    fn unix() {
        use std::os::unix::ffi::OsStrExt;

        let args = [
            OsStr::from_bytes(b"a\x80bc"),       // Positional
            OsStr::from_bytes(b"--\x80xx"),      // Unknown long option
            OsStr::from_bytes(b"--hah=a\x80bc"), // Known long, with in-same-arg data
            OsStr::from_bytes(b"--hah"),         // Known long, with in-next-arg data
            OsStr::from_bytes(b"abc\x80"),       // The in-next-arg data
            OsStr::from_bytes(b"--foo=\x80xyz"), // Known long option, does not take data, unexpected data
            OsStr::from_bytes(b"--=xy\x80z"),    // Long with no name, and non-empty data
            OsStr::from_bytes(b"-m\x80h"),       // Short option set, unknown, invalid, known
            OsStr::from_bytes(b"-oar\x80g"),     // Known short option, with in-same-arg data
            OsStr::from_bytes(b"-o"),            // Known short option, with in-next-arg data
            OsStr::from_bytes(b"\x80arg"),       // The in-next-arg data
            // Multiple invalid short option set bytes, which, being entirely unrelated (not a
            // prematurely-terminated multi-byte sequence) should each result in a single unicode
            // replacement character.
            OsStr::from_bytes(b"-\x80\x81\x82"),
            // This uses the first three bytes of a four byte character, checking that these being
            // related actually results in only one single unicode replacement character, not three.
            OsStr::from_bytes(b"-\xf0\x9f\x92"),
            // This uses the first two bytes of a four byte character, checking that this does not
            // trip up the check for a real replacement character, which uses a slice of three
            // bytes, which could panic if done wrong.
            OsStr::from_bytes(b"-\xf0\x9f"),
            // This uses a mix of basic known/unknown simple characters, including multibyte; an
            // incomplete multi-byte char; a simple invalid single-byte; and an actual unicode
            // replacement char (U+FFFD). The purpose is partly to throw lots of stuff into the mix
            // together, and partly to check that a replacement character itself is handled
            // correctly. Note, a byte string is ASCII only, thus: `\xe2\x9d\xa4` is `❤`; `\xc5\x9f`
            // is `ş`; and `\xef\xbf\xbd` is the rep char.
            OsStr::from_bytes(b"-m\xe2\x9d\xa4a\xc5\x9f\xf0\x9f\x92j\x80\xef\xbf\xbdk"),
            // Here we have a situation of in-same-arg data, with invalid bytes within the
            // proceeding short option set characters, thus checking whether or not we correctly
            // extract the right byte slice for the data value. An invalid byte is also present in
            // the value for good measure.
            OsStr::from_bytes(b"-m\x80\x81\x82oar\xf0\x9f\x92g"),
            // And what happens if an actual unicode replacement character (u+FFFD) is given?
            OsStr::from_bytes(b"-m\xef\xbf\xbdoar\x84\x85g"),
            // Finally, let's assert that a combined sequence of invalid bytes, of both related and
            // unrelated, comes out as the number of unicode replacement characters we expect. The
            // following should be four.
            OsStr::from_bytes(b"-\x80\x81\xef\xbf\xbd\x82"),
        ];

        let expected_strings = [
            OsStr::from_bytes(b"a\x80bc"),
            OsStr::from_bytes(b"\x80xx"),
            OsStr::from_bytes(b"a\x80bc"),
            OsStr::from_bytes(b"abc\x80"),
            OsStr::from_bytes(b"\x80xyz"),
            OsStr::from_bytes(b"ar\x80g"),
            OsStr::from_bytes(b"\x80arg"),
            OsStr::from_bytes(b"ar\xf0\x9f\x92g"),
            OsStr::from_bytes(b"ar\x84\x85g"),
        ];

        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, Positional, expected_strings[0]),
                expected_item!(1, UnknownLong, expected_strings[1]),
                expected_item!(2, LongWithData, "hah", expected_strings[2], DataLocation::SameArg),
                expected_item!(3, LongWithData, "hah", expected_strings[3], DataLocation::NextArg),
                expected_item!(5, LongWithUnexpectedData, "foo", expected_strings[4]),
                expected_item!(6, LongWithNoName),
                expected_item!(7, UnknownShort, 'm'),
                // Note, here, it is right that we do not receive the original invalid byte(s) as
                // the unrecognised short option, since it would be a pain to determine exactly what
                // byte(s) were turned into each individual unicode replacement char that was
                // analysed by the inner `str` based parser, which would also potentially involve
                // merging some of its analysis items. Thus we expect a replacement char here.
                expected_item!(7, UnknownShort, '�'),
                expected_item!(7, Short, 'h'),
                expected_item!(8, ShortWithData, 'o', expected_strings[5], DataLocation::SameArg),
                expected_item!(9, ShortWithData, 'o', expected_strings[6], DataLocation::NextArg),
                expected_item!(11, UnknownShort, '�'), // Notice three individual instances for arg 11
                expected_item!(11, UnknownShort, '�'),
                expected_item!(11, UnknownShort, '�'),
                expected_item!(12, UnknownShort, '�'), // Note only one instance for arg 12
                expected_item!(13, UnknownShort, '�'), // Note only one instance for arg 13
                expected_item!(14, UnknownShort, 'm'),
                expected_item!(14, Short, '❤'),
                expected_item!(14, UnknownShort, 'a'),
                expected_item!(14, UnknownShort, 'ş'),
                expected_item!(14, UnknownShort, '�'), // This one is from the incomplete multi-byte
                expected_item!(14, UnknownShort, 'j'),
                expected_item!(14, UnknownShort, '�'), // This one is from the other invalid byte
                expected_item!(14, UnknownShort, '�'), // This one is from the actual U+FFFD char
                expected_item!(14, UnknownShort, 'k'),
                expected_item!(15, UnknownShort, 'm'),
                expected_item!(15, UnknownShort, '�'),
                expected_item!(15, UnknownShort, '�'),
                expected_item!(15, UnknownShort, '�'),
                expected_item!(15, ShortWithData, 'o', expected_strings[7], DataLocation::SameArg),
                expected_item!(16, UnknownShort, 'm'),
                expected_item!(16, UnknownShort, '�'),
                expected_item!(16, ShortWithData, 'o', expected_strings[8], DataLocation::SameArg),
                expected_item!(17, UnknownShort, '�'),
                expected_item!(17, UnknownShort, '�'),
                expected_item!(17, UnknownShort, '�'),
                expected_item!(17, UnknownShort, '�'),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }

    #[cfg(windows)]
    #[test]
    fn windows() {
        // Necessary hack due to lack of access to raw bytes on Windows currently
        pub trait OsStrExt {
            fn from_bytes(slice: &[u8]) -> &Self;
        }
        impl OsStrExt for OsStr {
            fn from_bytes(slice: &[u8]) -> &OsStr { unsafe { std::mem::transmute(slice) } }
        }

        let args = [
            OsStr::from_bytes(b"a\xed\xa0\x80bc"),       // Positional
            OsStr::from_bytes(b"--\xed\xa0\x80xx"),      // Unknown long option
            OsStr::from_bytes(b"--hah=a\xed\xa0\x80bc"), // Known long, with in-same-arg data
            OsStr::from_bytes(b"--hah"),                 // Known long, with in-next-arg data
            OsStr::from_bytes(b"abc\xed\xa0\x80"),       // The in-next-arg data
            OsStr::from_bytes(b"--foo=\xed\xa0\x80xyz"), // Known long option, does not take data, unexpected data
            OsStr::from_bytes(b"--=xy\xed\xa0\x80z"),    // Long with no name, and non-empty data
            OsStr::from_bytes(b"-m\xed\xa0\x80h"),       // Short option set, unknown, invalid, known
            OsStr::from_bytes(b"-oar\xed\xa0\x80g"),     // Known short option, with in-same-arg data
            OsStr::from_bytes(b"-o"),                    // Known short option, with in-next-arg data
            OsStr::from_bytes(b"\xed\xa0\x80arg"),       // The in-next-arg data
            // Multiple invalid short option set character byte sequences, which should each result
            // in a single unicode replacement character.
            OsStr::from_bytes(b"-\xed\xa0\x80\xed\xa0\x81\xed\xa0\x82"),
            // This uses a mix of basic known/unknown simple characters, including multibyte; an
            // an invalid sequence (unpaired surrogate); and an actual unicode replacement char
            // (U+FFFD). The purpose is partly to throw lots of stuff into the mix together, and
            // partly to check that a replacement character itself is handled correctly. Note, a
            // byte string is ASCII only, thus: `\xe2\x9d\xa4` is `❤`; `\xc5\x9f` is `ş`; and
            // `\xef\xbf\xbd` is the rep char.
            OsStr::from_bytes(b"-m\xe2\x9d\xa4a\xc5\x9fj\xed\xa0\x80\xef\xbf\xbdk"),
            // Here we have a situation of in-same-arg data, with invalid bytes within the
            // proceeding short option set characters, thus checking whether or not we correctly
            // extract the right byte slice for the data value. Invalid bytes are also present in
            // the value for good measure.
            OsStr::from_bytes(b"-m\xed\xa0\x80\xed\xa0\x81\xed\xa0\x82oar\xed\xa0\x83g"),
            // And what happens if an actual unicode replacement character (u+FFFD) is given?
            OsStr::from_bytes(b"-m\xef\xbf\xbdoar\xed\xa0\x84\xed\xa0\x85g"),
        ];

        let expected_strings = [
            OsStr::from_bytes(b"a\xed\xa0\x80bc"),
            OsStr::from_bytes(b"\xed\xa0\x80xx"),
            OsStr::from_bytes(b"a\xed\xa0\x80bc"),
            OsStr::from_bytes(b"abc\xed\xa0\x80"),
            OsStr::from_bytes(b"\xed\xa0\x80xyz"),
            OsStr::from_bytes(b"ar\xed\xa0\x80g"),
            OsStr::from_bytes(b"\xed\xa0\x80arg"),
            OsStr::from_bytes(b"ar\xed\xa0\x83g"),
            OsStr::from_bytes(b"ar\xed\xa0\x84\xed\xa0\x85g"),
        ];

        let expected = expected!(
            error: false,
            warn: true,
            [
                expected_item!(0, Positional, expected_strings[0]),
                expected_item!(1, UnknownLong, expected_strings[1]),
                expected_item!(2, LongWithData, "hah", expected_strings[2], DataLocation::SameArg),
                expected_item!(3, LongWithData, "hah", expected_strings[3], DataLocation::NextArg),
                expected_item!(5, LongWithUnexpectedData, "foo", expected_strings[4]),
                expected_item!(6, LongWithNoName),
                expected_item!(7, UnknownShort, 'm'),
                expected_item!(7, UnknownShort, '�'),
                expected_item!(7, Short, 'h'),
                expected_item!(8, ShortWithData, 'o', expected_strings[5], DataLocation::SameArg),
                expected_item!(9, ShortWithData, 'o', expected_strings[6], DataLocation::NextArg),
                expected_item!(11, UnknownShort, '�'), // Notice three individual instances for arg 11
                expected_item!(11, UnknownShort, '�'),
                expected_item!(11, UnknownShort, '�'),
                expected_item!(12, UnknownShort, 'm'),
                expected_item!(12, Short, '❤'),
                expected_item!(12, UnknownShort, 'a'),
                expected_item!(12, UnknownShort, 'ş'),
                expected_item!(12, UnknownShort, 'j'),
                expected_item!(12, UnknownShort, '�'),
                expected_item!(12, UnknownShort, '�'), // This one is from the actual U+FFFD char
                expected_item!(12, UnknownShort, 'k'),
                expected_item!(13, UnknownShort, 'm'),
                expected_item!(13, UnknownShort, '�'),
                expected_item!(13, UnknownShort, '�'),
                expected_item!(13, UnknownShort, '�'),
                expected_item!(13, ShortWithData, 'o', expected_strings[7], DataLocation::SameArg),
                expected_item!(14, UnknownShort, 'm'),
                expected_item!(14, UnknownShort, '�'),
                expected_item!(14, ShortWithData, 'o', expected_strings[8], DataLocation::SameArg),
            ]
        );
        check_result(&Actual(get_parser().parse(&args)), &expected);
    }
}
