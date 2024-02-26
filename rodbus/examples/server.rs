use std::process::exit;
use std::collections::HashMap;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};

use rodbus::server::*;
use rodbus::*;


struct SimpleHandler {
    coils: Vec<bool>,
    discrete_inputs: Vec<bool>,
    holding_registers: Vec<u16>,
    input_registers: Vec<u16>,

    basic_info: [InfoObject; 3],
    regular_keys: [InfoObject; 4],
    extended_values: [InfoObject; 4],

    info_response_layout_basic: HashMap<u8, usize>,
    info_response_data_basic: Vec<u8>,

    info_response_layout_regular: HashMap<u8, usize>,
    info_response_data_regular: Vec<u8>,

    info_response_layout_extended: HashMap<u8, usize>,
    info_response_data_extended: Vec<u8>,
}

impl SimpleHandler {
    fn new(
        coils: Vec<bool>,
        discrete_inputs: Vec<bool>,
        holding_registers: Vec<u16>,
        input_registers: Vec<u16>,
    ) -> Self {
        Self {
            coils,
            discrete_inputs,
            holding_registers,
            input_registers,

            basic_info: [
                InfoObject::new(0x00, "Example Vendor".as_bytes()),
                InfoObject::new(0x01, "Little Dictionary".as_bytes()),
                InfoObject::new(0x02, "0.1.0".as_bytes())
            ],
            regular_keys: [
                InfoObject::new(0x03, "0x8A".as_bytes()),
                InfoObject::new(0x04, "0x8B".as_bytes()),
                InfoObject::new(0x05, "0x8C".as_bytes()),
                InfoObject::new(0x06, "0x8D".as_bytes())
            ],
            extended_values: [
                InfoObject::new(0x8A, "This is the value for key 0x8A".as_bytes()),
                InfoObject::new(0x8B, "Value for 0x8B which is a bit longer than your usual method to test if the behavior of sending responses is correct and works, and will it work over a different client as well ?".as_bytes()),
                InfoObject::new(0x8C, "Another value for 0x8C".as_bytes()),
                InfoObject::new(0x8D, "Last but not least the value for 0x8D".as_bytes())
            ],

            info_response_layout_basic: HashMap::new(),
            info_response_data_basic: vec![],

            info_response_layout_regular: HashMap::new(),
            info_response_data_regular: vec![],

            info_response_layout_extended: HashMap::new(),
            info_response_data_extended: vec![],
        }.generate_info_response()
    }

    fn coils_as_mut(&mut self) -> &mut [bool] {
        self.coils.as_mut_slice()
    }

    fn discrete_inputs_as_mut(&mut self) -> &mut [bool] {
        self.discrete_inputs.as_mut_slice()
    }

    fn holding_registers_as_mut(&mut self) -> &mut [u16] {
        self.holding_registers.as_mut_slice()
    }

    fn input_registers_as_mut(&mut self) -> &mut [u16] {
        self.input_registers.as_mut_slice()
    }

