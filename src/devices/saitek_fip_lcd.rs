use std::{sync::{Arc, Weak, RwLock}, time::Duration, mem::MaybeUninit, cell::OnceCell};

use crate::devices::ManagedDisplay;

struct DeviceHandlerWrapper<T: rusb::UsbContext> {
    libusb_handle: rusb::DeviceHandle<T>,
    read_endpoint_address: u8,
    write_endpoint_address: u8,
}

impl<T: rusb::UsbContext> DeviceHandlerWrapper<T> {
    fn read_bulk(&self, buf: &mut [u8], timeout: Duration) -> Result<usize, rusb::Error> {
        log::trace!("reading bulk");
        self.libusb_handle.read_bulk(self.read_endpoint_address, buf, timeout)
    }

    fn write_bulk(&self, buf: &[u8], timeout: Duration) -> Result<usize, rusb::Error> {
        log::trace!("writing bulk");
        self.libusb_handle.write_bulk(self.write_endpoint_address, buf, timeout)
    }
}

struct UsbSaitekFipLcdInt<T: rusb::UsbContext> {
    handle: DeviceHandlerWrapper<T>,
    serial_number: String,
}
struct UsbSaitekFipLcd<T: rusb::UsbContext> {
    libusb_device: rusb::Device<T>,
    int: Arc<RwLock<Option<UsbSaitekFipLcdInt<T>>>>,
}

impl<T: rusb::UsbContext> UsbSaitekFipLcdInt<T> {
    fn new(dev: &UsbSaitekFipLcd<T>) -> UsbSaitekFipLcdInt<T> {
        let mut libusb_handle = dev.libusb_device.open().expect("Cannot open device handle");
        let device_descriptor = dev.libusb_device.device_descriptor().expect("Cannot read device descriptor");

        let config_descriptor = dev.libusb_device.active_config_descriptor().expect("Cannot read device config descriptor");
        let vendor_interface = config_descriptor.interfaces().find(|interface| {
            match interface.descriptors().nth(0) {
                Some(desc) => desc.class_code() == 0xff,
                None => false
            }
        }).expect("Cannot find vendor's interface of the device");
        libusb_handle.claim_interface(vendor_interface.number()).expect("Cannot claim vendor's interface of the device");
        
        let langs = libusb_handle
            .read_languages(std::time::Duration::from_secs(5))
            .expect("Could not read languages from the device");
        let serial_number = libusb_handle
            .read_serial_number_string(langs[0], &device_descriptor, std::time::Duration::from_secs(1))
            .expect("Could not read serial number from the device");

        let read_endpoint_address: OnceCell<u8> = OnceCell::new();
        let write_endpoint_address: OnceCell<u8> = OnceCell::new();
        vendor_interface.descriptors().nth(0).expect("Cannot read device interface descriptors").endpoint_descriptors().for_each(|endpoint| {
            match endpoint.direction() {
                rusb::Direction::In => read_endpoint_address.set(endpoint.address()).expect("Found multiple IN endpoints"),
                rusb::Direction::Out => write_endpoint_address.set(endpoint.address()).expect("Found multiple OUT endpoints"),
            }
        });

        UsbSaitekFipLcdInt {
            handle: DeviceHandlerWrapper {
                libusb_handle,
                read_endpoint_address: *read_endpoint_address.get().expect("Could not find IN endpoint"),
                write_endpoint_address: *write_endpoint_address.get().expect("Could not find OUT endpoint"),
            },
            serial_number,
        }
    }
}

impl<T: rusb::UsbContext> UsbSaitekFipLcd<T> {
    fn read(&self) -> Result<[u8; 44], rusb::Error> {
        let int_guard = self.int.read().expect("Device is poisoned");
        let int = int_guard.as_ref().expect("Device is gone or not initialized yet");
        let mut read_buffer: [u8; 44] = unsafe { #[allow(invalid_value)] MaybeUninit::uninit().assume_init() };
        match int.handle.read_bulk(&mut read_buffer, Duration::from_secs(5)) {
            Ok(size) => if size == 44 { Ok(read_buffer) } else { Err(rusb::Error::Other) },
            Err(err) => Err(err)
        }
    }

    fn write(&self, data: &[u8]) -> Result<usize, rusb::Error> {
        let int_guard = self.int.read().expect("Device is poisoned");
        let int = int_guard.as_ref().expect("Device is gone or not initialized yet");
        int.handle.write_bulk(data, Duration::from_secs(5))
    }

    fn _thread_target(device_weak: Weak<UsbSaitekFipLcd<T>>) {
        {  // don't hold reference for long
            let device = match device_weak.upgrade() {
                Some(device) => device,
                None => return,  // device is dropped
            };
            let _ = device.int.write().expect("Device is poisoned").replace(UsbSaitekFipLcdInt::new(&device));
        }
        loop {
            let device = match device_weak.upgrade() {
                Some(device) => device,
                None => return,  // device is dropped
            };
            match device.read() {
                Ok(data) => {
                    log::debug!("Read data from device: {:?}", data);
                    continue;
                },
                Err(rusb::Error::Timeout) => {
                    continue;
                },
                Err(rusb::Error::NoDevice) => {
                    _ = device.int.write().expect("Device is poisoned").take();  // invalidate the device
                    log::info!("Device is disconnected, invalidating it");
                }
                Err(err) => {
                    _ = device.int.write().expect("Device is poisoned").take();  // invalidate the device
                    log::error!("Could not read from device ({}), invalidating it", err);
                },
            };
        };
    }
}

pub fn new_from_libusb<T: rusb::UsbContext + 'static>(libusb_device: rusb::Device<T>) -> Arc<dyn ManagedDisplay> {
    let device = Arc::new(UsbSaitekFipLcd{
        libusb_device: libusb_device.clone(),
        int: Arc::new(RwLock::new(None))
    });

    let device_ref = Arc::downgrade(&device);
    std::thread::Builder::new()
        .name(format!("Saitek FIP @ {:03}-{:03}", libusb_device.bus_number(), libusb_device.address()))
        .spawn(|| { UsbSaitekFipLcd::_thread_target(device_ref) })
        .expect("Could not start device thread");

    device
}

impl<T: rusb::UsbContext> ManagedDisplay for UsbSaitekFipLcd<T> {
    fn ready(&self) -> bool {
        self.int.read().is_ok_and(|int| { int.is_some() })
    }
    
    fn serial_number(&self) -> String {
        let int_guard = self.int.read().expect("Device is poisoned");
        let int = int_guard.as_ref().expect("Device is gone or not initialized yet");
        int.serial_number.clone()
    }

    fn set_image_data(&self, data: &[u8; 0x38400]) -> Result<(), ()> {
        let res = self.write(&[
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x03, 0x84, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x06,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);
        // TODO: error handling
        let res = self.write(data);
        // TODO: error handling
        Ok(())
    }
}
