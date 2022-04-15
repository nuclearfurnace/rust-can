//! OBD-specific (On-board diagnostics) identifiers, based on ISO 15765-4.

use std::fmt;

use super::{filter::Filter, ExtendedId, Id, StandardId};

const OBD_BROADCAST_ADDR_STANDARD: Id = Id::Standard(standard_id(0x7DF));
const OBD_BROADCAST_ADDR_EXTENDED: Id = Id::Extended(extended_id(0x18DB33F1));
const OBD_REQ_ADDR_START_STANDARD: Id = Id::Standard(standard_id(0x7E0));
const OBD_REQ_ADDR_END_STANDARD: Id = Id::Standard(standard_id(0x7E7));
const OBD_RESP_ADDR_START_STANDARD: Id = Id::Standard(standard_id(0x7E8));
const OBD_RESP_ADDR_END_STANDARD: Id = Id::Standard(standard_id(0x7EF));
const OBD_REQ_ADDR_START_EXTENDED: Id = Id::Extended(extended_id(0x18DA00F1));
const OBD_REQ_ADDR_END_EXTENDED: Id = Id::Extended(extended_id(0x18DAFFF1));
const OBD_RESP_ADDR_START_EXTENDED: Id = Id::Extended(extended_id(0x18DAF100));
const OBD_RESP_ADDR_END_EXTENDED: Id = Id::Extended(extended_id(0x18DAF1FF));
const OBD_REQ_RESP_ADDR_OFFSET_STANDARD: u16 = 8;

/// Functional request address for legislated OBD diagnostic messages.
///
/// For legislated OBD diagnostic services in automobiles, this functional request address can be
/// used a broadcast address.  This means that any device providing the aforementioned OBD services
/// will treat a message to this address is if it had been addressed directly.
///
/// As such, it is useful for discovering all devices on the bus that support diagnostic services.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct DiagnosticBroadcastAddress(Id);

impl DiagnosticBroadcastAddress {
    /// Gets the diagnostic broadcast address for standard addressing.
    ///
    /// Standard addressing refers to the 11-bit addressing mode, also known as CAN 2.0A.
    ///
    /// The identifier in this addressing mode is 0x7DF, as outlined by ISO 15765-4:2005(E), section
    /// 6.3.2.2, table 3, "11 bit legislated-OBD CAN identifiers".
    pub const fn standard() -> Self {
        Self(OBD_BROADCAST_ADDR_STANDARD)
    }

    /// Gets the diagnostic broadcast address for extended addressing.
    ///
    /// Extended addressing refers to the 29-bit addressing mode, also known as CAN 2.0B.
    ///
    /// The identifier in this addressing mode is 0x18DB33F1, as outlined by ISO 15765-4:2005(E),
    /// section 6.3.2.3, table 5, "29 bit legislated-OBD CAN identifiers".
    pub const fn extended() -> Self {
        Self(OBD_BROADCAST_ADDR_EXTENDED)
    }

    /// Gets the identifier that this broadcast address represents.
    pub fn id(&self) -> Id {
        self.0
    }
}

impl fmt::Display for DiagnosticBroadcastAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Into<Id> for DiagnosticBroadcastAddress {
    fn into(self) -> Id {
        self.0
    }
}

/// Physical request address for legislated OBD diagnostic messages.
///
/// For legislated OBD diagnostic services in automobile, this physical request address represents
/// an identifier that a specific device on the bus will answer to, such as the powertrain control
/// module. As multiple legislated OBD devices may be present on this bus, this could represent any
/// one of those devices.  Physical request addresses and physical response addresses are paired
/// with one another, such that for any given request address, the response address is unique and
/// can be calculated ahead of time.
///
/// Based on the identifier lengths, up to eight (8) devices can be addressed using standard
/// addressing (11-bit identifier) while up to 256 devices can be addressed using extended
/// addressing (29-bit identifier).  However, the number of legislated OBD devices is not allowed to
/// exceed eight (8) devices, regardless of addressing scheme.  Additionally, it is recommended, but
/// not required, that these devices use the lowest possible numbering within their respective
/// identifier ranges.  Thus, while standard addressing only supports eight (8) addresses and thus
/// implicitly constrains the expected identifiers, systems using extended addressing may have up to
/// eight (8) devices assigned anywhere within the the range of the 256 possible identifiers.
///
/// Practically speaking, this is the identifier that an external test device would send messages to.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct DiagnosticRequestAddress(Id);

