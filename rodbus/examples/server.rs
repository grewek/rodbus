use std::process::exit;

use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};

use rodbus::server::*;
use rodbus::*;

struct SimpleHandler {
    coils: Vec<bool>,
    discrete_inputs: Vec<bool>,
    holding_registers: Vec<u16>,
    input_registers: Vec<u16>,

    basic_info: [String; 3],
    regular_keys: [String; 4],
    extended_values: [String; 4],

    basic_streaming_response_data: Vec<u8>,
    regular_streaming_response_data: Vec<u8>,

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

            basic_info: ["Example Vendor".to_string(), "Little Dictionary".to_string(), "0.1.0".to_string()],
            regular_keys: ["0x8A".to_string(), "0x8B".to_string(), "0x8C".to_string(), "0x8D".to_string()],
            extended_values: ["This is the value for key 0x8A".to_string(), "Value for 0x8B which is a bit longer than your usual method to test if the behavior of sending responses is correct and works, and will it work over a different client as well ?".to_string(), "Another value for 0x8C".to_string(), "Last but not least the value for 0x8D".to_string()],
            basic_streaming_response_data: vec![0u8; u8::MAX as usize],
            regular_streaming_response_data: vec![0u8; u8::MAX as usize],
        }.create_basic_streaming_response().create_regular_streaming_response()
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

    fn create_basic_streaming_response(mut self) -> Self{
        let mut answer = Vec::with_capacity(u8::MAX as usize);
        let data = self.basic_info.as_slice().iter().enumerate();

        //NOTE: We write out how many objects our answer contains
        answer.push(data.len() as u8);

        for (index, string_data) in data {
            //NOTE: Write out the Object ID
            answer.push(index as u8);

            assert!(string_data.len() <= u8::MAX as usize);
            //NOTE: Write the Object length
            answer.push(string_data.len() as u8);

            //NOTE: Write the data itself !
            answer.extend_from_slice(string_data.as_bytes());
        }

        self.basic_streaming_response_data = answer;

        self
    }

    fn create_regular_streaming_response(mut self) -> Self {
        let mut answer = Vec::with_capacity(u8::MAX as usize);
        let data = self.regular_keys.as_slice().iter().enumerate();

        answer.push(data.len() as u8);

        let base = 0x03;

        for (index, string_data) in data {

            answer.push(base + index as u8);

            assert!(string_data.len() <= u8::MAX as usize);
            //NOTE: Write the Object length
            answer.push(string_data.len() as u8);

            //NOTE: Write the data itself !
            answer.extend_from_slice(string_data.as_bytes());
        }

        self.regular_streaming_response_data = answer;

        self
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

    fn read_device_info(
        &self,
        mei_code: MeiCode,
        read_dev_id: ReadDeviceCode,
        object_id: Option<u8>,
    ) -> Result<ServerDeviceInfo, ExceptionCode> {
        let mut device_info = DeviceInfo::new(
        match read_dev_id {
            ReadDeviceCode::BasicStreaming =>
                return Ok(ServerDeviceInfo::new(
                    DeviceConformityLevel::ExtendedIdentificationIndividual,
                    None,
                    &self.basic_streaming_response_data,
                )),
            ReadDeviceCode::RegularStreaming => return Ok(ServerDeviceInfo::new(
                DeviceConformityLevel::ExtendedIdentificationIndividual,
                None,
                &self.regular_streaming_response_data
            )),
            _ => todo!(),
            /*ReadDeviceCode::ExtendedStreaming => {
                let mut writecursor = 0;

                for (index, string_data) in self.basic_info.as_slice().iter().enumerate() {
                    self.response_data[writecursor] = index as u8;
                    writecursor += 1;
                    assert!(string_data.len() <= u8::MAX as usize);
                    self.response_data[writecursor] = (string_data.len() as u8);
                    writecursor += 1;

                    for byte in string_data.as_bytes() {
                        self.response_data[writecursor];
                        writecursor += 1;
                    }
                }
            },
            ReadDeviceCode::Specific if object_id.is_some() => {
                let index = object_id.unwrap() as usize;

                let message = match object_id.unwrap() {
                    0x8A => &self.extended_values[0],
                    0x8B => &self.extended_values[1],
                    0x8C => &self.extended_values[2],
                    0x8D => &self.extended_values[3],
                    _ => unreachable!(),
                };
                device_info.storage = vec![InfoObject::new(
                    //ReadDeviceCode::Specific,
                    index as u8,
                    //message.len() as u8,
                    message.as_bytes(),
                )];
                self.extended_values.as_slice()
            },*/
        };

        /*if read_dev_id == ReadDeviceCode::Specific && object_id.is_some() {
            device_info.number_objects = 1;
            let index = object_id.unwrap() as usize;

            let message = match object_id.unwrap() {
                0x8A => &self.extended_values[0],
                0x8B => &self.extended_values[1],
                0x8C => &self.extended_values[2],
                0x8D => &self.extended_values[3],
                _ => unreachable!(),
            };
            device_info.storage = vec![InfoObject::new(
                //ReadDeviceCode::Specific,
                index as u8,
                //message.len() as u8,
                message.as_bytes(),
            )];

            return Ok(device_info);
        } else {
            device_info.number_objects = response_data.len() as u8;
            device_info.storage = vec![];

            for (idx, info_string) in response_data.iter().enumerate() {
                let obj = RawModbusInfoObject::new(
                    read_dev_id,
                    idx as u8,
                    info_string.len() as u8,
                    info_string.as_bytes(),
                );
                device_info.storage.push(obj);
            }
        }*/

        //TODO(Kay): This is just a hack for getting things to work right now !
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
