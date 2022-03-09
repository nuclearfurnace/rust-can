//! CAN Identifiers.

use std::{cmp, fmt};

/// Standard (11-bit) CAN identifier.
///
/// Commonly referred to as CAN 2.0A, a standard identifier falls within the range of 0 to 0x7FF, inclusive.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct StandardId(u16);

impl StandardId {
    /// Minimum value for a standard identifier.
    ///
    /// This is the highest priority standard identifier.
    pub const ZERO: Self = Self(0);

    /// Maximum value for a standard identifier.
    ///
    /// This is the lowest priority standard identifier.
    pub const MAX: Self = Self(0x7FF);

    /// Creates a `StandardId`.
    ///
    /// Returns `None` if `identifier` is greater than [`MAX`][Self::MAX].
    #[inline]
    pub const fn new(identifier: u16) -> Option<Self> {
        if identifier <= Self::MAX.as_raw() {
            Some(Self(identifier))
        } else {
            None
        }
    }

    /// Returns the identifier as a raw integer.
    #[inline]
    pub const fn as_raw(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for StandardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

/// Extended (29-bit) CAN identifier.
///
/// Commonly referred to as CAN 2.0B, an extended identifier falls within the range of 0 to
/// 0x1FFFFFFF, inclusive.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct ExtendedId(u32);

impl ExtendedId {
    /// Minimum value for an extended identifier.
    ///
    /// This is the highest priority extended identifier.
    pub const ZERO: Self = Self(0);

    /// Maximum value for ban extended identifier.
    ///
    /// This is the lowest priority extended identifier.
    pub const MAX: Self = Self(0x1FFF_FFFF);

    /// Creates an `ExtendedId`.
    ///
    /// Returns `None` if `identifier` is greater than [`MAX`][Self::MAX].
    #[inline]
    pub const fn new(identifier: u32) -> Option<Self> {
        if identifier <= Self::MAX.as_raw() {
            Some(Self(identifier))
        } else {
            None
        }
    }

    /// Returns the identifier as a raw integer.
    #[inline]
    pub const fn as_raw(&self) -> u32 {
        self.0
    }

    /// Returns the base (standard) portion of this extended identifier.
    pub const fn as_standard_id(&self) -> StandardId {
        StandardId((self.0 >> 18) as u16)
    }
}

impl fmt::Display for ExtendedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

/// A CAN identifier (standard or extended).
///
/// The identifier serves both as a logical key, or address, for a CAN message, where a message with
/// a given identifier might be destined to be received by a specific node on the bus, or may
/// represent a broadcast address that interested nodes can watch for.
///
/// Additionally, the identifier is used in the arbitration process, when multiple bus nodes are
/// attempting to transmit simultaneously and one must be chosen to decide who is allowed to send,
/// and who must wait to retry their message.  Identifiers that are lower in value have higher
/// priority on the bus, and vise versa.
///
/// ## Priority and sorting
///
/// In following with the CAN specification, a `StandardId` is always a higher priority than an
/// `ExtendedId` as the "Identifier Extension (IDE)" bit will be recessive (1) in the case of an
/// extended identifier, and so the sorting behavior for `StandardId`, `ExtendedId`, and `Id` all
/// reflect this.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum Id {
    /// Standard (11-bit) CAN identifier.
    Standard(StandardId),

    /// Extended (29-bit) CAN identifier.
    Extended(ExtendedId),
}

impl Id {
    pub const fn as_raw(&self) -> u32 {
        match self {
            Self::Standard(sid) => sid.as_raw() as u32,
            Self::Extended(eid) => eid.as_raw(),
        }
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Id::Standard(s1), Id::Standard(s2)) => s1.partial_cmp(s2),
            (Id::Standard(_), Id::Extended(_)) => Some(cmp::Ordering::Less),
            (Id::Extended(_), Id::Standard(_)) => Some(cmp::Ordering::Greater),
            (Id::Extended(e1), Id::Extended(e2)) => e1.partial_cmp(e2),
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard(sid) => sid.fmt(f),
            Self::Extended(eid) => eid.fmt(f),
        }
    }
}

impl From<StandardId> for Id {
    #[inline]
    fn from(id: StandardId) -> Self {
        Id::Standard(id)
    }
}

impl From<ExtendedId> for Id {
    #[inline]
    fn from(id: ExtendedId) -> Self {
        Id::Extended(id)
    }
}

impl Into<embedded_can::StandardId> for StandardId {
    fn into(self) -> embedded_can::StandardId {
        unsafe { embedded_can::StandardId::new_unchecked(self.0) }
    }
}

impl Into<embedded_can::ExtendedId> for ExtendedId {
    fn into(self) -> embedded_can::ExtendedId {
        unsafe { embedded_can::ExtendedId::new_unchecked(self.0) }
    }
}

impl Into<embedded_can::Id> for Id {
    fn into(self) -> embedded_can::Id {
        match self {
            Self::Standard(sid) => embedded_can::Id::Standard(sid.into()),
            Self::Extended(eid) => embedded_can::Id::Extended(eid.into()),
        }
    }
}
