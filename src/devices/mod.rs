mod saitek_fip_lcd;
mod usb_ids;

use rusb::UsbContext;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, RwLock},
};
use uuid::Uuid;

pub trait ManagedDisplay: Send + Sync {
    fn ready(&self) -> bool;
    fn serial_number(&self) -> String;
    fn device_type_uuid(&self) -> Uuid;
    fn set_image_data(&self, page: u8, data: &[u8; 0x38400]) -> Result<(), ()>;
    fn set_led(&self, page: u8, index: u8, value: bool) -> Result<(), ()>;
    fn clear_image(&self, page: u8) -> Result<(), ()>;
}

pub type UsbDeviceAddress = (u8, u8);

pub struct State {
    #[allow(dead_code)] // prevent dropping
    libusb_context: rusb::Context,
    #[allow(dead_code)] // prevent dropping
	    libusb_hotplug_reg: Mutex<rusb::Registration<rusb::Context>>,
    displays: Arc<RwLock<BTreeMap<UsbDeviceAddress, Arc<dyn ManagedDisplay>>>>,
}

struct UsbHotplugHandler {
    displays: Arc<RwLock<BTreeMap<UsbDeviceAddress, Arc<dyn ManagedDisplay>>>>,
}

pub fn init() -> Result<State, ()> {
    let displays: Arc<RwLock<BTreeMap<UsbDeviceAddress, Arc<dyn ManagedDisplay>>>> =
        Arc::new(RwLock::new(BTreeMap::new()));

    let libusb_context: rusb::Context = rusb::Context::new().expect("Cannot create libusb context");
    let libusb_hotplug_reg = rusb::HotplugBuilder::new()
        .enumerate(true)
        .vendor_id(usb_ids::VID_SAITEK)
        .register(
            &libusb_context,
            Box::new(UsbHotplugHandler {
                displays: displays.clone(),
            }),
        )
        .expect("Cannot register libusb hotplug handler");

    let _libusb_context = libusb_context.clone();
    std::thread::Builder::new()
        .name("libusb events handling thread".to_owned())
        .spawn(move || loop {
            _libusb_context
                .handle_events(None)
                .expect("Cannot handle events (libusb)");
        })
        .expect("Cannot start libusb events handling thread");

    Ok(State {
        libusb_context: libusb_context.clone(),
        libusb_hotplug_reg: Mutex::new(libusb_hotplug_reg), // need to save for object not to be dropped
        displays,
    })
}

impl<T: UsbContext + 'static> rusb::Hotplug<T> for UsbHotplugHandler {
    fn device_arrived(&mut self, device: rusb::Device<T>) {
        let addr = (device.bus_number(), device.address());

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

            let display = crate::devices::saitek_fip_lcd::new_from_libusb(device);

            let mut displays = self.displays.write().expect("State is poisoned");
            displays.insert(addr, display);
        }
    }

    fn device_left(&mut self, device: rusb::Device<T>) {
        let addr = (device.bus_number(), device.address());
        let mut displays = self.displays.write().expect("State is poisoned");
        match displays.remove(&addr) {
            Some(_) => {
                log::info!(
                    "USB device disconnected ({bus_number}-{address})",
                    bus_number = device.bus_number(),
                    address = device.address()
                );
            }
            None => (),
        }
    }
}

impl State {
    pub fn display_addrs(&self) -> Vec<UsbDeviceAddress> {
        let displays = self.displays.read().unwrap();
        displays
            .iter()
            .filter_map(|kv| {
                if !kv.1.ready() {
                    return None;
                }
                Some(kv.0.clone())
            })
            .collect()
    }

    pub fn display_by_addr(&self, addr: &UsbDeviceAddress) -> Option<Arc<dyn ManagedDisplay>> {
        let displays = self.displays.read().unwrap();
        match displays.get(addr) {
            Some(display) => Some(display.clone()),
            None => None,
        }
    }
}
