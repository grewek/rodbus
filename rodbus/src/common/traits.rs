use crate::decode::AppDecodeLevel;
use crate::error::*;
use crate::ExceptionCode;

use scursor::{ReadCursor, WriteCursor};
use crate::common::frame::FrameRecords;

pub(crate) trait Serialize {
    //TODO(Kay): In order to use the FrameRecords API it was necessary to add a records argument
    //           to the serialize trait. As it needs access to the WriteCursor and this was the
    //           most simple way to get access to it ! Again i don't think this is necessarily
    //           a good API it's more of a "how it could work" approach and not a "how it should
    //           should work" approach.
    fn serialize(&self, records: &mut FrameRecords, cursor: &mut WriteCursor) -> Result<(), RequestError>;
}

pub(crate) trait Loggable {
    fn log(
        &self,
        bytes: &[u8],
        level: AppDecodeLevel,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result;
}

pub(crate) struct LoggableDisplay<'a, 'b> {
    loggable: &'a dyn Loggable,
    bytes: &'b [u8],
    level: AppDecodeLevel,
}

impl<'a, 'b> LoggableDisplay<'a, 'b> {
    pub(crate) fn new(loggable: &'a dyn Loggable, bytes: &'b [u8], level: AppDecodeLevel) -> Self {
        Self {
            loggable,
            bytes,
            level,
        }
    }
}

impl std::fmt::Display for LoggableDisplay<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.loggable.log(self.bytes, self.level, f)
    }
}

pub(crate) trait Parse: Sized {
    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError>;
}

impl Loggable for ExceptionCode {
    fn log(
        &self,
        _bytes: &[u8],
        _level: AppDecodeLevel,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