    fn generate_info_response(mut self) -> Self {
        for obj in &self.basic_info {
            match obj {
                InfoObject::DefinedString(id, data) => {
                    self.info_response_layout_basic.insert((*id).into(), self.info_response_data_basic.len());
                    self.info_response_data_basic.push((*id).into());
                    self.info_response_data_basic.push(data.len() as u8);
                    self.info_response_data_basic.extend_from_slice(data.as_bytes());
                },
                InfoObject::Other(id, data) =>{
                    self.info_response_layout_basic.insert((*id).into(), self.info_response_data_basic.len());
                    self.info_response_data_basic.push(*id);
                    self.info_response_data_regular.push(data.len() as u8);
                    self.info_response_data_basic.extend_from_slice(data.as_slice());
                },
            };
        }

        for obj in &self.regular_keys {
            match obj {
                InfoObject::DefinedString(id, data) => {
                    self.info_response_layout_regular.insert((*id).into(), self.info_response_data_regular.len());
                    self.info_response_data_regular.push((*id).into());
                    self.info_response_data_regular.push(data.len() as u8);
                    self.info_response_data_regular.extend_from_slice(data.as_bytes());
                },
                InfoObject::Other(id, data) =>{
                    self.info_response_layout_regular.insert((*id).into(), self.info_response_data_regular.len());
                    self.info_response_data_regular.push(*id);
                    self.info_response_data_regular.push(data.len() as u8);
                    self.info_response_data_regular.extend_from_slice(data.as_slice());
                },
            };
        }

        for obj in &self.extended_values {
            match obj {
                InfoObject::DefinedString(id, data) => {
                    self.info_response_layout_extended.insert((*id).into(), self.info_response_data_extended.len());
                    self.info_response_data_extended.push((*id).into());
                    self.info_response_data_extended.push(data.len() as u8);
                    self.info_response_data_extended.extend_from_slice(data.as_bytes());
                },
                InfoObject::Other(id, data) =>{
                    self.info_response_layout_extended.insert((*id).into(), self.info_response_data_extended.len());
                    self.info_response_data_extended.push(*id);
                    self.info_response_data_extended.push(data.len() as u8);
                    self.info_response_data_extended.extend_from_slice(data.as_slice());
                },
            };
        }

        /*for obj in self.regular_keys {
            match obj {
                InfoObject::DefinedString(id, data) | InfoObject::Other(id, data) => {
                    self.info_response_data_regular.push(id.try_into().unwrap());
                    self.info_response_data_regular.extend_from_slice(data.as_bytes());
                }
            }
        }

        for obj in self.extended_values {
            match obj {
                InfoObject::DefinedString(id, data) | InfoObject::Other(id, data) => {
                    self.info_response_data_extended.push(id.try_into().unwrap());
                    self.info_response_data_extended.extend_from_slice(data.as_bytes());
                }
            }
        }*/
        self
    }

    fn get_info_object_response_data(&self, read_device_code: ReadDeviceCode, object_id: Option<u8>) -> ServerDeviceInfo {
        todo!()
    }
    fn get_basic_info_response(&self, object_id: Option<u8>) -> &[u8] {
        if let Some(start) = object_id {
            //BUG: No object count !
            let start = self.info_response_layout_basic[&start];
            return &self.info_response_data_basic[start..];
        }

        &self.info_response_data_basic
    }

    fn get_extended_info_response(&self, object_id: Option<u8>) -> &[u8] {
        if let Some(start) = object_id {
            if self.info_response_layout_extended.contains_key(&start) {
                let start = self.info_response_layout_extended[&start];
                return &self.info_response_data_extended[start..];
            }
        }

        &self.info_response_data_extended
    }

    fn get_regular_info_response(&self, object_id: Option<u8>) -> &[u8] {
        if let Some(start) = object_id {
            let start = self.info_response_layout_regular[&start];
            return &self.info_response_data_regular[start..]
        }

        &self.info_response_data_regular
    }

}

// ANCHOR: request_handler
impl RequestHandler for SimpleHandler {
    fn read_coil(&self, address: u16) -> Result<bool, ExceptionCode> {
        self.coils.get(address as usize).to_result()
    }

    fn read_discrete_input(&self, address: u16) -> Result<bool, ExceptionCode> {
        self.discrete_inputs.get(address as usize).to_result()
    }

    fn read_holding_register(&self, address: u16) -> Result<u16, ExceptionCode> {
        self.holding_registers.get(address as usize).to_result()
    }

    fn read_input_register(&self, address: u16) -> Result<u16, ExceptionCode> {
        self.input_registers.get(address as usize).to_result()
    }


    fn write_single_coil(&mut self, value: Indexed<bool>) -> Result<(), ExceptionCode> {
        tracing::info!(
            "write single coil, index: {} value: {}",
            value.index,
            value.value
        );

        if let Some(coil) = self.coils.get_mut(value.index as usize) {
            *coil = value.value;
            Ok(())
        } else {
            Err(ExceptionCode::IllegalDataAddress)
        }
    }

    fn write_single_register(&mut self, value: Indexed<u16>) -> Result<(), ExceptionCode> {
        tracing::info!(
            "write single register, index: {} value: {}",
            value.index,
            value.value
        );

        if let Some(reg) = self.holding_registers.get_mut(value.index as usize) {
            *reg = value.value;
            Ok(())
        } else {
            Err(ExceptionCode::IllegalDataAddress)
        }
    }

