//! Various CAN-specific constants.
//!
//! These constants are primarily used by the high-level types in the crate, such as encoding the
//! frame type in an identifier, or masking specific identifiers in a filter.  However, they're
//! exposed here in case they are necessary and/or can provide value to users.

use bitflags::bitflags;

bitflags! {
    /// Identifier flags for indicating various frame types.
    ///
    /// These flags are applied logically in `can`, but flag values themselves correspond to the
    /// format used by the Linux [SocketCAN][socketcan] library.  This lets flags be applied
    /// logically to identifiers such that callers can construct their calls to the underlying CAN
    /// transceivers/controllers in whatever way is required, but also provides a happy path for
    /// SocketCAN users by allowing generation of the all-in-one 32-bit identifier value.
    ///
    /// [socketcan]: https://www.kernel.org/doc/Documentation/networking/can.txt
    #[repr(transparent)]
    pub struct IdentifierFlags: u32 {
        /// The frame is using the extended format i.e. 29-bit extended identifiers.
        const EXTENDED = 0x80000000;

        /// The frame is a remote transmission request.
        const REMOTE = 0x40000000;

        /// The frame is an error frame.
        const ERROR = 0x20000000;
    }
}

/// Mask for standard identifiers.
pub const SFF_MASK: u32 = 0x000007ff;

/// Mask for extended identifiers.
pub const EFF_MASK: u32 = 0x1fffffff;

#[cfg(test)]
pub(crate) mod tests {
    use proptest::{arbitrary::any as arb_any, strategy::Strategy};

    use super::IdentifierFlags;

    pub(crate) fn arb_identifier_flags() -> impl Strategy<Value = IdentifierFlags> {
        arb_any::<(bool, u8)>().prop_map(|(extended, frame_type)| {
            let id_length = if extended {
                IdentifierFlags::EXTENDED
            } else {
                IdentifierFlags::empty()
            };

            let frame_type = match frame_type % 3 {
                0 => IdentifierFlags::empty(),
                1 => IdentifierFlags::REMOTE,
                _ => IdentifierFlags::ERROR,
            };

            id_length.union(frame_type)
        })
    }
}
