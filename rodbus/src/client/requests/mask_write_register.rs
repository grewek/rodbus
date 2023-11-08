
/// Contains a and, or mask for the MaskWriteRegister Function Code.
#[derive(Debug,PartialEq,Copy,Clone)]
pub struct MaskWriteRegister {
    pub(crate) and_mask: u16,
    pub(crate) or_mask: u16,
}

impl std::fmt::Display for MaskWriteRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AND_MASK: {:#016b} OR_MASK: {:#016b}", self.and_mask, self.or_mask)
    }
}