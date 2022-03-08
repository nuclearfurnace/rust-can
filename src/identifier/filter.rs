use crate::constants::EFF_MASK;

use super::Id;

pub struct Mask(u32);

impl Mask {
    pub fn new(mask: u32) -> Option<Mask> {
        if mask > EFF_MASK {
            None
        } else {
            Some(Self(mask))
        }
    }
}
pub struct Filter {
    id: Id,
    mask: Mask,
}

impl Filter {
    pub fn new(id: Id, mask: Mask) -> Self {
        Self { id, mask }
    }

    pub fn from_identity(id: Id) -> Self {
        Self {
            id,
            mask: Mask(id.as_raw()),
        }
    }

    pub fn matches(&self, id: Id) -> bool {
        id.as_raw() & self.mask.0 == self.id.as_raw() & self.mask.0
    }
}

impl Into<socketcan::CANFilter> for Filter {
    fn into(self) -> socketcan::CANFilter {
        socketcan::CANFilter::new(self.id.as_raw(), self.mask.0).unwrap()
    }
}
