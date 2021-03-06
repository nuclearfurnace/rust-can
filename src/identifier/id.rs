use std::{cmp, fmt};

use crate::constants::IdentifierFlags;

/// Standard (11-bit) CAN identifier.
///
/// Commonly referred to as CAN 2.0A, a standard identifier falls within the range of 0 to 0x7FF, inclusive.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct StandardId {
    identifier: u16,
    flags: IdentifierFlags,
}

impl StandardId {
    /// Minimum value for a standard identifier.
    ///
    /// This is the highest priority standard identifier.
    pub const ZERO: Self = Self {
        identifier: 0,
        flags: IdentifierFlags::empty(),
    };

    /// Maximum value for a standard identifier.
    ///
    /// This is the lowest priority standard identifier.
    pub const MAX: Self = Self {
        identifier: 0x7FF,
        flags: IdentifierFlags::empty(),
    };

    /// Creates a `StandardId`.
    ///
    /// Returns `None` if `identifier` is greater than [`MAX`][Self::MAX].
    #[inline]
    pub const fn new(identifier: u16) -> Option<Self> {
        if identifier <= Self::MAX.as_raw() {
            Some(Self {
                identifier,
                flags: IdentifierFlags::empty(),
            })
        } else {
            None
        }
    }

    /// Creates a `StandardId` with additional flags.
    ///
    /// Returns `None` if `identifier` is greater than [`MAX`][Self::MAX].
    #[inline]
    pub const fn with_flags(identifier: u16, flags: IdentifierFlags) -> Option<Self> {
        if identifier <= Self::MAX.as_raw() {
            Some(Self {
                identifier,
                flags: flags.difference(IdentifierFlags::EXTENDED),
            })
        } else {
            None
        }
    }

    /// Returns the identifier as a raw integer.
    #[inline]
    pub const fn as_raw(&self) -> u16 {
        self.identifier
    }

    /// Returns the flags set for this identifier.
    #[inline]
    pub const fn flags(&self) -> IdentifierFlags {
        self.flags
    }

    /// Creates a new `StandardId` after setting its flags to a new value.
    #[inline]
    pub const fn set_flags(self, flags: IdentifierFlags) -> Self {
        Self {
            identifier: self.identifier,
            flags,
        }
    }

    /// Creates a new `StandardId` after mapping its flags to a new value.
    #[inline]
    pub fn map_flags<F>(self, f: F) -> Self
    where
        F: FnOnce(IdentifierFlags) -> IdentifierFlags,
    {
        Self {
            identifier: self.identifier,
            flags: f(self.flags),
        }
    }

    /// Returns an extended version of this identifier.
    #[inline]
    pub const fn as_extended_id(&self) -> ExtendedId {
        ExtendedId {
            identifier: self.identifier as u32,
            flags: self.flags.union(IdentifierFlags::EXTENDED),
        }
    }
}

impl fmt::Display for StandardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = if self.flags.is_empty() {
            String::new()
        } else {
            format!("({:?})", self.flags)
        };
        write!(f, "{:#X}{}", self.identifier, flags)
    }
}

/// Extended (29-bit) CAN identifier.
///
/// Commonly referred to as CAN 2.0B, an extended identifier falls within the range of 0 to
/// 0x1FFFFFFF, inclusive.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct ExtendedId {
    identifier: u32,
    flags: IdentifierFlags,
}

impl ExtendedId {
    /// Minimum value for an extended identifier.
    ///
    /// This is the highest priority extended identifier.
    pub const ZERO: Self = Self {
        identifier: 0,
        flags: IdentifierFlags::EXTENDED,
    };

    /// Maximum value for ban extended identifier.
    ///
    /// This is the lowest priority extended identifier.
    pub const MAX: Self = Self {
        identifier: 0x1FFF_FFFF,
        flags: IdentifierFlags::EXTENDED,
    };

    /// Creates an `ExtendedId`.
    ///
    /// Returns `None` if `identifier` is greater than [`MAX`][Self::MAX].
    #[inline]
    pub const fn new(identifier: u32) -> Option<Self> {
        if identifier <= Self::MAX.identifier {
            Some(Self {
                identifier,
                flags: IdentifierFlags::EXTENDED,
            })
        } else {
            None
        }
    }

    /// Creates an `ExtendedId` with additional flags.
    ///
    /// Returns `None` if `identifier` is greater than [`MAX`][Self::MAX].
    #[inline]
    pub const fn with_flags(identifier: u32, flags: IdentifierFlags) -> Option<Self> {
        if identifier <= Self::MAX.as_raw() {
            Some(Self {
                identifier,
                flags: flags.union(IdentifierFlags::EXTENDED),
            })
        } else {
            None
        }
    }

    /// Returns the identifier as a raw integer.
    #[inline]
    pub const fn as_raw(&self) -> u32 {
        self.identifier
    }

    /// Returns the flags set for this identifier.
    #[inline]
    pub const fn flags(&self) -> IdentifierFlags {
        self.flags
    }

