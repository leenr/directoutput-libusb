mod saitek_fip_lcd;
mod usb_ids;

use rusb::UsbContext;
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, RwLock, Weak},
};
use uuid::Uuid;

pub trait ManagedDisplay: Send + Sync {
    fn ready(&self) -> bool;
    fn serial_number(&self) -> String;
    fn device_type_uuid(&self) -> Uuid;
    fn set_image_data(&self, page: u8, data: &[u8; 0x38400]) -> Result<(), ()>;
    fn set_led(&self, page: u8, index: u8, value: bool) -> Result<(), ()>;
    fn clear_image(&self, page: u8) -> Result<(), ()>;
    fn save_file(&self, page: u8, file: u8, data: &mut dyn Read) -> Result<(), ()>;
    fn display_file(&self, page: u8, index: u8, file: u8) -> Result<(), ()>;
    fn delete_file(&self, page: u8, file: u8) -> Result<(), ()>;
}

pub type UsbDeviceAddress = (u8, u8);

pub struct State {
    #[allow(dead_code)] // prevent dropping
    libusb_context: rusb::Context,
    #[allow(dead_code)] // prevent dropping
    libusb_hotplug_reg: rusb::Registration<rusb::Context>,
    displays: Arc<RwLock<BTreeMap<UsbDeviceAddress, Arc<dyn ManagedDisplay>>>>,
    display_hotplug_handlers: Arc<RwLock<Vec<Box<dyn Hotplug>>>>,
}

pub trait Hotplug: Send + Sync {
    fn display_arrived(&mut self, device_addr: UsbDeviceAddress);
    fn display_left(&mut self, device_addr: UsbDeviceAddress);
}

struct UsbHotplugHandler {
    displays: Weak<RwLock<BTreeMap<UsbDeviceAddress, Arc<dyn ManagedDisplay>>>>,
    display_hotplug_handlers: Weak<RwLock<Vec<Box<dyn Hotplug>>>>,
}

pub fn init() -> Result<State, ()> {
    let displays: Arc<RwLock<BTreeMap<UsbDeviceAddress, Arc<dyn ManagedDisplay>>>> =
        Arc::new(RwLock::new(BTreeMap::new()));
    let display_hotplug_handlers: Arc<RwLock<Vec<Box<dyn Hotplug>>>> =
        Arc::new(RwLock::new(Vec::with_capacity(1)));

    let libusb_context: rusb::Context = rusb::Context::new().expect("Cannot create libusb context");
    let libusb_hotplug_reg = rusb::HotplugBuilder::new()
        .enumerate(true)
        .vendor_id(usb_ids::VID_SAITEK)
        .register(
            &libusb_context,
            Box::new(UsbHotplugHandler {
                displays: Arc::downgrade(&displays),
                display_hotplug_handlers: Arc::downgrade(&display_hotplug_handlers),
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
        libusb_context,
        libusb_hotplug_reg,
        displays,
        display_hotplug_handlers,
    })
}

impl<T: UsbContext + 'static> rusb::Hotplug<T> for UsbHotplugHandler {
    fn device_arrived(&mut self, device: rusb::Device<T>) {
        let addr = (device.bus_number(), device.address());

        let Ok(desc) = device.device_descriptor() else {
            log::warn!(
                "Could not read USB device {bus_number}-{address} descriptor",
                bus_number = device.bus_number(),
                address = device.address()
            );
            return;
        };

        let display = match (desc.vendor_id(), desc.product_id()) {
            (usb_ids::VID_SAITEK, usb_ids::PID_SAITEK_FIP) => {
                log::info!(
                    "Saitek FIP device detected via USB ({bus_number}-{address})",
                    bus_number = device.bus_number(),
                    address = device.address()
                );
                crate::devices::saitek_fip_lcd::new_from_libusb(device)
            }
            _ => return,
        };

        {
            let Some(ref rc) = self.displays.upgrade() else { return; };
            let mut displays = rc.write().expect("State is poisoned");
            displays.insert(addr, display);
        }
        {
            let Some(ref rc) = self.display_hotplug_handlers.upgrade() else { return; };
            let mut handlers = rc.write().expect("State is poisoned");
            handlers
                .iter_mut()
                .for_each(|handler| handler.display_arrived(addr))
        }
    }

    fn device_left(&mut self, device: rusb::Device<T>) {
        let addr = (device.bus_number(), device.address());
        {
            let Some(ref rc) = self.displays.upgrade() else { return; };
            let mut displays = rc.write().expect("State is poisoned");
            if displays.remove(&addr).is_none() {
                return;
            }
            log::info!(
                "USB device disconnected ({bus_number}-{address})",
                bus_number = device.bus_number(),
                address = device.address()
            );
        }
        {
            let Some(ref rc) = self.display_hotplug_handlers.upgrade() else { return; };
            let mut handlers = rc.write().expect("State is poisoned");
            handlers
                .iter_mut()
                .for_each(|handler| handler.display_left(addr))
        }
    }
}

impl State {
    pub fn add_hotplug_handler(&mut self, hotplug: Box<dyn Hotplug>) {
        self.display_hotplug_handlers.write().unwrap().push(hotplug);
    }

    pub fn display_addrs(&self) -> Vec<UsbDeviceAddress> {
        let displays = self.displays.read().unwrap();
        displays
            .iter()
            .filter_map(|kv| if kv.1.ready() { Some(*kv.0) } else { None })
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
