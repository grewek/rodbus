use std::convert::TryFrom;
use std::ops::Range;

use crate::client::WriteMultiple;
use crate::common::frame::constants::MAX_ADU_LENGTH;
use crate::common::traits::Loggable;
use crate::common::traits::Parse;
use crate::common::traits::Serialize;
use crate::error::{InternalError, RequestError};
use crate::server::response::DeviceIdentificationResponse;
use crate::server::response::{BitWriter, RegisterWriter};
use crate::types::{
    coil_from_u16, coil_to_u16, AddressRange, BitIterator, BitIteratorDisplay, Indexed,
    RegisterIterator, RegisterIteratorDisplay,
};
use crate::DeviceInfo;
use crate::ReadDeviceRequest;

use scursor::{ReadCursor, WriteCursor};
use crate::common::frame::FrameRecords;
use crate::server::ServerDeviceInfo;

pub(crate) fn calc_bytes_for_bits(num_bits: usize) -> Result<u8, InternalError> {
    let div_8 = num_bits / 8;

    let count = if num_bits % 8 == 0 { div_8 } else { div_8 + 1 };

    u8::try_from(count).map_err(|_| InternalError::BadByteCount(count))
}

pub(crate) fn calc_bytes_for_registers(num_registers: usize) -> Result<u8, InternalError> {
    let count = 2 * num_registers;
    u8::try_from(count).map_err(|_| InternalError::BadByteCount(count))
}

impl Serialize for AddressRange {
    fn serialize(&self, _: &mut FrameRecords, cur: &mut WriteCursor) -> Result<(), RequestError> {
        cur.write_u16_be(self.start)?;
        cur.write_u16_be(self.count)?;
        Ok(())
    }
}

impl Loggable for AddressRange {
    fn log(
        &self,
        payload: &[u8],
        level: crate::decode::AppDecodeLevel,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        if level.data_headers() {
            let mut cursor = ReadCursor::new(payload);

            if let Ok(value) = AddressRange::parse(&mut cursor) {
                write!(f, "{value}")?;
            }
        }

        Ok(())
    }
}

impl Serialize for crate::exception::ExceptionCode {
    fn serialize(&self, _: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        cursor.write_u8((*self).into())?;
        Ok(())
    }
}

impl Serialize for Indexed<bool> {
    fn serialize(&self, _: &mut FrameRecords, cur: &mut WriteCursor) -> Result<(), RequestError> {
        cur.write_u16_be(self.index)?;
        cur.write_u16_be(coil_to_u16(self.value))?;
        Ok(())
    }
}

impl Loggable for Indexed<bool> {
    fn log(
        &self,
        payload: &[u8],
        level: crate::decode::AppDecodeLevel,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        if level.data_headers() {
            let mut cursor = ReadCursor::new(payload);

            let index = match cursor.read_u16_be() {
                Ok(idx) => idx,
                Err(_) => return Ok(()),
            };
            let coil_raw_value = match cursor.read_u16_be() {
                Ok(value) => value,
                Err(_) => return Ok(()),
            };
            let coil_value = match coil_from_u16(coil_raw_value) {
                Ok(value) => value,
                Err(_) => return Ok(()),
            };
            let value = Indexed::new(index, coil_value);

            write!(f, "{value}")?;
        }

        Ok(())
    }
}

impl Serialize for Indexed<u16> {
    fn serialize(&self, _: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        cursor.write_u16_be(self.index)?;
        cursor.write_u16_be(self.value)?;
        Ok(())
    }
}

impl Loggable for Indexed<u16> {
    fn log(
        &self,
        payload: &[u8],
        level: crate::decode::AppDecodeLevel,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        if level.data_headers() {
            let mut cursor = ReadCursor::new(payload);

            let index = match cursor.read_u16_be() {
                Ok(idx) => idx,
                Err(_) => return Ok(()),
            };
            let raw_value = match cursor.read_u16_be() {
                Ok(value) => value,
                Err(_) => return Ok(()),
            };
            let value = Indexed::new(index, raw_value);

            write!(f, "{value}")?;
        }

        Ok(())
    }
}

impl Serialize for &[bool] {
    fn serialize(&self, _: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        // how many bytes should we have?
        let num_bytes = calc_bytes_for_bits(self.len())?;

        cursor.write_u8(num_bytes)?;

        for byte in self.chunks(8) {
            let mut acc: u8 = 0;
            for (count, bit) in byte.iter().enumerate() {
                if *bit {
                    acc |= 1 << count as u8;
                }
            }
            cursor.write_u8(acc)?;
        }

        Ok(())
    }
}

