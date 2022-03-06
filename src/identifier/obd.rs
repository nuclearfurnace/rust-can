use super::{ExtendedId, Id, StandardId};

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

pub struct DiagnosticBroadcastAddress(Id);

impl DiagnosticBroadcastAddress {
    pub const fn standard() -> Self {
        Self(OBD_BROADCAST_ADDR_STANDARD)
    }

    pub const fn extended() -> Self {
        Self(OBD_BROADCAST_ADDR_EXTENDED)
    }
}

impl DiagnosticBroadcastAddress {
    pub fn id(&self) -> Id {
        self.0
    }
}

impl Into<Id> for DiagnosticBroadcastAddress {
    fn into(self) -> Id {
        self.0
    }
}

pub struct DiagnosticRequestAddress(Id);

impl DiagnosticRequestAddress {
    pub fn from_id(id: Id) -> Option<DiagnosticRequestAddress> {
        let is_standard = id >= OBD_REQ_ADDR_START_STANDARD && id <= OBD_REQ_ADDR_END_STANDARD;
        let is_extended = id >= OBD_REQ_ADDR_START_EXTENDED && id <= OBD_REQ_ADDR_END_EXTENDED;

        if is_standard || is_extended {
            Some(Self(id))
        } else {
            None
        }
    }

    pub fn id(&self) -> Id {
        self.0
    }

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

pub struct DiagnosticResponseAddress(Id);

impl DiagnosticResponseAddress {
    pub fn from_id(id: Id) -> Option<DiagnosticResponseAddress> {
        let is_standard = id >= OBD_RESP_ADDR_START_STANDARD && id <= OBD_RESP_ADDR_END_STANDARD;
        let is_extended = id >= OBD_RESP_ADDR_START_EXTENDED && id <= OBD_RESP_ADDR_END_EXTENDED;

        if is_standard || is_extended {
            Some(Self(id))
        } else {
            None
        }
    }

    pub fn id(&self) -> Id {
        self.0
    }

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
