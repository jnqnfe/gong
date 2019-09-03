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
///
/// Internally the variants here boil down to a pair of minimum and maximum quantities (thus
/// unlimited is not actually truly unlimited). If too many positionals are encountered (more than
/// the maximum quantity) then the parser will start serving them as unexpected-positional problem
/// items instead of normal positional items. If too few are given (fewer than the minimum quantity)
/// then a missing-positionals problem item will be served as a final item.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Policy {
    /// An unlimited number of positionals should be accepted.
    ///
    /// Really this is treated the same as `MinMax(0, Quantity::max_value())` for simplicity, so not
    /// truly unlimited, but effectively it is since the max is a crazy number of input arguments.
    Unlimited,
    /// A specific number is required (same as `MinMax(n, n)`).
    Fixed(Quantity),
    /// A specific maximum number is allowed, with an unspecified minimum (same as `MinMax(0, n)`).
    ///
    /// Positionals encountered beyond this quantity will be reported as unexpected.
    Max(Quantity),
    /// A specific minimum number is required, with an unspecified maximum (same as `MinMax(n,
    /// Quantity::max_value())`).
    ///
    /// Fewer positionals encountered than this number will result in a missing-positionals problem
    /// item being served.
    Min(Quantity),
    /// A combination of both minimum and maximum (in that order).
    ///
    /// Specifying a minimum that is greater than the maximum is invalid.
    MinMax(Quantity, Quantity),
}

impl Default for Policy {
    fn default() -> Self { DEFAULT_POLICY }
}

/// Simplified positionals policy
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct SimplePolicy {
    pub min: Quantity,
    pub max: Quantity,
}

impl Policy {
    /// Get the max value, with `None` representing unlimited
    #[inline]
    pub fn get_max(&self) -> Option<Quantity> {
        match *self {
            Policy::Unlimited | Policy::Min(_) => None,
            Policy::Fixed(max) | Policy::Max(max) | Policy::MinMax(_, max) => Some(max),
        }
    }

    /// Get the minimum value
    #[inline]
    pub fn get_min(&self) -> Quantity {
        match *self {
            Policy::Unlimited | Policy::Max(_) => 0,
            Policy::Fixed(min) | Policy::Min(min) | Policy::MinMax(min, _) => min,
        }
    }

    /// Get the number remaining to be accepted, given a number so far accepted
    #[inline]
    pub fn get_remaining_min(&self, accepted: Quantity) -> Quantity {
        let min = self.get_min();
        match accepted >= min {
            true => 0,
            false => min - accepted,
        }
    }

    /// Get the limit on the number remaining that can be accepted, given a number so far accepted
    ///
    /// `None` means no limit, `Some(_)` means there is a limit, enclosing that limit.
    #[inline]
    pub fn get_remaining_allowed(&self, accepted: Quantity) -> Option<Quantity> {
        match self.get_max() {
            None => None,
            Some(max) => match accepted < max {
                false => Some(0),
                true => Some(max - accepted),
            },
        }
    }

    /// Given the number accepted so far, will the next one be unexpected?
    #[inline]
    pub fn is_next_unexpected(&self, accepted: Quantity) -> bool {
        match self.get_max() {
            None => false,
            Some(max) => accepted >= max,
        }
    }

    /// Check valid
    ///
    /// If setting a min+max policy, you could set min to be greater than max, which would not make
    /// sense and thus would be invalid. This is all this checks for.
    #[inline]
    pub fn is_valid(&self) -> bool {
        match *self {
            Policy::MinMax(min, max) => min <= max,
            _ => true,
        }
    }

    /// Internal helper
    #[inline]
    pub(crate) fn assert_valid(&self) {
        assert!(self.is_valid(), "Invalid positionals policy: {:?}", *self);
    }
}

impl From<Policy> for SimplePolicy {
    fn from(p: Policy) -> Self {
        let (min, max) = match p {
            Policy::Unlimited => (0, Quantity::max_value()),
            Policy::Fixed(i) => (i, i),
            Policy::Max(i) => (0, i),
            Policy::Min(i) => (i, Quantity::max_value()),
            Policy::MinMax(a, b) => (a, b),
        };
        SimplePolicy::new(min, max)
    }
}

impl SimplePolicy {
    /// Create new
    #[inline(always)]
    pub const fn new(min: Quantity, max: Quantity) -> Self {
        Self { min, max }
    }

