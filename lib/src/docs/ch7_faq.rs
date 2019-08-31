// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: FAQ
//!
//! # How do I specify “positional” arguments?
//!
//! Positionals do not need to be described to the parser in the way that options are. Instead the
//! parser is only concerned with quantity, such that it can highlight unexpected positionals as a
//! problem once you go over a maximum number, that is if you choose to set one (you can
//! alternatively set the maximum to *unlimited*). See the information in the
//! [usage documentation][usage_doc].
//!
//! # How do I specify conditions where the quantity of positionals change?
//!
//! As in where the number of positionals to accept changes depending upon conditions such as the
//! use of a particular option? You don't. This libary does not provide a framework for expressing
//! such logic, considering application-specific logic to be better, as discussed below (see the
//! answer to the question about option relationships).
//!
//! However, consider the following: The quantity requirement specified to the parser is a *maximum*
//! which naturally offers some flexibility on its own. If this is insufficient then note that the
//! parser iterators provide a method for adjusting the quantity originally set in the parser
//! inbetween iterations. Currently there is no means of automatically achieving this with
//! data-mining objects, but you can always use a maximum of unlimited and then just treat some
//! positionals as unexpected yourself.
//!
//! Note that naturally once an argument has been parsed as being either an expected or unexpected
//! positional, this is effectively set in stone; use of options that may demand reduction of the
//! maximum quantity of positionals to accept per your application requirements, cannot change the
//! expected status of positionals already assessed.
//!
//! # How do I specify an option as being “required”?
//!
//! You don’t. If your application has one or more “required” options, that’s an attribute for you
//! to enforce when you react to the analysis.
//!
//! Why? See the answer to the question about option relationships.
//!
//! # How do I specify an option as being “multi-use”/“single-use”?
//!
//! > What is a “multi-use”/“single-use” option? A “multi-use” option is one that a user is allowed
//! > to supply more than once in an argument list, while a “single-use” option is the opposite.
//!
//! You don’t. This library does not enforce multi-use/single-use properties for you and thus has no
//! use for such information; all options are effectively multi-use unless you yourself enforce the
//! single-use property in your program when responding to the analysis given to you by this
//! library. I.e. for multi-use options with data values, just collect up the data; for single-use
//! options, if encountered more than once, output an error and stop your program.
//!
//! Furthermore, the wisdom of actually enforcing such a “single-use” option property is arguably
//! questionable. Do you really need to give an error if a so called “single-use” option is given
//! multiple times in the argument list by a user? With options that do not take data, it surely
//! makes no difference, and with options that do take data, all you *really* need to do is
//! basically just ignore all but the last value given for that option (or allow instances to
//! overwrite values given by earlier ones). Not giving an error is more simple, requiring less
//! code. Giving an error may even create problems, for instance it may not be expected, taking
//! users (or code generating arguments to call your program with) by surprise. It’s up to you
//! though, the flexibility is there for you to make a choice.
//!
//! # How do I specify option relationships?
//!
//! You don’t. This would require a framework for expressing rules, with supporting processing
//! logic, capable of handling no end of potential complexity. Such a thing is only desirable in a
//! library that tries to do everything for you, which this library deliberately does not. In my
//! opinion it is much better to let you, the application programmer, write the necessary code
//! specific to your application; it should be typically trivial in application-specific form, as
//! well as being invariably much more efficient than a bloated dynamic framework. A balance must be
//! struck as to how much of the work this library does and how much you do, and it seems right that
//! this library steers away from attempting to incorporate this.
//!
//! # What is “posixly correct” parsing?
//!
//! It is parsing in a way that conforms to the POSIX/SUS standard. Basically this standard requires
//! that a user must specify options before positional arguments, i.e. they cannot be intermixed,
//! thus an argument parser must assume (and interpret as such) that all subsequent arguments after
//! encountering a positional argument must also be positional arguments. By default this library
//! allows free intermixing, but you can control this via a setting. See the
//! [`Settings::set_posixly_correct`] method documentation for a little more info.
//!
//! [`Settings::set_posixly_correct`]: ../../parser/struct.Settings.html#method.set_posixly_correct
//!
//! # Can supplying a command be optional?
//!
//! If your program uses command arguments, then a user can legitimately neglect to use them, for
//! instance consider ```<prog-name> --help```. However, with respect to positionals, it is another
//! story. The first non-option given in the arguments list, and similarly following each command
//! that has a sub-command set, is expected to be a command, and thus will either end up as a
//! matched (known) or an unmatched (unknown) command item. Thus a user cannot provide a non-option
//! argument in such a situation and expect it to come out as a positional item on the basis of it
//! not matching a known command; it will come out as an unknown command item.
//!
//! If this is not what you want then you can simply treat the unknown command item as a positional
//! (the string returned is the original `&OsStr` just as with positionals).
//!
//! This may change in future, as it would not be difficult to build in an option to control this
//! behaviour, but currently this is not so, and it is not known that such an option is desirable
//! or even wanted.
//!
//! # What exactly is “mismatch suggestions”?
//!
//! This is a feature that assists in generating more helpful error messages when a user supplies an
//! unknown long option or command to your program. Instead of just outputting an error that points
//! out the problematic argument, it may be helpful to users if you can included in that error
//! message a suggestion of the option/command that most closely resembles what they tried to use,
//! if any real ones are a close enough match. (Note, this does not apply to *short* options,
//! obviously).
//!
//! Thus, this feature consists of a means by which a supplied unknown option/command can be
//! compared with the set of available options/commands you described for your program, and the best
//! suggestion, if any, obtained.
//!
//! This capability is provided by this crate, built upon use of a string comparison measuring
//! algorithm (specifically `jaro_winkler`) from the `strsim` crate. This is an optional feature
//! controlled via the `suggestions` `Cargo` feature.
//!
//! # Is option and/or command argument matching case-sensitive?
//!
//! Yes for both.
//!
//! # Is any whitespace trimming (or other modification) performed?
//!
//! No. To be more clear: No modifications are done to the original input argument strings, or the
//! list, they are treated as immutable; the library only returns whole or partial slices in the
//! analysis item data it returns; whitespace is left exactly as is and is not treated in any
//! special way at all, and its presence in option input arguments affects option matching.
//!
//! Some examples:
//!
//! ```text
//! me:~$> <prog-name> "--f o o=   hello    world   !"
//! name: "f o o"
//! data-value: "   hello    world   !"
//! ```
//!
//! Here the surrounding quotes force the terminal to treat everything between them as one single
//! input argument; the library would try to match everything between the double-dash prefix (`--`)
//! and the equals (`=`) exactly as given, as an *option name*, and provide everything after the
//! equals (`=`) exactly as given as the *data value*. The latter two lines in the example
//! demonstrate our pretend program having printed the given name and data components in question.
//!
//! ```text
//! me:~$> <prog-name> "-f o"
//! ```
//!
//! Here again the quotes force it to be one input argument. The library would see three short
//! options here, trying to match `'f'`, `' '` (space) and `'o'` individually (unless `'f'` were a
//! matched data taking option, in which case the other two characters (`" o"`) would be consumed as
//! its *data value*).
//!
//! ```text
//! me:~$> <prog-name> " --foo"
//! ```
//!
//! Here the quotes force the leading space to be preserved by the terminal. The presence of the
//! space causes the library to see this input argument as a *non-option* of `" --foo"`, and thus
//! would **not** try to match any `foo` option, it would be taken to be a *positional*.
//!
//! Note that all of these examples required quotes to **force** the creation of potential option
//! match problems. The likelihood of users making genuine such mistakes is quite low.
//!
//! The decision to not trim whitespace for *data values* is obviously important, and similarly so
//! for *positionals*. The decision to not do so for option names/`char`s was a deliberate design
//! choice, for efficiency, simplicity of parsing implementation, and flexibility of requirements.
//! It is not expected to present any usability problem for users of programs, nor put anyone off
//! using this library in their program. Note that you always do have the option of ignoring
//! unrecognised whitespace short options. Trimming of whitespace in the last example for instance
//! would place restrictions upon the possible *positional* strings that could be specified to a
//! program, and the trimming could catch users off guard, being unexpected behaviour.
//!
//! [usage_doc]: ../ch6_usage/index.html
