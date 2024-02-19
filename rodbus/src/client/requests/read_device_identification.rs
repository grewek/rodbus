use scursor::{ReadCursor, WriteCursor};
use tokio::sync::oneshot::Sender;

use crate::{common::{function::FunctionCode, traits::Serialize}, AppDecodeLevel, DeviceInfo, ReadDeviceCode, ReadDeviceRequest, RequestError, DeviceInfoObjectIterator, MeiCode, DeviceConformityLevel};

pub(crate) struct ReadDevice {
    pub(crate) request: ReadDeviceRequest,
    promise: Promise,
}

pub(crate) trait DeviceIdentificationCallback:
    FnOnce(Result<DeviceInfo, RequestError>) + Send + Sync + 'static
{
}

impl<T> DeviceIdentificationCallback for T where
    T: FnOnce(Result<DeviceInfo, RequestError>) + Send + Sync + 'static
{
}

pub(crate) struct Promise {
    callback: Option<Box<dyn DeviceIdentificationCallback>>,
}

impl Drop for Promise {
    fn drop(&mut self) {
        self.failure(RequestError::Shutdown);
    }
}

impl Promise {
    pub(crate) fn new<T>(callback: T) -> Self
    where
        T: DeviceIdentificationCallback,
    {
        Self {
            callback: Some(Box::new(callback)),
        }
    }

    pub(crate) fn failure(&mut self, err: RequestError) {
        self.complete(Err(err));
    }

    pub(crate) fn success(&mut self, identifier: DeviceInfo) {
        self.complete(Ok(identifier));
    }

    fn complete(&mut self, x: Result<DeviceInfo, RequestError>) {
        if let Some(callback) = self.callback.take() {
            callback(x);
        }
    }
}

impl ReadDevice {
    fn new(request: ReadDeviceRequest, promise: Promise) -> Self {
        Self { request, promise }
    }

    pub(crate) fn serialize(&self, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        self.request.serialize(cursor)
    }

    pub(crate) fn channel(
        request: ReadDeviceRequest,
        tx: Sender<Result<DeviceInfo, RequestError>>,
    ) -> Self {
        Self::new(
            request,
            Promise::new(|x: Result<DeviceInfo, RequestError>| {
                let _ = tx.send(x);
            }),
        )
    }

    pub(crate) fn failure(&mut self, err: RequestError) {
        self.promise.failure(err);
    }

    pub(crate) fn handle_response(
        &mut self,
        mut cursor: ReadCursor,
        function: FunctionCode,
        decode: AppDecodeLevel,
    ) -> Result<(), RequestError> {
        let response = Self::parse_device_identification_response(&mut cursor)?;

        if decode.enabled() {
            tracing::info!("PDU RX - {} {}", function, response,);
        }

        self.promise.success(response);
        Ok(())
    }

    fn parse_device_identification_response(
        cursor: &mut ReadCursor,
    ) -> Result<DeviceInfo, RequestError> {

        cursor.read_u8()?; //Consume the MEI Code
        //TODO(Kay): Consume the Read Device ID Code that is usually Echoed by the Server.
        let conformity_level = cursor.read_u8()?;
        let more_follows = cursor.read_u8()?;
        let value = cursor.read_u8()?;
        //let value = cursor.read_u8()?;
        //let object_count = cursor.read_u8()?;
        let object_count = cursor.read_u8()?;

        //let mut test_objects = vec![];

        let objects = cursor.read_all();

        //TODO(Kay): This messy code works as it should ! Now here is the tricky part getting
        //           this mess into a state which is working and is not a sore for the eyes...


        //TODO(Kay): We need to figure out a better type here probably maybe even one that encapsulates the
        //           DeviceInfoObjectIterator...
        let iter = DeviceInfoObjectIterator::new(objects);

        let result = DeviceInfo::new(MeiCode::ReadDeviceId,
                        ReadDeviceCode::ExtendedStreaming,
                        DeviceConformityLevel::BasicIdentificationStream,
                        object_count,
                        iter);

        Ok(result)
    }

    /*fn parse_device_info_objects(
        read_device_code: ReadDeviceCode,
        container: &mut Vec<InfoObject>,
        cursor: &mut ReadCursor,
    ) -> Result<(), RequestError> {
        loop {
            //TODO(Kay): Figure out if there is a potential that this loop will hang ?
            let obj_id = cursor.read_u8()?;
            let obj_length = cursor.read_u8()?;
            let data = cursor.read_bytes(obj_length as usize)?;
            let object = InfoObject::new(obj_id, data);
            container.push(object);

            if cursor.is_empty() {
                break;
            }
        }

        Ok(())
    }*/
}
