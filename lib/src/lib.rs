// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
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

pub use engine::*;

/* -- Deprecated stuff -- */
/* Note, not possible to use a type for enum aliasing to mark deprecated, have to do without */

// Does marking the `pub use` for re-exporting enums to old location with deprecated work?
#[deprecated(note = "access from the `analysis` mod")]
pub use analysis::{ItemClass, Item, ItemE, ItemW, DataLocation};
#[deprecated(note = "access from the `options` mod")]
pub use options::{OptionsMode};

#[deprecated(note = "now called `analysis::Analysis`")]
pub type Results<'a> = analysis::Analysis<'a>;
#[deprecated(note = "moved to `options::Options`")]
pub type Options<'a> = options::Options<'a>;
#[deprecated(note = "moved to `options::LongOption`")]
pub type LongOption<'a> = options::LongOption<'a>;
#[deprecated(note = "moved to `options::ShortOption`")]
pub type ShortOption = options::ShortOption;
