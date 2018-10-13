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
mod options;
mod processor;
mod results;

pub use options::*;
pub use processor::*;
pub use results::*;