impl<T> Serialize for BitWriter<T>
where
    T: Fn(u16) -> Result<bool, crate::exception::ExceptionCode>,
{
    fn serialize(&self, _: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        let range = self.range.get();
        // write the number of bytes that follow
        let num_bytes = calc_bytes_for_bits(range.count as usize)?;
        cursor.write_u8(num_bytes)?;

        let mut acc = 0;
        let mut num_bits: usize = 0;

        // iterate over all the addresses, accumulating bits in the byte
        for address in self.range.get().iter() {
            if (self.getter)(address)? {
                // merge the bit into the byte
                acc |= 1 << num_bits;
            }
            num_bits += 1;
            if num_bits == 8 {
                // flush the byte
                cursor.write_u8(acc)?;
                acc = 0;
                num_bits = 0;
            }
        }

        // write any partial bytes
        if num_bits > 0 {
            cursor.write_u8(acc)?;
        }

        Ok(())
    }
}

impl<T> Loggable for BitWriter<T>
where
    T: Fn(u16) -> Result<bool, crate::exception::ExceptionCode>,
{
    fn log(
        &self,
        payload: &[u8],
        level: crate::decode::AppDecodeLevel,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        if level.data_headers() {
            let mut cursor = ReadCursor::new(payload);
            let _ = cursor.read_u8(); // ignore the byte count

            let iterator = match BitIterator::parse_all(self.range.get(), &mut cursor) {
                Ok(it) => it,
                Err(_) => return Ok(()),
            };

            write!(f, "{}", BitIteratorDisplay::new(level, iterator))?;
        }

        Ok(())
    }
}

impl<T> Serialize for RegisterWriter<T>
where
    T: Fn(u16) -> Result<u16, crate::exception::ExceptionCode>,
{
    fn serialize(&self, _: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        // write the number of bytes that follow
        let num_bytes = calc_bytes_for_registers(self.range.get().count as usize)?;
        cursor.write_u8(num_bytes)?;

        // iterate over all the addresses, accumulating the registers
        for address in self.range.get().iter() {
            let value = (self.getter)(address)?;
            cursor.write_u16_be(value)?;
        }

        Ok(())
    }
}

impl<T> Loggable for RegisterWriter<T>
where
    T: Fn(u16) -> Result<u16, crate::exception::ExceptionCode>,
{
    fn log(
        &self,
        payload: &[u8],
        level: crate::decode::AppDecodeLevel,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        if level.data_headers() {
            let mut cursor = ReadCursor::new(payload);
            let _ = cursor.read_u8(); // ignore the byte count

            let iterator = match RegisterIterator::parse_all(self.range.get(), &mut cursor) {
                Ok(it) => it,
                Err(_) => return Ok(()),
            };

            write!(f, "{}", RegisterIteratorDisplay::new(level, iterator))?;
        }

        Ok(())
    }
}

impl Serialize for &[u16] {
    fn serialize(&self, _: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        let num_bytes = calc_bytes_for_registers(self.len())?;
        cursor.write_u8(num_bytes)?;

        for value in *self {
            cursor.write_u16_be(*value)?
        }

        Ok(())
    }
}

impl Serialize for WriteMultiple<bool> {
    fn serialize(&self, records: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        self.range.serialize(records, cursor)?;
        self.values.as_slice().serialize(records, cursor)
    }
}

impl Serialize for WriteMultiple<u16> {
    fn serialize(&self, records: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        self.range.serialize(records, cursor)?;
        self.values.as_slice().serialize(records, cursor)
    }
}

impl Serialize for ReadDeviceRequest {
    fn serialize(&self, _: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        cursor.write_u8(self.mei_code as u8)?;
        cursor.write_u8(self.dev_id as u8)?;

        if let Some(value) = self.obj_id {
            cursor.write_u8(value)?;
        } else {
            cursor.write_u8(0x00)?;
        }

        Ok(())
    }
}



