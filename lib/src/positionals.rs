// Copyright 2019 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Positional argument related components

/// Used for expressing a quantity of positional arguments
pub type Quantity = u16;

/// The maximum quantity of positionals that can be expressed (`Quantity::max_value()`)
pub const MAX: Quantity = Quantity::max_value();

/// Default to use with parsers
#[doc(hidden)]
pub const DEFAULT_POLICY: Policy = Policy::Unlimited;

/// Positionals policy
///
/// Used to specifiy the policy regarding the number of positional arguments.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Policy {
    /// An unlimited number of positionals should be accepted.
    Unlimited,
    /// Only the specified number should be accepted, with others reported as unexpected.
    Max(Quantity),
}

impl Default for Policy {
    fn default() -> Self { DEFAULT_POLICY }
}

impl Policy {
    /// Given the number accepted so far, will the next one be unexpected?
    #[inline]
    pub fn is_next_unexpected(&self, accepted: Quantity) -> bool {
        match *self {
            Policy::Unlimited => false,
            Policy::Max(max) => accepted >= max,
        }
    }
}
