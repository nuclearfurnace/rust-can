//! CAN frame.

use bytes::Bytes;

use crate::{constants::IdentifierFlags, identifier::Id};

/// A CAN frame.
///
/// ## High-level structure
///
/// Logically, a CAN frame contains both an identifier and a payload.  Within the identifier,
/// the three of the four possible frame types are encoded: data, remote, and error.
///
/// Additionally, while a CAN frame, as seen being transmitted over the bus, is limited to eight
/// bytes of data, a logical frame can represent far more than that depending on whether or not any
/// additional transport layers are used on top of CAN itself, such as [ISO-TP][isotp].
///
/// As `Frame` is intended to be used by code that delegates the handling of low-level CAN details
/// to the operating system, or controller/transmitter peripherals, we focus purely on the logical
/// use cases, which is why `Frame` could be used for pure CAN, or ISO-TP and other transport
/// protocols, without necessarily needing to specialize the types involved.
///
/// [isotp]: https://en.wikipedia.org/wiki/ISO_15765-2
pub struct Frame {
    id: Id,
    data: Bytes,
}

impl Frame {
    /// Creates a frame from an identifier and data.
    pub const fn new(id: Id, data: Bytes) -> Self {
        Self { id, data }
    }

    /// Creates a frame from an identifier and static byte slice.
    pub const fn from_static(id: Id, data: &'static [u8]) -> Self {
        Self {
            id,
            data: Bytes::from_static(data),
        }
    }

    /// Gets the identifier of this frame.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Gets the flags of the identifier in this frame.
    pub const fn flags(&self) -> IdentifierFlags {
        self.id.flags()
    }

    /// Gets the data of this frame.
    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }

    /// Whether or not this is a data frame.
    pub const fn is_data_frame(&self) -> bool {
        !self
            .id
            .flags()
            .intersects(IdentifierFlags::ERROR.union(IdentifierFlags::REMOTE))
    }

    /// Whether or not this is a remote frame.
    pub const fn is_remote_frame(&self) -> bool {
        self.id.flags().contains(IdentifierFlags::REMOTE)
    }

    /// Whether or not this is an error frame.
    pub const fn is_error_frame(&self) -> bool {
        self.id.flags().contains(IdentifierFlags::ERROR)
    }
}
