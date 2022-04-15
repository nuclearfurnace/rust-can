use std::ops::{Add, BitAnd, BitOr, BitXor, Sub};

use crate::constants::{IdentifierFlags, EFF_MASK};

use super::{ExtendedId, Id, StandardId};

/// Mask component of a filter.
#[derive(Debug)]
pub struct Mask(u32);

impl Mask {
    /// Matches all bits in the identifier.
    pub const ALL: Mask = Self(0xFFFFFFFF);

    /// Matches no bits in the identifier.
    pub const NONE: Mask = Self(0);

    /// Creates a new [`Mask`].
    pub const fn new(mask: u32) -> Mask {
        Self(mask)
    }
}

impl BitAnd for Mask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Mask(self.0 & rhs.0)
    }
}

impl BitOr for Mask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Mask(self.0 | rhs.0)
    }
}

impl BitXor for Mask {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Mask(self.0 ^ rhs.0)
    }
}

impl Add for Mask {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Mask(self.0.wrapping_add(rhs.0))
    }
}

impl Sub for Mask {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Mask(self.0.wrapping_sub(rhs.0))
    }
}

/// An identifier filter.
///
/// Often times when communicating on a bus with many devices, it can be useful to filter which
/// messages are captured and which are ignored.  This can dramatically speed up processing times
/// and reduce the overhead of having to receive and then discard every single message transmitted
/// on the bus.
///
/// A [`Filter`] takes an identifier and a mask, which when used together and applied to an incoming
/// message's identifier, is used to decide if that message should be discarded or not, using the
/// following logic:
///
/// <message identifier> & mask == <filter identifier> & mask
///
/// ## Usage
///
/// For example, the caller maybe interested in responses from a single standard identifier, 0x246.
/// We'll assume that they don't care whether it's a standard identifier or extended.  Internally, a
/// filter for this would have an identifier value of 0x246 and a mask of 0x1FFFFFFF. Let's apply
/// this filter to a few theoretical message identifiers:
///
/// - 0x132 & 0x1FFFFFFF (0x132) does not match 0x246 & 0x1FFFFFFF (0x246), so we discard this
///   message
/// - 0x80000246 & 0x1FFFFFFF (0x246) does match 0x246 & 0x1FFFFFFF (0x246), so we keep the message
///
/// We can tweak this example further, as well.  Maybe we know that we only specifically care about
/// the standard identifier 0x246, and not the extended version of it.  If we change our mask to
/// 0xFFFFFFFF, and recheck the previous message, we get the following result:
///
/// - 0x80000246 & 0xFFFFFFFF (0x80000246) does not match 0x246 & 0xFFFFFFFF (0x246), so we discard
///   the message
///
/// As another example, we may want to sometimes only pay attention to a few specific bytes in the
/// identifier.  This is common when filtering certain ranges of addresses, like those used for
/// legislated OBD devices, as they're contiguously and we can capture multiple identifiers with one
/// filter, like so:
///
/// - we want to filter addresses 0x7E8 to 0x7EF (0b11111101000 to 0b11111101111, respectively)
/// - thus, we know that everything between them, inclusive, is something we care about
/// - we take the "none" mask -- 0xFFFFFFFF -- as our base mask, to ensure we don't accidentally
///   match extended identifiers
/// - we take the difference between the higher address and lower address (0x7EF - 0x7E8 == 0x7) and
///   subtract it from our base mask, which gives us a mask of 0xFFFFFFFF - 0x7, or 0xFFFFFFF8
/// - we use the lower address as our identifier
///
/// Let's apply this mask to some more theoretical message identifiers:
///
/// - 0x7E8 & 0xFFFFFFF8 (0x7E8) does match 0x7E8 & 0xFFFFFFF8 (0x7E8), so we keep this message
/// - 0x7EF & 0xFFFFFFF8 (0x7E8) does match 0x7E8 & 0xFFFFFFF8 (0x7E8), so we keep the message
/// - 0x7F0 & 0xFFFFFFF8 (0x7F0) does not match 0x7E8 & 0xFFFFFFF8 (0x7E0), so we discard this
///   message
/// - 0x800007E8 & 0xFFFFFFF8 (0x800007E8) does not match 0x7E8 & 0xFFFFFFF8 (0x7E8), so we discard
///   the message
///
/// ## Caveats
///
/// Internally, [`Filter`] uses a format that maps to the identifier format used by
/// [SocketCAN][socketcan], where both the identifier (the logical address itself) and the
/// identifier flags (error frame, remote frame, etc) are encoded into a single 32-bit unsigned
/// integer.
///
/// While the identifier type ([`Id`]) encodes these flags directly, [`Mask`] allows more direct
/// control.  Despite this, it is often best to utilize the flags-based helper methods for defining
/// masks.  These make it easier to construct filters based on functional need: match a single
/// identifier, match error frames only, etc.
///
/// [socketcan]: https://www.kernel.org/doc/Documentation/networking/can.txt
#[derive(Debug)]
pub struct Filter {
    id: Id,
    mask: Mask,
}

