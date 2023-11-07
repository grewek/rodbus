use scursor::{WriteCursor, ReadCursor};

use crate::{RequestError, Indexed};

use super::write_single::SingleWriteOperation;

#[derive(Debug,PartialEq,Copy,Clone)]
pub struct MaskWriteRegister {
    pub(crate) and_mask: u16,
    pub(crate) or_mask: u16,
}

impl std::fmt::Display for MaskWriteRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}