    fn write_multiple_coils(&mut self, values: WriteCoils) -> Result<(), ExceptionCode> {
        tracing::info!("write multiple coils {:?}", values.range);

        let mut result = Ok(());

        for value in values.iterator {
            if let Some(coil) = self.coils.get_mut(value.index as usize) {
                *coil = value.value;
            } else {
                result = Err(ExceptionCode::IllegalDataAddress)
            }
        }

        result
    }

    fn write_multiple_registers(&mut self, values: WriteRegisters) -> Result<(), ExceptionCode> {
        tracing::info!("write multiple registers {:?}", values.range);

        let mut result = Ok(());

        for value in values.iterator {
            if let Some(reg) = self.holding_registers.get_mut(value.index as usize) {
                *reg = value.value;
            } else {
                result = Err(ExceptionCode::IllegalDataAddress)
            }
        }

        result
    }

    fn read_device_info_basic_streaming(
        &self,
        read_dev_id: ReadDeviceCode,
        object_id: Option<u8>,
    ) -> Result<ServerDeviceInfo, ExceptionCode> {

        Ok(ServerDeviceInfo::new(
            DeviceConformityLevel::BasicIdentificationStream,
            object_id,
            self.get_basic_info_response(object_id)
        ))
    }

    fn read_device_info_regular_streaming(&self, read_dev_id: ReadDeviceCode, object_id: Option<u8>) -> Result<ServerDeviceInfo, ExceptionCode> {
        Ok(ServerDeviceInfo::new(
            DeviceConformityLevel::RegularIdentificationStream,
            object_id,
            self.get_regular_info_response(object_id)
        ))
    }

    fn read_device_info_extended_streaming(&self, read_dev_id: ReadDeviceCode, object_id: Option<u8>) -> Result<ServerDeviceInfo, ExceptionCode> {
        Ok(ServerDeviceInfo::new(
            DeviceConformityLevel::ExtendedIdentificationStream,
            object_id,
            self.get_extended_info_response(object_id)
        ))
    }

    fn read_device_info_individual(&self, read_dev_id: ReadDeviceCode, object_id: Option<u8>) -> Result<ServerDeviceInfo, ExceptionCode> {
        todo!()
    }

    fn wrap(self) -> std::sync::Arc<std::sync::Mutex<Box<Self>>>
    where
        Self: Sized,
    {
        std::sync::Arc::new(std::sync::Mutex::new(Box::new(self)))
    }
}
// ANCHOR_END: request_handler

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let transport: &str = match &args[..] {
        [_, x] => x,
        _ => {
            eprintln!("please specify a transport:");
            eprintln!("usage: outstation <transport> (tcp, rtu, tls-ca, tls-self-signed)");
            exit(-1);
        }
    };
    match transport {
        "tcp" => run_tcp().await,
        #[cfg(feature = "serial")]
        "rtu" => run_rtu().await,
        #[cfg(feature = "tls")]
        "tls-ca" => run_tls(get_ca_chain_config()?).await,
        #[cfg(feature = "tls")]
        "tls-self-signed" => run_tls(get_self_signed_config()?).await,
        _ => {
            eprintln!(
                "unknown transport '{transport}', options are (tcp, rtu, tls-ca, tls-self-signed)"
            );
            exit(-1);
        }
    }
}

async fn run_tcp() -> Result<(), Box<dyn std::error::Error>> {
    let (handler, map) = create_handler();

    // ANCHOR: tcp_server_create
    let server = rodbus::server::spawn_tcp_server_task(
        1,
        "127.0.0.1:502".parse()?,
        map,
        AddressFilter::Any,
        AppDecodeLevel::DataValues.into(),
    )
    .await?;
    // ANCHOR_END: tcp_server_create

    run_server(server, handler).await
}