    /// Get the number remaining to be accepted, given a number so far accepted
    #[inline]
    pub fn get_remaining_min(&self, accepted: Quantity) -> Quantity {
        match accepted >= self.min {
            true => 0,
            false => self.min - accepted,
        }
    }

    /// Given the number accepted so far, will the next one be unexpected?
    #[inline]
    pub const fn is_next_unexpected(&self, accepted: Quantity) -> bool {
        accepted >= self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Since there is a difference with respect to max in `SimplePolicy` (no option wrapper), they
    /// are not designed to convert to simple form before offering up a value, so we certainly
    /// have a need to know that they offer up the right values.
    #[test]
    fn policy_min_max() {
        let tests = [
            // (<policy>, <get-max>, <get-min>)
            (Policy::Unlimited,    None,    0),
            (Policy::Fixed(0),     Some(0), 0),
            (Policy::Fixed(1),     Some(1), 1),
            (Policy::Min(0),       None,    0),
            (Policy::Min(1),       None,    1),
            (Policy::Max(0),       Some(0), 0),
            (Policy::Max(1),       Some(1), 0),
            (Policy::MinMax(0, 0), Some(0), 0),
            (Policy::MinMax(1, 1), Some(1), 1),
            (Policy::MinMax(1, 0), Some(0), 1),
            (Policy::MinMax(0, 1), Some(1), 0),
            (Policy::MinMax(2, 3), Some(3), 2),
            (Policy::MinMax(3, 2), Some(2), 3),
        ];
        for test in &tests {
            eprintln!("testing ({:?}, {:?}, {})", test.0, test.1, test.2);
            assert_eq!(test.1, test.0.get_max());
            assert_eq!(test.2, test.0.get_min());
        }
    }

    #[test]
    fn policy() {
        let policy = Policy::Unlimited;
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(0, policy.get_remaining_min(MAX));
        assert_eq!(None, policy.get_remaining_allowed(0));
        assert_eq!(None, policy.get_remaining_allowed(1));
        assert_eq!(None, policy.get_remaining_allowed(MAX));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX));

        let policy = Policy::Fixed(0);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(Some(0), policy.get_remaining_allowed(0));
        assert_eq!(Some(0), policy.get_remaining_allowed(1));
        assert_eq!(Some(0), policy.get_remaining_allowed(MAX));
        assert_eq!(true, policy.is_next_unexpected(0));
        assert_eq!(true, policy.is_next_unexpected(1));

        let policy = Policy::Fixed(2);
        assert_eq!(2, policy.get_remaining_min(0));
        assert_eq!(1, policy.get_remaining_min(1));
        assert_eq!(0, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(Some(2), policy.get_remaining_allowed(0));
        assert_eq!(Some(1), policy.get_remaining_allowed(1));
        assert_eq!(Some(0), policy.get_remaining_allowed(2));
        assert_eq!(Some(0), policy.get_remaining_allowed(3));
        assert_eq!(Some(0), policy.get_remaining_allowed(MAX));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(true,  policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));

