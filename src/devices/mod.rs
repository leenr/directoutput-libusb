mod saitek_fip_lcd;
mod usb_ids;

use rusb::UsbContext;
use std::{collections::BTreeMap, sync::{Arc, Mutex, RwLock}};

pub trait ManagedDisplay: Send + Sync {
    fn ready(&self) -> bool;
    fn serial_number(&self) -> String;
    fn set_image_data(&self, data: &[u8; 0x38400]) -> Result<(), ()>;
}

pub type UsbDeviceAddress = (u8, u8);

pub struct State {
    #[allow(dead_code)] libusb_context: rusb::Context,  // prevent dropping
    #[allow(dead_code)] libusb_hotplug_reg: Mutex<rusb::Registration<rusb::Context>>,  // prevent dropping
    displays: Arc<RwLock<BTreeMap<UsbDeviceAddress, Arc<dyn ManagedDisplay>>>>,
}

struct UsbHotplugHandler {
    displays: Arc<RwLock<BTreeMap<UsbDeviceAddress, Arc<dyn ManagedDisplay>>>>,
}

pub fn init() -> Result<State, ()> {
    let displays: Arc<RwLock<BTreeMap<UsbDeviceAddress, Arc<dyn ManagedDisplay>>>> = Arc::new(RwLock::new(BTreeMap::new()));

    let libusb_context: rusb::Context = rusb::Context::new().unwrap();
    let libusb_hotplug_reg = rusb::HotplugBuilder::new()
                .enumerate(true)
                .vendor_id(usb_ids::VID_SAITEK)
                .register(&libusb_context, Box::new(UsbHotplugHandler { displays: displays.clone() }))
                .unwrap();

    let _libusb_context = libusb_context.clone();
    std::thread::Builder::new()
        .name("libusb thread".to_owned())
        .spawn(move || { loop { _libusb_context.handle_events(None).unwrap(); }; }).unwrap();

    Ok(State {
        libusb_context: libusb_context.clone(),
        libusb_hotplug_reg: Mutex::new(libusb_hotplug_reg),  // need to save for object not to be dropped
        displays
    })
}

impl<T: UsbContext + 'static> rusb::Hotplug<T> for UsbHotplugHandler {
    fn device_arrived(&mut self, device: rusb::Device<T>) {
        let desc = device.device_descriptor();
        if desc.is_err() {
            log::warn!(
                "Could not read USB device {bus_number}-{address} descriptor",
                bus_number = device.bus_number(),
                address = device.address()
            );
            return;
        }

        let desc = desc.unwrap();
        if desc.vendor_id() == usb_ids::VID_SAITEK && desc.product_id() == usb_ids::PID_SAITEK_FIP {
            log::info!(
                "Saitek FIP device detected via USB ({bus_number}-{address})",
                bus_number = device.bus_number(),
                address = device.address()
            );

            self.displays.write().unwrap()
                .insert((device.bus_number(), device.address()), crate::devices::saitek_fip_lcd::new_from_libusb(device));
        }
    }

    fn device_left(&mut self, _: rusb::Device<T>) {
        println!("Device removed");
        todo!()
    }
}

impl State {
    pub fn display_addrs(&self) -> Vec<UsbDeviceAddress> {
        let displays = self.displays.read().unwrap();
        displays.iter().filter_map(|kv| {
            if !kv.1.ready() {
                return None;
            }
            Some(kv.0.clone())
        }).collect()
    }

    pub fn display_by_addr(&self, addr: &UsbDeviceAddress) -> Option<Arc<dyn ManagedDisplay>> {
        let displays = self.displays.read().unwrap();
        match displays.get(addr) {
            Some(display) => Some(display.clone()),
            None => None
        }
    }
}