impl Filter {
    /// Creates a [`Filter`] based on the given identifier and mask.
    pub const fn new(id: Id, mask: Mask) -> Self {
        Self { id, mask }
    }

    /// Creates a [`Filter`] that will only match the given [`Id`].
    ///
    /// This only allows matching the identifier in its specific addressing mode.  In other words,
    /// if the identifier is 0x123 in standard addressing mode, an identifier of 0x123 in _extended_
    /// addressing mode will _not_ match.
    pub const fn from_identity(id: Id) -> Self {
        Self {
            id,
            mask: Mask(EFF_MASK | id.flags().bits()),
        }
    }

    /// Creates a [`Filter`] that will match any identifier between `start` and `end`, inclusive.
    pub const fn range(start: Id, end: Id) -> Self {
        let (id, delta_mask) = if start.as_raw() > end.as_raw() {
            (end, start.as_raw() - end.as_raw())
        } else {
            (start, end.as_raw() - start.as_raw())
        };

        Self {
            id,
            mask: Mask(Mask::ALL.0 - delta_mask),
        }
    }

    /// Creates a [`Filter`] that matches no identifiers.
    pub const fn none() -> Self {
        // Abuse the fact that, in practice, a CAN frame can/should never be a data frame, error
        // frame, and remote frame simultaneously. Callers could _technically_ construct an
        // identifier and do what we're doing here, but I'm comfortable saying: you brought this on
        // yourself. :P
        Self {
            id: Id::Extended(ExtendedId::MAX).set_flags(IdentifierFlags::all()),
            mask: Mask::ALL,
        }
    }

    /// Creates a [`Filter`] that matches any identifiers.
    pub const fn any() -> Self {
        Self {
            id: Id::Standard(StandardId::ZERO),
            mask: Mask(0),
        }
    }

    /// Creates a [`Filter`] that matches only data frames.
    pub const fn data_frames_only() -> Self {
        Self {
            id: Id::Standard(StandardId::ZERO),
            mask: Mask(IdentifierFlags::ERROR.bits()),
        }
    }

    /// Creates a [`Filter`] that matches only error frames.
    pub const fn error_frames_only() -> Self {
        Self {
            id: Id::Standard(StandardId::ZERO.set_flags(IdentifierFlags::ERROR)),
            mask: Mask(IdentifierFlags::ERROR.bits()),
        }
    }

    /// Updates this [`Filter`] to allow matching extended frames.
    pub const fn allow_extended_frames(self) -> Self {
        Self {
            id: self.id,
            mask: Mask(self.mask.0 | IdentifierFlags::EXTENDED.bits()),
        }
    }

    /// Updates this [`Filter`] to disallow matching extended frames.
    pub const fn disallow_extended_frames(self) -> Self {
        Self {
            id: self.id,
            mask: Mask(self.mask.0 & !IdentifierFlags::EXTENDED.bits()),
        }
    }

    /// Updates this [`Filter`] to allow matching remote frames.
    pub const fn allow_rtr_frames(self) -> Self {
        Self {
            id: self.id,
            mask: Mask(self.mask.0 | IdentifierFlags::REMOTE.bits()),
        }
    }

