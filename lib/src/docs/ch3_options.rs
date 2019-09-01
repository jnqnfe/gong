// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Option argument support
//!
//! This crate has been designed around standard *option* conventions.
//!
//! Basic feature support is on par with the C `getopt_long` function. (See the [overview] chapter
//! for mention of the small differences).
//!
//! # Option styles
//!
//! There are two different common *option* styles this library can parse, which we refer to in this
//! library as the *option* parsing “mode”:
//!
//! - Standard mode (default) supports traditional *long* and *short* *options*, where *long
//!   options* use a “double-dash” prefix (e.g. `--help`), and *short options* use a single dash
//!   (e.g. `-h`).
//! - Alternate mode supports *long options* only (no *short options*), with a single dash prefix
//!   (i.e. `-help` rather than `--help`). Some people simply prefer this style.
//!
//! > **Note:** *Short options* can still be added to an *option set* when using this mode, it will
//! > still pass as valid; they will simply be ignored when performing matching.
//!
//! Which style an argument list will be parsed with must be specified in the parser’s settings.
//!
//! Note, from this point on, the below discussion is written towards *standard* mode conventions,
//! though everything applies equally to *alternate* mode, only with a few obvious details adjusted
//! and stuff relating to *short options* ignored, as applicable.
//!
//! # Basic parsing model
//!
//! The fundamental argument parsing logic follows this model:
//!
//!  - An argument either **not** starting with a dash (`-`), or consisting only of a single dash,
//!    is **not** an *option* argument.
//!  - An argument of exactly two dashes (`--`) only is **not** an *option* argument, it is an
//!    *early terminator*.
//!  - An argument starting with two dashes (`--`) followed by additional characters is a *long
//!    option*. The portion after the double-dash prefix is the *long option name* (or possibly the
//!    name combined with an “in-same-argument” *data value*, as discussed below).
//!  - An argument starting with a single dash (`-`) followed by additional (non-dash) characters is
//!    a *short option set*, where each character (`char`) after the single-dash prefix represents a
//!    *short option* (except with respect to “in-same-argument” *data values*, as discussed below).
//!    (More than one *short option* can be specified in a single argument). Note that a dash (`-`)
//!    itself is not permitted to be a valid program *short option* (it would be misinterpreted in
//!    some cases). Note also that interpretation of what consists of a “character” may surprise you
//!    as actually being a complicated matter, as per the dedicated [unicode related
//!    discussion][unicode].
//!
//! Naturally, for *alternate* mode *option* style, ignore the discussion of *short options*, and
//! remember that only a single dash is used as the prefix for *long options*.
//!
//! Arguments are parsed in sequence, one at a time; Parsing of each argument may alter how one or
//! more subsequent arguments are interpreted, deviating from the above. For instance, consider the
//! effect of the *early terminator* and of *options* with “in-next-argument” data values, as
//! discussed below.
//!
//! Note, *option* matching is case-sensitive.
//!
//! # Data values
//!
//! *Long* and *short* *options* can be configured as either of “flag” type or of “data taking”
//! type, with the latter taking a single accompanying *data value*. This value can be configured as
//! either mandatory, or optional, per option. In the latter case certain restrictions apply on
//! their use.
//!
//! Mandatory *data values* for both *long* and *short* *options* can either be supplied within the
//! same argument as the *option* itself (“in-same-argument”), or as the next argument
//! (“in-next-argument”), in which case that will thus be consumed as the *option*’s *data value*
//! and otherwise ignored. In the case of optional *data values*, these are only ever consumed from
//! within the same argument.
//!
//! ## With long options
//!
//! For *long options*, “in-next-argument” style looks like `--foo bar`, while “in-same-argument”
//! style uses an equals (`=`) character between the *option name* and *value* components, e.g.
//! `--foo=bar`. Note that a program’s “available” *long options* are forbidden from containing an
//! equals (`=`) character in their name as this would otherwise introduce significant problems.
//!
//! When parsing a *long option* argument, if the argument contains one or more equals (`=`)
//! characters then it is considered to have an “in-same-argument” *data value* (since names are not
//! permitted to contain them), even if that equals character is the last character in the string
//! (to allow users to signal an empty string as being the intended string value). It thus splits it
//! up into two components upon the first equals. The left hand portion (without the double-dash
//! prefix) is taken as the name, and the right as the “in-same-argument” *data value*, with the
//! equals (`=`) separator being discarded. For example, `--foo=bar` translates to a name of “foo”
//! and a value of “bar”). This naturally occurs **before** checking for a matching “available”
//! program *option*.
//!
//! The components are then handled thusly:
//!
//!  - If the name component does not match any “available” *long option*, then it is reported as
//!    unknown, with any “in-same-argument” *data value* component ignored.
//!  - If a match is found, then what happens depends upon the type of option matched against:
//!
//!     - For “flag” type options (those that **do not** take a *data value*), if no
//!       “in-same-argument” *data value* component was supplied, then all is good, otherwise its
//!       presence is reported as unexpected. However if it is an empty string then here an empty
//!       string will simply be ignored, i.e. `--foo=` is treated as though it were just `--foo`.
//!     - For mandatory data-taking options (those that require a value), then if an
//!       “in-same-argument” *data value* component was present, this is consumed as such (even if
//!       an empty string such as in `--foo=`), otherwise the next argument is consumed. If in an
//!       “in-next-argument” situation, the next argument is an empty string (e.g. as in`--foo ""`),
//!       the *data value* is also accepted as such. If in an “in-next-argument” situation there is
//!       no next argument, then the *data value* is reported as missing.
//!     - For optional data-taking options (those where supplying a value is optional), remember
//!       that a value can only be provided “in-same-argument” style. If an “in-same-argument”
//!       *data value* component was present, this is consumed as such in an identical manner to the
//!       mandatory type. If no “in-same-argument” value was supplied, no value is returned with the
//!       item, in the same way as with flag type options, allowing this case to be distinguished
//!       from an empty string (i.e. `--foo` and `--foo=` are treated differently).
//!
//! ## With short options
//!
//! For *short options*, “in-next-argument” style looks like `-o val`, while “in-same-argument”
//! style looks like `-oval`.
//!
//! When a *short option set* is encountered (remember, more than one *short option* can be grouped
//! together in the same argument), the characters are gone through in sequence, looking for a
//! matching program *short option* for each.
//!
//!  - If no match is found then it is reported as unknown.
//!  - If a match is found that **does not** take a *data value* (i.e. is a “flag” type option),
//!    then the match is simply reported.
//!  - If a match is found that **does** take a *data value*, then one needs to be found. If this
//!    character is **not** the last in the set, then the remaining portion of the argument is
//!    consumed as this *option*’s *data value* (for example, if `o` is such an *option* then in
//!    `-oval`, `val` is it’s *data value*). If it is the last character in the set, and this option
//!    **requires** a value, then the next argument is consumed (for example with `-o val` the value
//!    is `val`). If in an “in-next-argument” situation there is no next argument, then the *data
//!    value* is reported as missing. If supplying a value is optional (remember that this can only
//!    be done “in-same-argument” style) and no value is supplied, the value is considered to be an
//!    empty string.
//!
//! Note that it is not possible to supply an empty string as an “in-same-argument” data value, and
//! thus not possible at all with an option where supplying a value is optional.
//!
//! Naturally when multiple *short options* are grouped together in the same argument, only the last
//! in that group can be one that takes a *data value*, and users must be careful when constructing
//! such groups.
//!
//! # Abbreviated long option name matching
//!
//! Abbreviated *long option* name matching is a feature whereby an abbreviated form of a *long
//! option’s* name can be used and matched successfully to the *option*, so long as the abbreviation
//! uniquely matches a single *long option*.
//!
//! This is supported and is optional. It is enabled by default, but can be opted out of when
//! parsing, if not desired, through a parser setting.
//!
//! As an example, consider the input arguments from the following command line:
//!
//! ```text
//! <progname> --f --fo --foo --foob --fooba --foobar
//! ```
//!
//! If the feature is enabled, and a set of options consisting of `foo` and `foobar`, then:
//!
//!  - `--foo` and `--foobar` are exact matches for the available `foo` and `foobar` *options*
//!    respectively.
//!  - `--f` and `--fo` are invalid as being ambiguous (and reported as such by the parser).
//!  - `--foob` and `--fooba` both uniquely match `foobar` and so are valid.
//!
//! Note that an exact match always takes precedence.
//!
//! [overview]: ../ch1_overview/index.html
//! [commands]: ../ch4_commands/index.html
//! [unicode]: ../ch5_unicode/index.html
