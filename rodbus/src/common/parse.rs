use crate::client::requests::mask_write_register::MaskWriteRegister;
use crate::common::traits::Parse;
use crate::{error::*, ReadDeviceRequest, ReadDeviceCode, ExceptionCode, MeiCode};
use crate::types::{coil_from_u16, AddressRange, Indexed};

use scursor::ReadCursor;

impl Parse for AddressRange {
    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError> {
        Ok(AddressRange::try_from(
            cursor.read_u16_be()?,
            cursor.read_u16_be()?,
        )?)
    }
}

impl Parse for Indexed<bool> {
    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError> {
        Ok(Indexed::new(
            cursor.read_u16_be()?,
            coil_from_u16(cursor.read_u16_be()?)?,
        ))
    }
}

impl Parse for Indexed<u16> {
    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError> {
        Ok(Indexed::new(cursor.read_u16_be()?, cursor.read_u16_be()?))
    }
}

impl Parse for Indexed<MaskWriteRegister> {
    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError> {
        let reference_address = cursor.read_u16_be()?;
        let and_mask = cursor.read_u16_be()?;
        let or_mask = cursor.read_u16_be()?;
        Ok(Indexed::new(reference_address, MaskWriteRegister { and_mask: and_mask, or_mask: or_mask }))
    }
}

impl Parse for ReadDeviceRequest {
    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError> {
        let mei_type = cursor.read_u8()?.try_into()?;

        if mei_type == MeiCode::CanOpenGeneralReference {
            return Err(RequestError::Exception(ExceptionCode::IllegalDataValue))
        }

        let dev_id: ReadDeviceCode = cursor.read_u8()?.try_into()?;
        let obj_id = cursor.read_u8()?;

        Ok(Self {
            mei_code: mei_type,
            dev_id,
            obj_id: Some(obj_id),
        })
    }
}

#[cfg(test)]
mod coils {
    use crate::common::traits::Parse;
    use crate::error::AduParseError;
    use crate::types::Indexed;

    use scursor::ReadCursor;

    #[test]
    fn parse_fails_for_unknown_coil_value() {
        let mut cursor = ReadCursor::new(&[0x00, 0x01, 0xAB, 0xCD]);
        let result = Indexed::<bool>::parse(&mut cursor);
        assert_eq!(result, Err(AduParseError::UnknownCoilState(0xABCD).into()))
    }

    #[test]
    fn parse_succeeds_for_valid_coil_value_false() {
        let mut cursor = ReadCursor::new(&[0x00, 0x01, 0x00, 0x00]);
        let result = Indexed::<bool>::parse(&mut cursor);
        assert_eq!(result, Ok(Indexed::new(1, false)));
    }

    #[test]
    fn parse_succeeds_for_valid_coil_value_true() {
        let mut cursor = ReadCursor::new(&[0x00, 0x01, 0xFF, 0x00]);
        let result = Indexed::<bool>::parse(&mut cursor);
        assert_eq!(result, Ok(Indexed::new(1, true)));
    }

    #[test]
    fn parse_succeeds_for_valid_indexed_register() {
        let mut cursor = ReadCursor::new(&[0x00, 0x01, 0xCA, 0xFE]);
        let result = Indexed::<u16>::parse(&mut cursor);
        assert_eq!(result, Ok(Indexed::new(1, 0xCAFE)));
    }


}
#[cfg(test)]
mod read_device_info {
    use crate::{ReadDeviceRequest, MeiCode};
    use crate::common::traits::Parse;
    use crate::error::AduParseError;

    use scursor::ReadCursor;

    #[test]
    fn parse_fails_for_invalid_device_info_values() {
        let mut cursor = ReadCursor::new(&[0xFF, 0x01, 0x01]);

        let result = ReadDeviceRequest::parse(&mut cursor);
        assert_eq!(result, Err(AduParseError::MeiCodeOutOfRange(0xFF).into()));

        let mut cursor = ReadCursor::new(&[0x0E, 0xFF, 0x01]);
        
        let result = ReadDeviceRequest::parse(&mut cursor);
        assert_eq!(result, Err(AduParseError::DeviceCodeOutOfRange(0xFF).into()));

    }

    #[test]
    fn parse_succeeds_for_valid_device_info_values() {
        let mut cursor = ReadCursor::new(&[0x0E, 0x01, 0x00]);

        let result = ReadDeviceRequest::parse(&mut cursor);
        assert_eq!(result, Ok(ReadDeviceRequest { mei_code: MeiCode::ReadDeviceId, dev_id: crate::ReadDeviceCode::BasicStreaming, obj_id:  Some(0x00)}))
    }
}

#[cfg(test)]
mod mask_write_register {

    use crate::common::traits::Parse;
    use scursor::ReadCursor;

    use crate::{client::MaskWriteRegister, Indexed};

    #[test]
    fn parse_succeeds_for_valid_indexed_mask_write_register() {
        let mut cursor = ReadCursor::new(&[0x00, 0xBE, 0xCA,0x00,0x00,0xFE]);

        let result = Indexed::<MaskWriteRegister>::parse(&mut cursor);
        assert_eq!(result, Ok(Indexed::new(0xBE, MaskWriteRegister { and_mask: 0xCA00, or_mask: 0x00FE })));
    }
}