#[cfg(feature = "serial")]
async fn run_rtu() -> Result<(), Box<dyn std::error::Error>> {
    let (handler, map) = create_handler();

    // ANCHOR: rtu_server_create
    let server = rodbus::server::spawn_rtu_server_task(
        "/dev/ttySIM1",
        rodbus::SerialSettings::default(),
        default_retry_strategy(),
        map,
        DecodeLevel::new(
            AppDecodeLevel::DataValues,
            FrameDecodeLevel::Payload,
            PhysDecodeLevel::Data,
        ),
    )?;
    // ANCHOR_END: rtu_server_create

    run_server(server, handler).await
}

#[cfg(feature = "tls")]
async fn run_tls(tls_config: TlsServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let (handler, map) = create_handler();

    // ANCHOR: tls_server_create
    let server = rodbus::server::spawn_tls_server_task_with_authz(
        1,
        "127.0.0.1:802".parse()?,
        map,
        ReadOnlyAuthorizationHandler::create(),
        tls_config,
        AddressFilter::Any,
        AppDecodeLevel::DataValues.into(),
    )
    .await?;
    // ANCHOR_END: tls_server_create

    run_server(server, handler).await
}

fn create_handler() -> (
    ServerHandlerType<SimpleHandler>,
    ServerHandlerMap<SimpleHandler>,
) {
    // ANCHOR: handler_map_create
    let handler =
        SimpleHandler::new(vec![false; 10], vec![false; 10], vec![0; 10], vec![0; 10]).wrap();

    // map unit ids to a handler for processing requests
    let map = ServerHandlerMap::single(UnitId::new(1), handler.clone());
    // ANCHOR_END: handler_map_create

    (handler, map)
}

#[cfg(feature = "tls")]
fn get_self_signed_config() -> Result<TlsServerConfig, Box<dyn std::error::Error>> {
    use std::path::Path;
    // ANCHOR: tls_self_signed_config
    let tls_config = TlsServerConfig::new(
        Path::new("./certs/self_signed/entity1_cert.pem"),
        Path::new("./certs/self_signed/entity2_cert.pem"),
        Path::new("./certs/self_signed/entity2_key.pem"),
        None, // no password
        MinTlsVersion::V1_2,
        CertificateMode::SelfSigned,
    )?;
    // ANCHOR_END: tls_self_signed_config

    Ok(tls_config)
}

#[cfg(feature = "tls")]
fn get_ca_chain_config() -> Result<TlsServerConfig, Box<dyn std::error::Error>> {
    use std::path::Path;
    // ANCHOR: tls_ca_chain_config
    let tls_config = TlsServerConfig::new(
        Path::new("./certs/ca_chain/ca_cert.pem"),
        Path::new("./certs/ca_chain/server_cert.pem"),
        Path::new("./certs/ca_chain/server_key.pem"),
        None, // no password
        MinTlsVersion::V1_2,
        CertificateMode::AuthorityBased,
    )?;
    // ANCHOR_END: tls_ca_chain_config

    Ok(tls_config)
}

async fn run_server(
    mut server: ServerHandle,
    handler: ServerHandlerType<SimpleHandler>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = FramedRead::new(tokio::io::stdin(), LinesCodec::new());
    loop {
        match reader.next().await.unwrap()?.as_str() {
            "x" => return Ok(()),
            "ed" => {
                // enable decoding
                server
                    .set_decode_level(DecodeLevel::new(
                        AppDecodeLevel::DataValues,
                        FrameDecodeLevel::Header,
                        PhysDecodeLevel::Length,
                    ))
                    .await?;
            }
            "dd" => {
                // disable decoding
                server.set_decode_level(DecodeLevel::nothing()).await?;
            }
            "uc" => {
                let mut handler = handler.lock().unwrap();
                for coil in handler.coils_as_mut() {
                    *coil = !*coil;
                }
            }
            "udi" => {
                let mut handler = handler.lock().unwrap();
                for discrete_input in handler.discrete_inputs_as_mut() {
                    *discrete_input = !*discrete_input;
                }
            }
            "uhr" => {
                let mut handler = handler.lock().unwrap();
                for holding_register in handler.holding_registers_as_mut() {
                    *holding_register += 1;
                }
            }
            "uir" => {
                let mut handler = handler.lock().unwrap();
                for input_register in handler.input_registers_as_mut() {
                    *input_register += 1;
                }
            }
            _ => println!("unknown command"),
        }
    }
}