impl<'a, T> Serialize for DeviceIdentificationResponse<'a, T>
where
    T: Fn() -> Result<ServerDeviceInfo<'a>, crate::exception::ExceptionCode>,
{
    fn serialize(&self, records: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        //TODO(Kay): Fix this mess
        //TODO(Kay): We are still not conforming to the object count field and i don't know how to
        //           do that ?! As we don't have the original object count of the first message any
        //           more we also can't derive it from the data given by the user so now what ?
        let device_data: ServerDeviceInfo = (self.getter)()?;
        //NOTE: This will never change so we can just fix it in place
        //TODO(Kay): CONSTANT
        cursor.write_u8(0x0E)?;
        //FIXME(Kay): We need the Read Device ID Code or we are not conforming to the specification !
        cursor.write_u8(device_data.conformity_level as u8)?;

        //NOTE(Kay): Store the next two bytes (MORE_FOLLOWS) in our FrameRecords !
        let more_follows_indicator = records.push_record(cursor);
        let more_follows_value = records.push_record(cursor);

        //NOTE(Kay): We adding the object count position into our FrameRecords struct
        let written_object_count = records.push_record(cursor);
        let mut remaining_bytes = cursor.remaining();

        let mut id = 0;
        let mut length = 0;
        let mut read_cursor: usize = 0;

        let mut message_complete = false;
        let mut written_objects = 0;

        while remaining_bytes > 0 {
            if read_cursor == device_data.object_data.len() {
                message_complete = true;
                break;
            }


            id = device_data.object_data[read_cursor];
            read_cursor += 1;

            length = device_data.object_data[read_cursor];
            read_cursor += 1;


            if remaining_bytes < length as usize {
                break;
            }

            cursor.write_u8(id)?;
            cursor.write_u8(length)?;

            cursor.write_bytes(&device_data.object_data[read_cursor..(read_cursor + length as usize)])?;
            read_cursor += length as usize;
            written_objects +=  1;

            remaining_bytes = cursor.remaining();
        }

        if !message_complete {
            records.fill_record(more_follows_indicator, 0xFF, cursor);
            records.fill_record(more_follows_value, id, cursor);
        } else {
            records.fill_record(more_follows_indicator, 0x00, cursor);
            records.fill_record(more_follows_value, 0x00, cursor);
        }

        //TODO(Kay): We should not put the id of the object here but the amount of objects written
        //           into the stream !
        records.fill_record(written_object_count, written_objects, cursor);

        Ok(())
    }
}

impl<'a, T> Loggable for DeviceIdentificationResponse<'a, T>
where
    T: Fn() -> Result<ServerDeviceInfo<'a>, crate::exception::ExceptionCode>,
{
    fn log(
        &self,
        bytes: &[u8],
        level: crate::AppDecodeLevel,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        let mut cursor = ReadCursor::new(bytes);

        if level.data_headers() {
            writeln!(f, "DEVICE IDENTIFICATION RESPONSE")?;

            write!(f, "\t --> MEI CODE: {:X}", cursor.read_u8().unwrap())?;
            write!(
                f,
                "\t --> READ DEVICE CODE: {:X}",
                cursor.read_u8().unwrap()
            )?;
            write!(
                f,
                "\t --> CONFORMITY LEVEL: {:X}",
                cursor.read_u8().unwrap()
            )?;
        }

        if level.data_values() {
            writeln!(f, "DEVICE IDENTIFICATION RESPONSE")?;

            write!(f, "\t --> MEI CODE: {:X}", cursor.read_u8().unwrap())?;
            write!(
                f,
                "\t --> READ DEVICE CODE: {:X}",
                cursor.read_u8().unwrap()
            )?;
            write!(
                f,
                "\t --> CONFORMITY LEVEL: {:X}",
                cursor.read_u8().unwrap()
            )?;
            let raw_string_data = cursor.read_all();
            write!(f, "\t --> RAW STRING DATA: ")?;
            for str in raw_string_data {
                write!(f, "{:X}", str)?;
            }
        }

        Ok(())
    }
}

impl Serialize for Option<u8> {
    fn serialize(&self, _: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        const CONTINUE_MARKER: u8 = 0xFF;
        const END_MARKER: u8 = 0x00;
        match self {
            Some(value) => {
                cursor.write_u8(CONTINUE_MARKER)?;
                cursor.write_u8(*value)?;
            }
            None => {
                cursor.write_u8(END_MARKER)?;
                cursor.write_u8(0x00)?;
            }
        }
        Ok(())
    }
}

impl Serialize for &str {
    fn serialize(&self, _: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        cursor.write_u8(self.len() as u8)?;
        cursor.write_bytes(self.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_address_range() {
        let range = AddressRange::try_from(3, 512).unwrap();
        let mut buffer = [0u8; 4];
        let mut cursor = WriteCursor::new(&mut buffer);
        range.serialize(&mut FrameRecords::new(), &mut cursor).unwrap();
        assert_eq!(buffer, [0x00, 0x03, 0x02, 0x00]);
    }

    #[test]
    fn serialize_option() {
        let next_position = Some(0x3u8);
        let mut buffer = [0u8; 2];

        let mut cursor = WriteCursor::new(&mut buffer);

        next_position.serialize(&mut FrameRecords::new(), &mut cursor).unwrap();
        assert_eq!(buffer, [0xFF, 0x03]);

        let next_position = None;
        let mut buffer = [0u8; 2];

        let mut cursor = WriteCursor::new(&mut buffer);
        next_position.serialize(&mut FrameRecords::new(), &mut cursor).unwrap();
        assert_eq!(buffer, [0x00, 0x00]);
    }

    #[test]
    fn serialize_string() {
        let test_str: String = "Hello, World!".to_string();
        let mut buffer = [0u8; 14];

        let mut cursor = WriteCursor::new(&mut buffer);
        test_str.as_str().serialize(&mut FrameRecords::new(), &mut cursor).unwrap();

        let expected: [u8; 14] = [
            0x0D, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ];
        assert_eq!(buffer, expected);
    }
}
