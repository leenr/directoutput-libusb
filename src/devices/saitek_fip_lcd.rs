use std::{sync::Arc, time::Duration, mem::MaybeUninit};

use crate::devices::ManagedDisplay;

struct UsbSaitekFipLcdInt<T: rusb::UsbContext> {
    libusb_handle: rusb::DeviceHandle<T>,
    serial_number: String,
    read_endpoint_number: u8,
    write_endpoint_number: u8,
}
struct UsbSaitekFipLcd<T: rusb::UsbContext> {
    libusb_device: rusb::Device<T>,
    int: std::sync::OnceLock<UsbSaitekFipLcdInt<T>>,
}

impl<T: rusb::UsbContext> UsbSaitekFipLcdInt<T> {
    fn new(dev: &UsbSaitekFipLcd<T>) -> UsbSaitekFipLcdInt<T> {
        let mut libusb_handle = dev.libusb_device.open().unwrap();
        let device_descriptor = dev.libusb_device.device_descriptor().unwrap();

        let config_descriptor = dev.libusb_device.active_config_descriptor().unwrap();
        let vendor_interface = config_descriptor.interfaces().find(|interface| {
            interface.descriptors().nth(0).unwrap().class_code() == 0xff
        }).unwrap();
        libusb_handle.claim_interface(vendor_interface.number()).unwrap();
        
        let langs = libusb_handle
            .read_languages(std::time::Duration::from_secs(5))
            .unwrap();
        let serial_number = libusb_handle
            .read_serial_number_string(langs[0], &device_descriptor, std::time::Duration::from_secs(1))
            .unwrap();

        let mut read_endpoint_number: Option<u8> = None;
        let mut write_endpoint_number: Option<u8> = None;
        vendor_interface.descriptors().nth(0).unwrap().endpoint_descriptors().for_each(|endpoint| {
            match endpoint.direction() {
                rusb::Direction::In => read_endpoint_number = Some(endpoint.number()),
                rusb::Direction::Out => write_endpoint_number = Some(endpoint.number())
            }
        });

        UsbSaitekFipLcdInt {
            libusb_handle,
            serial_number,
            read_endpoint_number: read_endpoint_number.unwrap(),
            write_endpoint_number: write_endpoint_number.unwrap()
        }
    }
}

impl<T: rusb::UsbContext> UsbSaitekFipLcd<T> {
    fn get_int(&self) -> &UsbSaitekFipLcdInt<T> {
        return self.int.get().unwrap();
    }

    fn read(&self) -> Result<[u8; 44], rusb::Error> {
        let mut read_buffer: [u8; 44] = unsafe { #[allow(invalid_value)] MaybeUninit::uninit().assume_init() };
        let res = self.get_int().libusb_handle.read_bulk(
            self.get_int().read_endpoint_number,
            &mut read_buffer,
            Duration::from_secs(5)
        );
        match res {
            Ok(size) => if size >= 44 { Ok(read_buffer) } else { Err(rusb::Error::Other) },
            Err(err) => Err(err)
        }
    }
    
    fn write(&self, data: &[u8]) -> Result<usize, rusb::Error> {
        self.get_int().libusb_handle.write_bulk(self.get_int().write_endpoint_number, data, Duration::from_secs(5))
    }
}
    
pub fn new_from_libusb<T: rusb::UsbContext + 'static>(libusb_device: rusb::Device<T>) -> Arc<dyn ManagedDisplay> {
    let device = Arc::new(UsbSaitekFipLcd{libusb_device, int: std::sync::OnceLock::new()});

    let device_ref = Arc::downgrade(&device);
    std::thread::spawn(move || {
        {  // don't hold reference for long
            let device = device_ref.upgrade().unwrap();
            let _ = device.int.set(UsbSaitekFipLcdInt::new(&device));
        }
        loop {
            let device = device_ref.upgrade().unwrap();
            let _ = device.read();
        }
    });

    device
}

impl<T: rusb::UsbContext> ManagedDisplay for UsbSaitekFipLcd<T> {
    fn ready(&self) -> bool {
        self.int.get().is_some()
    }
    
    fn serial_number(&self) -> String {
        self.get_int().serial_number.clone()
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