impl DiagnosticRequestAddress {
    /// Creates a [`DiagnosticRequestAddress`] from the given identifier.
    ///
    /// Depending on the addressing mode of the identifier, a certain range of identifiers are valid
    /// for legislated OBD purposes.  If the given identifier is not within that range, `None` will
    /// be returned.
    pub fn from_id(id: Id) -> Option<DiagnosticRequestAddress> {
        let is_standard = id >= OBD_REQ_ADDR_START_STANDARD && id <= OBD_REQ_ADDR_END_STANDARD;
        let is_extended = id >= OBD_REQ_ADDR_START_EXTENDED && id <= OBD_REQ_ADDR_END_EXTENDED;

        if is_standard || is_extended {
            Some(Self(id))
        } else {
            None
        }
    }

    /// Gets the identifier that this request address represents.
    pub fn id(&self) -> Id {
        self.0
    }

    /// Creates the reciprocal [`DiagnosticResponseAddress`] to this request addresses.
    ///
    /// See the documentation of [`DiagnosticRequestAddress`] for more information.
    pub fn into_response_address(&self) -> DiagnosticResponseAddress {
        match self.0 {
            Id::Standard(sid) => {
                let raw_offset_id = sid.as_raw() + OBD_REQ_RESP_ADDR_OFFSET_STANDARD;
                let response_id = StandardId::new(raw_offset_id).unwrap();
                DiagnosticResponseAddress(Id::Standard(response_id))
            }
            Id::Extended(eid) => {
                let raw_offset_id = swap_eid_target_source(eid.as_raw());
                let response_id = ExtendedId::new(raw_offset_id).unwrap();
                DiagnosticResponseAddress(Id::Extended(response_id))
            }
        }
    }
}

impl Into<Id> for DiagnosticRequestAddress {
    fn into(self) -> Id {
        self.0
    }
}

impl fmt::Display for DiagnosticRequestAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Physical response address for legislated OBD diagnostic messages.
///
/// For legislated OBD diagnostic services in automobile, this physical response address represents
/// an identifier that a specific device on the bus will send its response to, which is typically an
/// external test device. Physical request addresses and physical response addresses are paired with
/// one another, such that for any given request address, the response address is unique and can be
/// calculated ahead of time.
///
/// Based on the identifier lengths, up to eight (8) devices can be addressed using standard
/// addressing (11-bit identifier) while up to 256 devices can be addressed using extended
/// addressing (29-bit identifier). However, the number of legislated OBD devices is not allowed to
/// exceed eight (8) devices, regardless of addressing scheme. Additionally, it is recommended, but
/// not required, that these devices use the lowest possible numbering within their respective
/// identifier ranges. Thus, while standard addressing only supports eight (8) addresses and thus
/// implicitly constrains the expected identifiers, systems using extended addressing may have up to
/// eight (8) devices assigned anywhere within the the range of the 256 possible identifiers.
///
/// Practically speaking, this is the identifier that an external test device would receive messages
/// on.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct DiagnosticResponseAddress(Id);

impl DiagnosticResponseAddress {
    /// Creates a [`DiagnosticResponseAddress`] from the given identifier.
    ///
    /// Depending on the addressing mode of the identifier, a certain range of identifiers are valid
    /// for legislated OBD purposes.  If the given identifier is not within that range, `None` will
    /// be returned.
    pub fn from_id(id: Id) -> Option<DiagnosticResponseAddress> {
        let is_standard = id >= OBD_RESP_ADDR_START_STANDARD && id <= OBD_RESP_ADDR_END_STANDARD;
        let is_extended = id >= OBD_RESP_ADDR_START_EXTENDED && id <= OBD_RESP_ADDR_END_EXTENDED;

        if is_standard || is_extended {
            Some(Self(id))
        } else {
            None
        }
    }