    /// Updates this [`Filter`] to disallow matching remote frames.
    pub const fn disallow_rtr_frames(self) -> Self {
        Self {
            id: self.id,
            mask: Mask(self.mask.0 & !IdentifierFlags::REMOTE.bits()),
        }
    }

    /// Updates this [`Filter`] to allow matching error frames.
    pub const fn allow_error_frames(self) -> Self {
        Self {
            id: self.id,
            mask: Mask(self.mask.0 | IdentifierFlags::ERROR.bits()),
        }
    }

    /// Updates this [`Filter`] to disallow matching error frames.
    pub const fn disallow_error_frames(self) -> Self {
        Self {
            id: self.id,
            mask: Mask(self.mask.0 & !IdentifierFlags::ERROR.bits()),
        }
    }

    /// Checks if the given identifier matches the filter.
    pub const fn matches(&self, id: Id) -> bool {
        let self_id = self.id.as_raw() | self.id.flags().bits();
        let other_id = id.as_raw() | id.flags().bits();

        other_id & self.mask.0 == self_id & self.mask.0
    }
}

#[cfg(feature = "socketcan-compat")]
#[cfg_attr(docsrs, doc(cfg(feature = "socketcan-compat")))]
impl Into<socketcan::CANFilter> for Filter {
    fn into(self) -> socketcan::CANFilter {
        socketcan::CANFilter::new(self.id.as_raw() & self.id.flags().bits(), self.mask.0).unwrap()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::identifier::{id::tests::arb_id, StandardId};

    use super::Filter;

    use proptest::{collection::vec as arb_vec, proptest};

    proptest! {
        #[test]
        fn none(ids in arb_vec(arb_id(), 100..1000)) {
            let filter = Filter::none();
            for id in ids {
                assert!(!filter.matches(id));
            }
        }

        #[test]
        fn any(ids in arb_vec(arb_id(), 100..1000)) {
            let filter = Filter::any();
            for id in ids {
                assert!(filter.matches(id));
            }
        }
    }

    #[test]
    fn range() {
        let start = StandardId::new(0x7E0).unwrap();
        let end = StandardId::new(0x7EF).unwrap();

        run_range(start, end);
    }

    #[test]
    fn range_reversed() {
        let start = StandardId::new(0x7E0).unwrap();
        let end = StandardId::new(0x7EF).unwrap();

        run_range(end, start);
    }

    fn run_range(start: StandardId, end: StandardId) {
        // Figure out the ranges of identifiers to test that should be outside of the range, as well
        // as inside the range.  We have to make sure we figure out if start/end are in the right
        // order since being able to handle out-of-order start/end arguments is part of
        // `Filter::range` so that we can avoid having to make it fallible when the "fix" is just
        // internally flipping the operands ourselves.
        let zero = StandardId::ZERO.as_raw();
        let max = StandardId::MAX.as_raw();
        let (filter_raw_start, filter_raw_end) = if start > end {
            // Start/end are out-of-order, so our "before" range needs to be ZERO <-> end and the
            // "after" range needs to be start <-> MAX.
            (end.as_raw(), start.as_raw())
        } else {
            // Things are already in the right order.
            (start.as_raw(), end.as_raw())
        };

        let before_range = zero..filter_raw_start;
        let after_range = (filter_raw_end + 1)..max;
        let match_range = filter_raw_start..=filter_raw_end;

        let filter = Filter::range(start.into(), end.into());

        // Make sure the identifiers outside of the range do not match.
        for i in before_range {
            let id = StandardId::new(i).unwrap();
            assert!(!filter.matches(id.into()));
        }

        for i in after_range {
            let id = StandardId::new(i).unwrap();
            assert!(!filter.matches(id.into()));
        }

        // And make sure the filter actually does match identifiers within the range, inclusive.
        for i in match_range {
            let id = StandardId::new(i).unwrap();
            assert!(filter.matches(id.into()));
        }
    }
}
