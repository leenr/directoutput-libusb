use std::{
    cell::OnceCell,
    mem,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use uuid::{self, Uuid};
use zerocopy::{AsBytes, FromBytes, Unaligned};

use crate::devices::ManagedDisplay;

struct DeviceHandlerWrapper<T: rusb::UsbContext> {
    libusb_handle: rusb::DeviceHandle<T>,
    read_endpoint_address: u8,
    write_endpoint_address: u8,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum Request {
    SaveFile = 0x03,
    SetImageFile = 0x04, // + DisplayFile
    SetImage = 0x06,
    StartServer = 0x09,
    ClearImage = 0x13,
    SetLed = 0x18,
}

impl<T: rusb::UsbContext> DeviceHandlerWrapper<T> {
    fn read_bulk(&self, buf: &mut [u8], timeout: Duration) -> Result<usize, rusb::Error> {
        log::trace!("reading bulk");
        self.libusb_handle
            .read_bulk(self.read_endpoint_address, buf, timeout)
    }

    fn write_bulk(&self, buf: &[u8], timeout: Duration) -> Result<usize, rusb::Error> {
        log::trace!("writing bulk");
        self.libusb_handle
            .write_bulk(self.write_endpoint_address, buf, timeout)
    }
}

struct UsbSaitekFipLcdInt<T: rusb::UsbContext> {
    handle: DeviceHandlerWrapper<T>,
    serial_number: String,
    device_type_uuid: Uuid,
}
struct UsbSaitekFipLcd<T: rusb::UsbContext> {
    libusb_device: rusb::Device<T>,
    int: Arc<Mutex<Option<UsbSaitekFipLcdInt<T>>>>,
}

impl<T: rusb::UsbContext> UsbSaitekFipLcdInt<T> {
    fn new(dev: &UsbSaitekFipLcd<T>) -> UsbSaitekFipLcdInt<T> {
        let mut libusb_handle = dev.libusb_device.open().expect("Cannot open device handle");
        let device_descriptor = dev
            .libusb_device
            .device_descriptor()
            .expect("Cannot read device descriptor");

        let config_descriptor = dev
            .libusb_device
            .active_config_descriptor()
            .expect("Cannot read device config descriptor");
        let vendor_interface = config_descriptor
            .interfaces()
            .find(|interface| match interface.descriptors().nth(0) {
                Some(desc) => desc.class_code() == 0xff,
                None => false,
            })
            .expect("Cannot find vendor's interface of the device");
        libusb_handle
            .claim_interface(vendor_interface.number())
            .expect("Cannot claim vendor's interface of the device");

        let serial_number = {
            let langs = libusb_handle
                .read_languages(std::time::Duration::from_secs(5))
                .expect("Could not read languages from the device");
            libusb_handle
                .read_serial_number_string(
                    langs[0],
                    &device_descriptor,
                    std::time::Duration::from_secs(1),
                )
                .expect("Could not read serial number from the device")
        };

        let device_type_uuid = uuid::uuid!("3E083CD8-6A37-4A58-80A8-3D6A2C07513E"); // seems like that is just a harcoded uuid with no way of retreiving (but I may be wrong)

        let read_endpoint_address: OnceCell<u8> = OnceCell::new();
        let write_endpoint_address: OnceCell<u8> = OnceCell::new();
        vendor_interface
            .descriptors()
            .nth(0)
            .expect("Cannot read device interface descriptors")
            .endpoint_descriptors()
            .for_each(|endpoint| match endpoint.direction() {
                rusb::Direction::In => read_endpoint_address
                    .set(endpoint.address())
                    .expect("Found multiple IN endpoints"),
                rusb::Direction::Out => write_endpoint_address
                    .set(endpoint.address())
                    .expect("Found multiple OUT endpoints"),
            });

        log::info!(
            "Saitek FIP device initialized (serial number: {:?}, type uuid: {:?})",
            serial_number,
            device_type_uuid
        );

        UsbSaitekFipLcdInt {
            handle: DeviceHandlerWrapper {
                libusb_handle,
                read_endpoint_address: *read_endpoint_address
                    .get()
                    .expect("Could not find IN endpoint"),
                write_endpoint_address: *write_endpoint_address
                    .get()
                    .expect("Could not find OUT endpoint"),
            },
            serial_number,
            device_type_uuid,
        }
    }
}

type BEU32 = zerocopy::byteorder::U32<zerocopy::byteorder::BigEndian>;

#[derive(Debug, FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct ControlPacket {
    server_id: BEU32,
    page: BEU32,
    data_size: BEU32,
    header_info: BEU32,
    header_error: BEU32,
    request: BEU32,
    led_page: BEU32,
    led_index: BEU32,
    led_value: BEU32,
    request_info: BEU32,
    request_error: BEU32,
}
impl ControlPacket {
    #[inline(always)]
    fn server_id(&self) -> u32 {
        self.server_id.get()
    }
    #[inline(always)]
    fn set_server_id(&mut self, value: u32) {
        self.server_id = value.into()
    }

    #[inline(always)]
    fn page(&self) -> u8 {
        self.page.get().try_into().expect("Got invalid `page`")
    }
    #[inline(always)]
    fn set_page(&mut self, value: u8) {
        self.page = <u32>::into(value.into())
    }

    #[inline(always)]
    fn data_size(&self) -> usize {
        self.data_size.get() as usize
    }
    #[inline(always)]
    fn set_data_size(&mut self, value: usize) {
        self.data_size = (value as u32).into()
    }

    #[inline(always)]
    fn header_info(&self) -> u32 {
        self.header_info.get()
    }
    #[inline(always)]
    fn set_header_info(&mut self, value: u32) {
        self.header_info = value.into()
    }

    #[inline(always)]
    fn header_error(&self) -> u32 {
        self.header_error.get()
    }
    #[inline(always)]
    fn set_header_error(&mut self, value: u32) {
        self.header_error = value.into()
    }

    #[inline(always)]
    fn request(&self) -> Result<Request, TryFromPrimitiveError<Request>> {
        Request::try_from(self.request.get())
    }
    #[inline(always)]
    fn set_request(&mut self, value: Request) {
        self.request = <u32>::into(value.into())
    }

    #[inline(always)]
    fn led_page(&self) -> u8 {
        self.led_page
            .get()
            .try_into()
            .expect("Got invalid `led_page`")
    }
    #[inline(always)]
    fn set_led_page(&mut self, value: u8) {
        self.led_page = <u32>::into(value.into())
    }

    #[inline(always)]
    fn led_index(&self) -> u8 {
        self.led_index
            .get()
            .try_into()
            .expect("Got invalid `led_index`")
    }
    #[inline(always)]
    fn set_led_index(&mut self, value: u8) {
        self.led_index = <u32>::into(value.into())
    }

    #[inline(always)]
    fn led_value(&self) -> bool {
        match self.led_value.get() {
            0 => false,
            1 => true,
            _ => panic!("Got invalid `led_value`"),
        }
    }
    #[inline(always)]
    fn set_led_value(&mut self, value: bool) {
        self.led_value = match value {
            false => 0.into(),
            true => 1.into(),
        }
    }

    #[inline(always)]
    fn request_info(&self) -> u32 {
        self.request_info.get()
    }
    #[inline(always)]
    fn set_request_info(&mut self, value: u32) {
        self.request_info = value.into()
    }

    #[inline(always)]
    fn request_error(&self) -> u32 {
        self.request_error.get()
    }
    #[inline(always)]
    fn set_request_error(&mut self, value: u32) {
        self.request_error = value.into()
    }

    fn has_error(&self) -> bool {
        self.header_info() > 0 || self.request_info() > 0
    }

    fn new(request: Request) -> ControlPacket {
        ControlPacket {
            server_id: 0.into(),
            page: 0.into(),
            data_size: 0.into(),
            header_info: 0.into(),
            header_error: 0.into(),
            request: <u32>::into(request.into()),
            led_page: 0.into(),
            led_index: 0.into(),
            led_value: 0.into(),
            request_info: 0.into(),
            request_error: 0.into(),
        }
    }
}

//#[derive(Debug)]
//struct ProtocolError {
//    header_info: u32,
//    header_error: u32,
//    request_info: u32,
//    request_error: u32,
//}
//
//#[derive(Debug)]
//enum TransmitError {
//    Rusb(rusb::Error),
//    Protocol(ProtocolError),
//}
//
//impl fmt::Display for TransmitError {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        match *self {
//            TransmitError::Rusb(..) => write!(f, "rusb/libusb error"),
//            TransmitError::Protocol(..) => write!(f, "Error received via protocol"),
//        }
//    }
//}
//
//impl error::Error for TransmitError {
//    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//        match *self {
//            TransmitError::Rusb(ref e) => Some(e),
//            TransmitError::Protocol(..) => None,
//        }
//    }
//}
//
//impl From<rusb::Error> for TransmitError {
//    fn from(err: rusb::Error) -> TransmitError {
//        TransmitError::Rusb(err)
//    }
//}

impl<T: rusb::UsbContext> UsbSaitekFipLcdInt<T> {
    fn read(&self) -> Result<(ControlPacket, Option<Vec<u8>>), rusb::Error> {
        let control_packet_bytes = {
            let mut buffer: [u8; 44] = unsafe {
                #[allow(invalid_value)]
                mem::MaybeUninit::uninit().assume_init()
            };
            match self.handle.read_bulk(&mut buffer, Duration::from_secs(5)) {
                Ok(read_size) => {
                    if read_size == 44 {
                        Ok(buffer)
                    } else {
                        Err(rusb::Error::Other)
                    }
                }
                Err(err) => Err(err),
            }
        }?;
        let control_packet =
            ControlPacket::read_from(&control_packet_bytes as &[u8]).expect("Something strange");
        log::debug!("Read control packet from device: {:?}", control_packet);

        if control_packet.data_size() == 0 {
            Ok((control_packet, None))
        } else {
            if control_packet.data_size() >= 512 * 1024 {
                panic!("Too big data size");
            }
            let mut vec = Vec::with_capacity(control_packet.data_size());
            let data = match self.handle.read_bulk(&mut vec, Duration::from_secs(5)) {
                Ok(read_size) => {
                    if read_size == control_packet.data_size() {
                        Ok(vec)
                    } else {
                        Err(rusb::Error::Other)
                    }
                }
                Err(err) => Err(err),
            }?;
            Ok((control_packet, Some(data)))
        }
    }

    fn write(&self, control_packet: ControlPacket, data: Option<&[u8]>) -> Result<(), rusb::Error> {
        let mut buffer: [u8; 44] = unsafe {
            #[allow(invalid_value)]
            mem::MaybeUninit::uninit().assume_init()
        };
        ControlPacket::write_to(&control_packet, buffer.as_mut_slice()).expect("Something strange");
        log::debug!("Write control packet to device: {:?}", control_packet);
        _ = self.handle.write_bulk(&buffer, Duration::from_secs(5))?;

        if data.unwrap_or(&[]).len() != control_packet.data_size() {
            panic!("Data size is not the same as the data size in the packet");
        }
        if data.is_some() {
            let data = data.unwrap();
            if !data.is_empty() {
                _ = self.handle.write_bulk(&data, Duration::from_secs(5))?;
            }
        }
        Ok(())
    }

    fn transmit(
        &self,
        control_packet: ControlPacket,
        data: Option<&[u8]>,
    ) -> Result<(ControlPacket, Option<Vec<u8>>), rusb::Error> {
        self.write(control_packet, data)?;
        self.read()
    }
}

impl<T: rusb::UsbContext> UsbSaitekFipLcd<T> {
    fn transmit(
        &self,
        control_packet: ControlPacket,
        data: Option<&[u8]>,
    ) -> Result<(ControlPacket, Option<Vec<u8>>), rusb::Error> {
        let int_guard = self.int.lock().expect("Device is poisoned");
        let int = int_guard
            .as_ref()
            .expect("Device is gone or not initialized yet");
        return int.transmit(control_packet, data);
    }

    fn _thread_target(device_weak: Weak<UsbSaitekFipLcd<T>>) {
        {
            // don't hold reference for long
            let device = match device_weak.upgrade() {
                Some(device) => device,
                None => return, // device is dropped
            };
            {
                let device_int_guard = device.int.lock();
                _ = device_int_guard
                    .expect("Device is poisoned")
                    .replace(UsbSaitekFipLcdInt::new(&device));
            };
            _ = device.clear_image(0);
        }
    }
}

pub fn new_from_libusb<T: rusb::UsbContext + 'static>(
    libusb_device: rusb::Device<T>,
) -> Arc<dyn ManagedDisplay> {
    let device = Arc::new(UsbSaitekFipLcd {
        libusb_device: libusb_device.clone(),
        int: Arc::new(Mutex::new(None)),
    });

    let device_ref = Arc::downgrade(&device);
    std::thread::Builder::new()
        .name(format!(
            "Saitek FIP @ {:03}-{:03}",
            libusb_device.bus_number(),
            libusb_device.address()
        ))
        .spawn(|| UsbSaitekFipLcd::_thread_target(device_ref))
        .expect("Could not start device thread");

    device
}

