// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
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
//! - [Command argument support](docs/commands/index.html)
//! - [Usage](docs/usage/index.html)

#![doc(html_logo_url = "https://github.com/jnqnfe/gong/raw/master/logo.png",
       html_favicon_url = "https://github.com/jnqnfe/gong/raw/master/favicon.ico")]

#[cfg(feature = "suggestions")]
extern crate strsim;

pub mod docs;
#[macro_use]
mod macros; //Note: If we use these in the lib (e.g. internal tests) then this mod must come first!
pub mod analysis;
pub mod commands;
mod engine;
mod engine_os;
pub mod options;
pub mod parser;
