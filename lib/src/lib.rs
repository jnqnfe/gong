// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! A lightweight, flexible and simple-to-use library provided to *assist* in processing command
//! line arguments.
//!
//! Licensed under the MIT license or the Apache license, Version 2.0, at your option.
//!
//! # Documentation
//!
//! Documentation has been split up into chapters:
//!
//! - [Overview](docs/overview/index.html)
//! - [Options support](docs/options/index.html)
//! - [Usage](docs/usage/index.html)

#![doc(html_logo_url = "https://github.com/jnqnfe/gong/raw/master/logo.png",
       html_favicon_url = "https://github.com/jnqnfe/gong/raw/master/favicon.ico")]

pub mod docs;
#[macro_use]
mod macros; //Note: If we use these in the lib (e.g. internal tests) then this mod must come first!
pub mod analysis;
mod engine;
pub mod options;

/* -- Deprecated stuff -- */
/* Note, not possible to use a type for enum aliasing to mark deprecated, have to do without */

// Does marking the `pub use` for re-exporting enums to old location with deprecated work?
#[deprecated(since = "1.1.0", note = "access from the `analysis` mod")]
pub use analysis::{ItemClass, Item, ItemE, ItemW, DataLocation};
#[deprecated(since = "1.1.0", note = "access from the `options` mod")]
pub use options::{OptionsMode};

#[deprecated(since = "1.1.0", note = "now called `analysis::Analysis`")]
pub type Results<'a> = analysis::Analysis<'a>;
#[deprecated(since = "1.1.0", note = "moved to `options::OptionSetEx`")]
pub type Options<'a> = options::OptionSetEx<'a>;
#[deprecated(since = "1.1.0", note = "moved to `options::LongOption`")]
pub type LongOption<'a> = options::LongOption<'a>;
#[deprecated(since = "1.1.0", note = "moved to `options::ShortOption`")]
pub type ShortOption = options::ShortOption;

/// Analyses provided program arguments, using provided information about valid available options.
///
/// Returns a result set describing the result of the analysis. This may include `&str` references
/// to strings provided in the `args` and `options` parameter data. Take note of this with respect
/// to object lifetimes.
///
/// Expects available `options` data to have already been validated. (See
/// [`OptionSetEx::is_valid`](options/struct.OptionSetEx.html#method.is_valid)).
#[deprecated(since = "1.2.0", note = "use the method on the option set object instead")]
pub fn process<'o, 'a, A>(args: &'a [A], options: &'o options::OptionSetEx<'a>) -> analysis::Analysis<'a>
    where A: 'a + std::convert::AsRef<str>,
          'a: 'o
{
    engine::process(args, &options.as_fixed())
}