impl<T: rusb::UsbContext> ManagedDisplay for UsbSaitekFipLcd<T> {
    fn ready(&self) -> bool {
        self.int.lock().is_ok_and(|int| int.is_some())
    }

    fn serial_number(&self) -> String {
        let int_guard = self.int.lock().expect("Device is poisoned");
        let int = int_guard
            .as_ref()
            .expect("Device is gone or not initialized yet");
        int.serial_number.clone()
    }

    fn device_type_uuid(&self) -> Uuid {
        let int_guard = self.int.lock().expect("Device is poisoned");
        let int = int_guard
            .as_ref()
            .expect("Device is gone or not initialized yet");
        int.device_type_uuid
    }

    fn set_image_data(&self, page: u8, data: &[u8; 0x38400]) -> Result<(), ()> {
        let mut packet = ControlPacket::new(Request::SetImage);
        packet.set_page(page);
        packet.set_data_size(data.len());
        match self.transmit(packet, Some(data)) {
            Ok((packet, _)) => match packet.has_error() {
                false => Ok(()),
                true => Err(()), // TODO
            },
            Err(_) => Err(()), // TODO
        }
    }

    fn set_led(&self, page: u8, index: u8, value: bool) -> Result<(), ()> {
        let mut packet = ControlPacket::new(Request::SetLed);
        packet.set_led_page(page);
        packet.set_led_index(index);
        packet.set_led_value(value);
        match self.transmit(packet, None) {
            Ok((packet, _)) => match packet.has_error() {
                false => Ok(()),
                true => Err(()), // TODO
            },
            Err(_) => Err(()), // TODO
        }
    }

    fn clear_image(&self, page: u8) -> Result<(), ()> {
        let mut packet = ControlPacket::new(Request::ClearImage);
        packet.set_page(page);
        match self.transmit(packet, None) {
            Ok((packet, _)) => match packet.has_error() {
                false => Ok(()),
                true => Err(()), // TODO
            },
            Err(_) => Err(()), // TODO
        }
    }
}