        let policy = Policy::Fixed(MAX);
        assert_eq!(MAX,     policy.get_remaining_min(0));
        assert_eq!(MAX - 1, policy.get_remaining_min(1));
        assert_eq!(MAX - 2, policy.get_remaining_min(2));
        assert_eq!(1,       policy.get_remaining_min(MAX - 1));
        assert_eq!(0,       policy.get_remaining_min(MAX    ));
        assert_eq!(Some(MAX    ), policy.get_remaining_allowed(0));
        assert_eq!(Some(MAX - 1), policy.get_remaining_allowed(1));
        assert_eq!(Some(0),       policy.get_remaining_allowed(MAX));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX - 1));
        assert_eq!(true,  policy.is_next_unexpected(MAX    ));

        let policy = Policy::Min(0);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(None, policy.get_remaining_allowed(0));
        assert_eq!(None, policy.get_remaining_allowed(1));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX));

        let policy = Policy::Min(2);
        assert_eq!(2, policy.get_remaining_min(0));
        assert_eq!(1, policy.get_remaining_min(1));
        assert_eq!(0, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(None, policy.get_remaining_allowed(0));
        assert_eq!(None, policy.get_remaining_allowed(1));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX));

        let policy = Policy::Min(MAX);
        assert_eq!(MAX,     policy.get_remaining_min(0));
        assert_eq!(MAX - 1, policy.get_remaining_min(1));
        assert_eq!(MAX - 2, policy.get_remaining_min(2));
        assert_eq!(1,       policy.get_remaining_min(MAX - 1));
        assert_eq!(0,       policy.get_remaining_min(MAX    ));
        assert_eq!(None, policy.get_remaining_allowed(0));
        assert_eq!(None, policy.get_remaining_allowed(1));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX));

        let policy = Policy::Max(0);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(Some(0), policy.get_remaining_allowed(0));
        assert_eq!(Some(0), policy.get_remaining_allowed(1));
        assert_eq!(true, policy.is_next_unexpected(0));
        assert_eq!(true, policy.is_next_unexpected(1));

        let policy = Policy::Max(2);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(Some(2), policy.get_remaining_allowed(0));
        assert_eq!(Some(1), policy.get_remaining_allowed(1));
        assert_eq!(Some(0), policy.get_remaining_allowed(2));
        assert_eq!(Some(0), policy.get_remaining_allowed(3));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(true,  policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));

        let policy = Policy::Max(MAX);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(Some(MAX    ), policy.get_remaining_allowed(0));
        assert_eq!(Some(MAX - 1), policy.get_remaining_allowed(1));
        assert_eq!(Some(1),       policy.get_remaining_allowed(MAX - 1));
        assert_eq!(Some(0),       policy.get_remaining_allowed(MAX    ));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX - 1));
        assert_eq!(true,  policy.is_next_unexpected(MAX    ));

        let policy = Policy::MinMax(0, 0);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(Some(0), policy.get_remaining_allowed(0));
        assert_eq!(Some(0), policy.get_remaining_allowed(1));
        assert_eq!(true, policy.is_next_unexpected(0));
        assert_eq!(true, policy.is_next_unexpected(1));

        let policy = Policy::MinMax(2, 0);
        assert_eq!(2, policy.get_remaining_min(0));
        assert_eq!(1, policy.get_remaining_min(1));
        assert_eq!(0, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(Some(0), policy.get_remaining_allowed(0));
        assert_eq!(Some(0), policy.get_remaining_allowed(1));
        assert_eq!(true, policy.is_next_unexpected(0));
        assert_eq!(true, policy.is_next_unexpected(1));

        let policy = Policy::MinMax(MAX, 0);
        assert_eq!(MAX,     policy.get_remaining_min(0));
        assert_eq!(MAX - 1, policy.get_remaining_min(1));
        assert_eq!(MAX - 2, policy.get_remaining_min(2));
        assert_eq!(1,       policy.get_remaining_min(MAX - 1));
        assert_eq!(0,       policy.get_remaining_min(MAX    ));
        assert_eq!(Some(0), policy.get_remaining_allowed(0));
        assert_eq!(Some(0), policy.get_remaining_allowed(1));
        assert_eq!(true, policy.is_next_unexpected(0));
        assert_eq!(true, policy.is_next_unexpected(1));

        let policy = Policy::MinMax(0, 2);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(Some(2), policy.get_remaining_allowed(0));
        assert_eq!(Some(1), policy.get_remaining_allowed(1));
        assert_eq!(Some(0), policy.get_remaining_allowed(2));
        assert_eq!(Some(0), policy.get_remaining_allowed(3));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(true,  policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));

        let policy = Policy::MinMax(0, MAX);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(Some(MAX    ), policy.get_remaining_allowed(0));
        assert_eq!(Some(MAX - 1), policy.get_remaining_allowed(1));
        assert_eq!(Some(1),       policy.get_remaining_allowed(MAX - 1));
        assert_eq!(Some(0),       policy.get_remaining_allowed(MAX    ));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX - 1));
        assert_eq!(true,  policy.is_next_unexpected(MAX    ));

        let policy = Policy::MinMax(2, 3);
        assert_eq!(2, policy.get_remaining_min(0));
        assert_eq!(1, policy.get_remaining_min(1));
        assert_eq!(0, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(Some(3), policy.get_remaining_allowed(0));
        assert_eq!(Some(2), policy.get_remaining_allowed(1));
        assert_eq!(Some(1), policy.get_remaining_allowed(2));
        assert_eq!(Some(0), policy.get_remaining_allowed(3));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));

        let policy = Policy::MinMax(2, MAX);
        assert_eq!(2, policy.get_remaining_min(0));
        assert_eq!(1, policy.get_remaining_min(1));
        assert_eq!(0, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(Some(MAX    ), policy.get_remaining_allowed(0));
        assert_eq!(Some(MAX - 1), policy.get_remaining_allowed(1));
        assert_eq!(Some(1),       policy.get_remaining_allowed(MAX - 1));
        assert_eq!(Some(0),       policy.get_remaining_allowed(MAX    ));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX - 1));
        assert_eq!(true,  policy.is_next_unexpected(MAX    ));

        let policy = Policy::MinMax(MAX, 2);
        assert_eq!(MAX,     policy.get_remaining_min(0));
        assert_eq!(MAX - 1, policy.get_remaining_min(1));
        assert_eq!(MAX - 2, policy.get_remaining_min(2));
        assert_eq!(1,       policy.get_remaining_min(MAX - 1));
        assert_eq!(0,       policy.get_remaining_min(MAX    ));
        assert_eq!(Some(2), policy.get_remaining_allowed(0));
        assert_eq!(Some(1), policy.get_remaining_allowed(1));
        assert_eq!(Some(0), policy.get_remaining_allowed(2));
        assert_eq!(Some(0), policy.get_remaining_allowed(3));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(true,  policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));

        let policy = Policy::MinMax(MAX, MAX);
        assert_eq!(MAX,     policy.get_remaining_min(0));
        assert_eq!(MAX - 1, policy.get_remaining_min(1));
        assert_eq!(MAX - 2, policy.get_remaining_min(2));
        assert_eq!(1,       policy.get_remaining_min(MAX - 1));
        assert_eq!(0,       policy.get_remaining_min(MAX    ));
        assert_eq!(Some(MAX    ), policy.get_remaining_allowed(0));
        assert_eq!(Some(MAX - 1), policy.get_remaining_allowed(1));
        assert_eq!(Some(1),       policy.get_remaining_allowed(MAX - 1));
        assert_eq!(Some(0),       policy.get_remaining_allowed(MAX    ));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX - 1));
        assert_eq!(true,  policy.is_next_unexpected(MAX    ));

        let policy = Policy::MinMax(3, 2); // Invalid
        assert_eq!(3, policy.get_remaining_min(0));
        assert_eq!(2, policy.get_remaining_min(1));
        assert_eq!(1, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(Some(2), policy.get_remaining_allowed(0));
        assert_eq!(Some(1), policy.get_remaining_allowed(1));
        assert_eq!(Some(0), policy.get_remaining_allowed(2));
        assert_eq!(Some(0), policy.get_remaining_allowed(3));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(true,  policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));
    }

    #[test]
    fn simplified_policy() {
        let policy = SimplePolicy::new(0, 0);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(true, policy.is_next_unexpected(0));
        assert_eq!(true, policy.is_next_unexpected(1));

        let policy = SimplePolicy::new(2, 0);
        assert_eq!(2, policy.get_remaining_min(0));
        assert_eq!(1, policy.get_remaining_min(1));
        assert_eq!(0, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(true, policy.is_next_unexpected(0));
        assert_eq!(true, policy.is_next_unexpected(1));

        let policy = SimplePolicy::new(MAX, 0);
        assert_eq!(MAX,     policy.get_remaining_min(0));
        assert_eq!(MAX - 1, policy.get_remaining_min(1));
        assert_eq!(MAX - 2, policy.get_remaining_min(2));
        assert_eq!(1,       policy.get_remaining_min(MAX - 1));
        assert_eq!(0,       policy.get_remaining_min(MAX    ));
        assert_eq!(true, policy.is_next_unexpected(0));
        assert_eq!(true, policy.is_next_unexpected(1));

        let policy = SimplePolicy::new(0, 2);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(true,  policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));

        let policy = SimplePolicy::new(0, MAX);
        assert_eq!(0, policy.get_remaining_min(0));
        assert_eq!(0, policy.get_remaining_min(1));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX - 1));
        assert_eq!(true,  policy.is_next_unexpected(MAX    ));

        let policy = SimplePolicy::new(2, 3);
        assert_eq!(2, policy.get_remaining_min(0));
        assert_eq!(1, policy.get_remaining_min(1));
        assert_eq!(0, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));

        let policy = SimplePolicy::new(2, MAX);
        assert_eq!(2, policy.get_remaining_min(0));
        assert_eq!(1, policy.get_remaining_min(1));
        assert_eq!(0, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX - 1));
        assert_eq!(true,  policy.is_next_unexpected(MAX    ));

        let policy = SimplePolicy::new(MAX, 2);
        assert_eq!(MAX,     policy.get_remaining_min(0));
        assert_eq!(MAX - 1, policy.get_remaining_min(1));
        assert_eq!(MAX - 2, policy.get_remaining_min(2));
        assert_eq!(1,       policy.get_remaining_min(MAX - 1));
        assert_eq!(0,       policy.get_remaining_min(MAX    ));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(true,  policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));

        let policy = SimplePolicy::new(MAX, MAX);
        assert_eq!(MAX,     policy.get_remaining_min(0));
        assert_eq!(MAX - 1, policy.get_remaining_min(1));
        assert_eq!(MAX - 2, policy.get_remaining_min(2));
        assert_eq!(1,       policy.get_remaining_min(MAX - 1));
        assert_eq!(0,       policy.get_remaining_min(MAX    ));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(false, policy.is_next_unexpected(MAX - 1));
        assert_eq!(true,  policy.is_next_unexpected(MAX    ));

        let policy = SimplePolicy::new(3, 2); // Invalid
        assert_eq!(3, policy.get_remaining_min(0));
        assert_eq!(2, policy.get_remaining_min(1));
        assert_eq!(1, policy.get_remaining_min(2));
        assert_eq!(0, policy.get_remaining_min(3));
        assert_eq!(false, policy.is_next_unexpected(0));
        assert_eq!(false, policy.is_next_unexpected(1));
        assert_eq!(true,  policy.is_next_unexpected(2));
        assert_eq!(true,  policy.is_next_unexpected(3));
    }

    #[test]
    fn conversion() {
        assert_eq!(SimplePolicy { min: 1, max: 2 }, SimplePolicy::new(1, 2));

        assert_eq!(SimplePolicy::new(0, MAX), Policy::Unlimited.into());
        assert_eq!(SimplePolicy::new(0, 0),   Policy::Fixed(0).into());
        assert_eq!(SimplePolicy::new(2, 2),   Policy::Fixed(2).into());
        assert_eq!(SimplePolicy::new(0, 0),   Policy::Max(0).into());
        assert_eq!(SimplePolicy::new(0, 2),   Policy::Max(2).into());
        assert_eq!(SimplePolicy::new(0, MAX), Policy::Min(0).into());
        assert_eq!(SimplePolicy::new(2, MAX), Policy::Min(2).into());
        assert_eq!(SimplePolicy::new(0, 0),   Policy::MinMax(0, 0).into());
        assert_eq!(SimplePolicy::new(0, 2),   Policy::MinMax(0, 2).into());
        assert_eq!(SimplePolicy::new(2, 0),   Policy::MinMax(2, 0).into());
        assert_eq!(SimplePolicy::new(3, 6),   Policy::MinMax(3, 6).into());
        assert_eq!(SimplePolicy::new(6, 3),   Policy::MinMax(6, 3).into()); //Invalid of course
    }

    #[test]
    fn is_valid() {
        assert_eq!(true,  Policy::Unlimited.is_valid());
        assert_eq!(true,  Policy::Fixed(0).is_valid());
        assert_eq!(true,  Policy::Fixed(2).is_valid());
        assert_eq!(true,  Policy::Fixed(Quantity::max_value()).is_valid());
        assert_eq!(true,  Policy::Max(0).is_valid());
        assert_eq!(true,  Policy::Max(2).is_valid());
        assert_eq!(true,  Policy::Max(Quantity::max_value()).is_valid());
        assert_eq!(true,  Policy::Min(0).is_valid());
        assert_eq!(true,  Policy::Min(2).is_valid());
        assert_eq!(true,  Policy::Min(Quantity::max_value()).is_valid());
        assert_eq!(true,  Policy::MinMax(0, 0).is_valid());
        assert_eq!(true,  Policy::MinMax(0, 2).is_valid());
        assert_eq!(true,  Policy::MinMax(0, Quantity::max_value()).is_valid());
        assert_eq!(true,  Policy::MinMax(3, 6).is_valid());
        assert_eq!(true,  Policy::MinMax(3, Quantity::max_value()).is_valid());
        assert_eq!(true,  Policy::MinMax(Quantity::max_value(), Quantity::max_value()).is_valid());
        assert_eq!(false, Policy::MinMax(2, 0).is_valid());
        assert_eq!(false, Policy::MinMax(6, 3).is_valid());
        assert_eq!(false, Policy::MinMax(Quantity::max_value(), 0).is_valid());
        assert_eq!(false, Policy::MinMax(Quantity::max_value(), 2).is_valid());
    }
}
