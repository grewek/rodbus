
/// Contains a and, or mask for the MaskWriteRegister Function Code.
#[derive(Debug,PartialEq,Copy,Clone)]
pub struct MaskWriteRegister {
    pub(crate) and_mask: u16,
    pub(crate) or_mask: u16,
}

impl MaskWriteRegister {
    /// Create a new MaskWriteRegister structure with the specified masks
    pub fn new(and_mask: u16, or_mask: u16) -> Self {
        Self {
            and_mask,
            or_mask,
        }
    }

    /// Value will be masked with the Values from the Struct. This is the default mask operation
    /// from the Modbus Specifications Manual see Page 36 MODBUS Application Protocol 1.1b
    pub fn mask_value(&self, value: u16) -> u16 {
        (value & self.and_mask) | (self.or_mask & (!self.and_mask))
    }
}

impl std::fmt::Display for MaskWriteRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AND_MASK: {:#016b} OR_MASK: {:#016b}", self.and_mask, self.or_mask)
    }
}