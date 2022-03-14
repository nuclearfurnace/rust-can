use bitflags::bitflags;

bitflags! {
    /// Identifier flags for indicating various frame types.
    ///
    /// These flags are applied logically in `can`, but flag values themselves correspond to the format used
    /// by the Linux SocketCAN library.  This lets flags be applied logically to identifiers such
    /// that callers can construct their calls to the underlying CAN transceivers/controllers in
    /// whatever way is required, but also provides a happy path for SocketCAN users by allowing
    /// generation of the all-in-one 32-bit identifier value.
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
