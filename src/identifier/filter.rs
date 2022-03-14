use crate::constants::{IdentifierFlags, EFF_MASK};

use super::{Id, StandardId};

/// Mask component of a filter.
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

/// An identifier filter.
///
/// ## Purpose
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
        let self_id = self.id.as_raw() & self.id.flags().bits();
        let other_id = id.as_raw() & id.flags().bits();
        other_id & self.mask.0 == self_id & self.mask.0
    }
}

impl Into<socketcan::CANFilter> for Filter {
    fn into(self) -> socketcan::CANFilter {
        socketcan::CANFilter::new(self.id.as_raw() & self.id.flags().bits(), self.mask.0).unwrap()
    }
}
