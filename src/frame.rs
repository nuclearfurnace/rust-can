//! CAN frame.

use bytes::{BufMut, Bytes, BytesMut};

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

    /// Creates a new `Frame` that is compliant as an ISO-TP "Single Frame".
    ///
    /// The existing identifier and data are copied over to the new frame, and the length of the
    /// existing data is prepended to the data as a single byte.
    ///
    /// # Errors
    ///
    /// If the size of the data in the current frame is too large to fit in an ISO-TP "Single
    /// Frame", then `None` is returned.
    pub fn as_isotp_frame(&self) -> Option<Self> {
        if self.data.len() > 7 {
            return None;
        }

        let data_len = u8::try_from(self.data.len()).expect("self.data.len() must be less than 8");
        let mut new_data = BytesMut::with_capacity(1 + self.data.len());
        new_data.put_u8(data_len);
        new_data.extend_from_slice(&self.data);

        Some(Self {
            id: self.id,
            data: new_data.freeze(),
        })
    }
}
