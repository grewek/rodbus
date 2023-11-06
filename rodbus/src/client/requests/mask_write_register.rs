#[derive(Debug,PartialEq,Copy,Clone)]
pub struct MaskWriteRegister {
    and_mask: u16,
    or_mask: u16,
}

impl std::fmt::Display for MaskWriteRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}