    /// Creates a new `ExtendedId` after setting its flags to a new value.
    #[inline]
    pub const fn set_flags(self, flags: IdentifierFlags) -> Self {
        Self {
            identifier: self.identifier,
            flags: flags.union(IdentifierFlags::EXTENDED),
        }
    }

    /// Creates a new `ExtendedId` after mapping its flags to a new value.
    #[inline]
    pub fn map_flags<F>(self, f: F) -> Self
    where
        F: FnOnce(IdentifierFlags) -> IdentifierFlags,
    {
        Self {
            identifier: self.identifier,
            flags: f(self.flags).union(IdentifierFlags::EXTENDED),
        }
    }

    /// Returns the base (standard) portion of this extended identifier.
    pub const fn as_standard_id(&self) -> StandardId {
        StandardId {
            identifier: (self.identifier >> 18) as u16,
            flags: self.flags.difference(IdentifierFlags::EXTENDED),
        }
    }
}

impl fmt::Display for ExtendedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = if self.flags.is_empty() {
            String::new()
        } else {
            format!("({:?})", self.flags)
        };
        write!(f, "{:#X}{}", self.identifier, flags)
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
    /// Returns the identifier as a raw integer.
    pub const fn as_raw(&self) -> u32 {
        match self {
            Self::Standard(sid) => sid.as_raw() as u32,
            Self::Extended(eid) => eid.as_raw(),
        }
    }

    /// Returns the flags set for this identifier.
    pub const fn flags(&self) -> IdentifierFlags {
        match self {
            Self::Standard(id) => id.flags(),
            Self::Extended(id) => id.flags(),
        }
    }

    /// Creates a new `Id` after setting its flags to a new value.
    #[inline]
    pub const fn set_flags(self, flags: IdentifierFlags) -> Self {
        match self {
            Self::Standard(id) => Self::Standard(id.set_flags(flags)),
            Self::Extended(id) => Self::Extended(id.set_flags(flags)),
        }
    }

    /// Creates a new `Id` after mapping its flags to a new value.
    #[inline]
    pub fn map_flags<F>(self, f: F) -> Self
    where
        F: FnOnce(IdentifierFlags) -> IdentifierFlags,
    {
        match self {
            Self::Standard(id) => Self::Standard(id.map_flags(f)),
            Self::Extended(id) => Self::Extended(id.map_flags(f)),
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

#[cfg(feature = "embedded-can-compat")]
#[cfg_attr(docsrs, doc(cfg(feature = "embedded-can-compat")))]
impl Into<embedded_can::StandardId> for StandardId {
    fn into(self) -> embedded_can::StandardId {
        unsafe { embedded_can::StandardId::new_unchecked(self.identifier) }
    }
}

#[cfg(feature = "embedded-can-compat")]
#[cfg_attr(docsrs, doc(cfg(feature = "embedded-can-compat")))]
impl Into<embedded_can::ExtendedId> for ExtendedId {
    fn into(self) -> embedded_can::ExtendedId {
        unsafe { embedded_can::ExtendedId::new_unchecked(self.identifier) }
    }
}

#[cfg(feature = "embedded-can-compat")]
#[cfg_attr(docsrs, doc(cfg(feature = "embedded-can-compat")))]
impl Into<embedded_can::Id> for Id {
    fn into(self) -> embedded_can::Id {
        match self {
            Self::Standard(sid) => embedded_can::Id::Standard(sid.into()),
            Self::Extended(eid) => embedded_can::Id::Extended(eid.into()),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::constants::tests::arb_identifier_flags;

    use super::{ExtendedId, Id, StandardId};
    use proptest::{prop_oneof, strategy::Strategy};

    const STANDARD_ID_MIN: u16 = StandardId::ZERO.as_raw();
    const STANDARD_ID_MAX: u16 = StandardId::MAX.as_raw();
    const EXTENDED_ID_MIN: u32 = ExtendedId::ZERO.as_raw();
    const EXTENDED_ID_MAX: u32 = ExtendedId::MAX.as_raw();

    pub fn arb_standardid() -> impl Strategy<Value = StandardId> {
        ((STANDARD_ID_MIN..=STANDARD_ID_MAX), arb_identifier_flags()).prop_map(|(id, flags)| {
            StandardId::with_flags(id, flags)
                .expect("arbitrary impl should never generate invalid standard IDs")
        })
    }

    pub fn arb_extendedid() -> impl Strategy<Value = ExtendedId> {
        ((EXTENDED_ID_MIN..=EXTENDED_ID_MAX), arb_identifier_flags()).prop_map(|(id, flags)| {
            ExtendedId::with_flags(id, flags)
                .expect("arbitrary impl should never generate invalid extended IDs")
        })
    }

    pub fn arb_id() -> impl Strategy<Value = Id> {
        prop_oneof![
            arb_standardid().prop_map(Id::from).boxed(),
            arb_extendedid().prop_map(Id::from).boxed(),
        ]
    }
}