    /// Gets the identifier that this response address represents.
    pub fn id(&self) -> Id {
        self.0
    }

    /// Creates the reciprocal [`DiagnosticRequestAddress`] to this request addresses.
    ///
    /// See the documentation of [`DiagnosticResponseAddress`] for more information.
    pub fn into_request_address(&self) -> DiagnosticRequestAddress {
        match self.0 {
            Id::Standard(sid) => {
                let raw_offset_id = sid.as_raw() - OBD_REQ_RESP_ADDR_OFFSET_STANDARD;
                let response_id = StandardId::new(raw_offset_id).unwrap();
                DiagnosticRequestAddress(Id::Standard(response_id))
            }
            Id::Extended(eid) => {
                let raw_offset_id = swap_eid_target_source(eid.as_raw());
                let response_id = ExtendedId::new(raw_offset_id).unwrap();
                DiagnosticRequestAddress(Id::Extended(response_id))
            }
        }
    }
}

impl Into<Id> for DiagnosticResponseAddress {
    fn into(self) -> Id {
        self.0
    }
}

impl fmt::Display for DiagnosticResponseAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Filter for physical response addresses for legislated OBD diagnostic messages.
///
/// When initially querying the bus for any available legislated OBD devices, using the functional
/// request (broadcast) addresses, it can be useful to filter out all identifiers that are not used
/// for responding to the broadcast address.
///
/// This filter only matches identifiers that are valid in the context of being mappable to a
/// [`DiagnosticResponseAddress`].
pub struct DiagnosticResponseFilter;

impl DiagnosticResponseFilter {
    /// Gets the filter for physical response identifiers when using standard addressing.
    ///
    /// Standard addressing refers to the 11-bit addressing mode, also known as CAN 2.0A.
    ///
    /// Matches identifiers 0x7E8 to 0x7EF, as outlined by ISO 15765-4:2005(E), section
    /// 6.3.2.2, table 3, "11 bit legislated-OBD CAN identifiers".
    pub const fn standard() -> Filter {
        Filter::range(OBD_RESP_ADDR_START_STANDARD, OBD_RESP_ADDR_END_STANDARD)
    }

    /// Gets the filter for physical response identifiers when using extended addressing.
    ///
    /// Extended addressing refers to the 29-bit addressing mode, also known as CAN 2.0B.
    ///
    /// Matches identifiers 0x18DAF100 to 0x18DAF1FF, as outlined by ISO 15765-4:2005(E),
    /// section 6.3.2.3, table 5, "29 bit legislated-OBD CAN identifiers".
    pub const fn extended() -> Filter {
        Filter::range(OBD_RESP_ADDR_START_EXTENDED, OBD_RESP_ADDR_END_EXTENDED)
    }
}

const fn standard_id(id: u16) -> StandardId {
    match StandardId::new(id) {
        Some(id) => id,
        None => panic!("invalid standard ID"),
    }
}

const fn extended_id(id: u32) -> ExtendedId {
    match ExtendedId::new(id) {
        Some(id) => id,
        None => panic!("invalid extended ID"),
    }
}

const fn swap_eid_target_source(eid_raw: u32) -> u32 {
    eid_raw & 0xFFFF0000 | (eid_raw & 0x0000FF00) >> 8 | (eid_raw & 0x000000FF) << 8
}

#[cfg(test)]
mod tests {
    use crate::identifier::obd::swap_eid_target_source;

    #[test]
    fn test_swap_eid_target_source() {
        let input = 0x18DAF142;
        let expected = 0x18DA42F1;

        assert_eq!(expected, swap_eid_target_source(input));
    }
}
