// Copyright 2019 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Input argument handling components
//!
//! The purpose of the types within this module are to simplify the capturing of arguments, avoiding
//! certain easy to overlook pitfalls.
//!
//! The [`Args`] type can either capture real input arguments from the environment, or can be
//! supplied with a custom set of strings. It is then simply to be given to the parser for the
//! arguments to be parsed.
//!
//! # Example
//!
//! A most trivial example of capturing the arguments for parsing:
//!
//! ```rust
//! use gong::arguments::Args;
//! let args = Args::new();
//! ```
//!
//! If it interests you, note that you can get a slice for the held arguments, (from which of course
//! you can also create an iterator), with the [`as_slice`] method.
//!
//! # Program name argument
//!
//! Note that if you want the program name argument for any reason, there is a method for retrieving
//! this (as long as the arguments object was created from real input arguments, not a custom
//! source, which would not be expected to include it):
//!
//! ```rust
//! # use gong::arguments::Args;
//! # let args = Args::new();
//! let prog_name = args.get_prog_name(); // Note, this is `Option` wrapped!
//! ```
//!
//! Note, as pointed out in the Rust `std` library documentation, the text of this argument can be
//! arbitrary and so should not be relied upon for security purposes.
//!
//! # Avoiding Pitfalls
//!
//! It is not necessary to understand the pitfalls of handling input arguments that the [`Args`]
//! type helps you to avoid, but for the sake of being informative, they are described here.
//!
//! ## Special first argument
//!
//! The very first entry in a program’s input argument list is the program path/name. This needs to
//! be skipped over in parsing the actual user input arguments. The [`Args`] type takes care of this
//! for you, though making this argument available via a method should you wish to retrieve it.
//!
//! ## Choosing the wrong source
//!
//! There are two different functions offered by the Rust `std` library for capturing input
//! arguments, `std::env::args()` and `std::env::args_os()`. Picking the wrong one could lead to
//! your program crashing with certain filename or path arguments!
//!
//! Understand that normal Rust strings must be fully valid (UTF-8) Unicode, but strings provided to
//! a program by some Operating Systems (including all the common ones) however are not guaranteed
//! to actually be fully valid Unicode. Rust thus has special `OsStr` and `OsString` types for
//! holding these and methods relating to conversions. Converting from OS form to normal Rust form
//! is not possible if a string is not valid Unicode, unless a “lossy” conversion is done where
//! invalid code points are converted to a “replacement” character. The Rust `std::env::args()`
//! function gives an iterator that tries to perform **non**-lossy conversion to normal Rust string
//! form, panicking if it encounters an invalid Unicode string. This should be undesirable for any
//! program, but would be worse if your program is actually designed to take filename or path
//! inputs. The alternative `std::env::args_os()` function gives an iterator that simply serves
//! arguments in `OsString` form. This requires a little more knowledge to work with, which is
//! probably why the alternative was provided, but comes without the downsides and is what you
//! should actually be using.
//!
//! This library expects to parse strings in OS form (as captured by [`Args`]) and returns OS form
//! types for certain item variants. Worry not, working with the resulting OS strings is not as
//! difficult as some might fear. For instance, if an input value is numberic, you can safely
//! attempt conversion to a normal Rust string with its `to_str()` method (`Option` wrapped result)
//! before attempting conversion to a number, rejecting it simply as an invalid value if conversion
//! fails at either stage.
//!
//! ## Costly arg iterator creation
//!
//! Creation of a Rust `std` library argument iterator instance is more costly than you might
//! realise. You might already suspect that for each argument it has to be converted from a NUL-
//! terminated C string form given by the operating system to a Rust form. You might also probably
//! imagine that this would occur for each argument one at a time per iteration. You would be very
//! wrong about the latter. In actual fact a full conversion of all arguments to the form of
//! `Vec<OsString>` (i.e. a vector of owned OS strings) is done when creating each instance of an
//! argument iterator, and stored within the iterator, from which it serves the actual `OsString`s.
//! The only aspect not done then is the attempt to convert to `String` if using the non-OS
//! iterator, which does occur per iteration. Thus the simple act of creating an instance of the
//! iterator is relatively expensive.
//!
//! As such you should avoid creating more than one where possible. For instance, if interested
//! in that special first program-name argument mentioned earlier, you should get it from the same
//! iterator as using for the to-be-parsed arguments, rather than creating a new iterator just for
//! retrieving it.
//!
//! For this reason the [`Args`] type gives you a method to retrieve the program-name argument.
//!
//! [`Args`]: struct.Args.html
//! [`as_slice`]: struct.Args.html#method.as_slice

