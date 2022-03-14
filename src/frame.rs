use bytes::Bytes;

use crate::{constants::IdentifierFlags, identifier::Id};

pub struct Frame {
    id: Id,
    data: Bytes,
}

impl Frame {
    pub const fn from_static(id: Id, data: &'static [u8]) -> Self {
        Self {
            id,
            data: Bytes::from_static(data),
        }
    }

    pub const fn from_bytes(id: Id, data: Bytes) -> Self {
        Self { id, data }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub const fn flags(&self) -> IdentifierFlags {
        self.id.flags()
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }

    pub const fn is_data_frame(&self) -> bool {
        !self
            .id
            .flags()
            .intersects(IdentifierFlags::ERROR.union(IdentifierFlags::REMOTE))
    }

    pub const fn is_remote_frame(&self) -> bool {
        self.id.flags().contains(IdentifierFlags::REMOTE)
    }

    pub const fn is_error_frame(&self) -> bool {
        self.id.flags().contains(IdentifierFlags::ERROR)
    }
}