use std::convert::AsRef;
use std::ffi::{OsStr, OsString};

/// Input arguments to be parsed
///
/// This is a wrapper for a set of real or fake input arguments for the parser to parse.
///
/// The reason this wrapper exists is to help plaster over some of the common pitfalls in directly
/// using the functionality for argument retrieval in the Rust `std` library, as discussed in the
/// [module level documentation](index.html).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args<'arg, A> where A: AsRef<OsStr> + 'arg {
    /// Arguments
    args: Inner<'arg, A>,
    /// Is there a program name in the set?
    have_prog_name: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Inner<'arg, A> where A: AsRef<OsStr> + 'arg {
    Vec(Vec<A>),
    Slice(&'arg [A]),
}

impl Args<'_, OsString> {
    /// Create a new instance from real input arguments
    ///
    /// This makes use of `std::env::args_os()` rather than `std::env::args()`, thus ensuring full
    /// compatibility with all possible OS filenames and paths that could be supplied as arguments
    /// (which are not always guaranteed to be fully valid Utf-8 on all platforms).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gong::arguments::Args;
    /// let args = Args::new();
    /// ```
    pub fn new() -> Self {
        Self {
            args: Inner::Vec(std::env::args_os().collect()),
            have_prog_name: true,
        }
    }

    /// Create a new dummy instance containing no actual arguments
    pub fn new_empty() -> Self {
        Self {
            args: Inner::Slice(&[]),
            have_prog_name: false,
        }
    }
}

impl<'arg, A> Args<'arg, A> where A: AsRef<OsStr> + 'arg {
    /// Create a new instance from an existing vector
    ///
    /// This takes ownership of the vector. If this is not okay, there is an alternate method which
    /// takes a slice instead.
    ///
    /// Note, unlike a real set of input arguments, which includes a program name as the first, this
    /// should **not** have such a first argument.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gong::arguments::Args;
    /// let fake_args = vec![ "--foo", "--bar" ];
    /// let args = Args::from_vec(fake_args);
    /// ```
    pub fn from_vec(args: Vec<A>) -> Self {
        Self {
            args: Inner::Vec(args),
            have_prog_name: false,
        }
    }

    /// Create a new instance from a slice
    ///
    /// Note, unlike a real set of input arguments, which includes a program name as the first, this
    /// should **not** have such a first argument.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gong::arguments::Args;
    /// let fake_args = [ "--foo", "--bar" ];
    /// let args = Args::from_slice(&fake_args[..]);
    /// ```
    pub fn from_slice(args: &'arg [A]) -> Self {
        Self {
            args: Inner::Slice(args),
            have_prog_name: false,
        }
    }

    /// Create a new instance from an existing iterator
    ///
    /// Note, unlike a real set of input arguments, which includes a program name as the first, this
    /// should **not** have such a first argument.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gong::arguments::Args;
    /// let fake_args = [ "--foo", "--bar" ];
    /// let args = Args::from_iter(fake_args[..].iter());
    /// ```
    pub fn from_iter(args: impl Iterator<Item = A>) -> Self {
        Self {
            args: Inner::Vec(args.collect()),
            have_prog_name: false,
        }
    }

    /// Get the program name argument
    ///
    /// Note the `Option` wrapper. While an instance of `Self` from real input arguments will always
    /// have a program name argument, an instance created from a custom argument source will not.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gong::arguments::Args;
    /// let args = Args::new();
    /// let prog_name_arg = args.get_prog_name().unwrap();
    /// ```
    pub fn get_prog_name(&self) -> Option<&OsStr> {
        match self.have_prog_name {
            false => None,
            true => match &self.args {
                Inner::Slice(s) => Some(s[0].as_ref()),
                Inner::Vec(v) => Some(v[0].as_ref()),
            }
        }
    }

    /// Get a slice of the arguments
    ///
    /// Note that the program name argument, if held, is skipped.
    ///
    /// # Example
    ///
    /// The following example gets a slice, creates an enumerated iterator from it, and prints out
    /// the list of arguments to standard output.
    ///
    /// ```rust
    /// # use gong::arguments::Args;
    /// let args = Args::new();
    /// for (index, arg) in args.as_slice().iter().enumerate() {
    ///     println!("[arg {}]: {:?}", index, arg);
    /// }
    /// ```
    pub fn as_slice(&self) -> &[A] {
        match &self.args {
            Inner::Slice(s) => s,
            Inner::Vec(v) if self.have_prog_name => &v[1..],
            Inner::Vec(v) => &v[..],
        }
    }
}
