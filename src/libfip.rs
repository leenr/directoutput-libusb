use core::slice;
use std::{sync::{Mutex, Arc}, cmp};

extern crate pretty_env_logger;

mod devices;

type PrgCtx = *mut libc::c_void;
type DevicePtr = u64;
type DWORD = i32;
type HRESULT = i64;

#[allow(non_camel_case_types)]
type Pfn_DirectOutput_EnumerateCallback = unsafe extern "C" fn (device_ptr: DevicePtr, prg_ctx: PrgCtx);
#[allow(non_camel_case_types)]
type Pfn_DirectOutput_DeviceChange = unsafe extern "C" fn (device_ptr: DevicePtr, is_added: bool, prg_ctx: PrgCtx);
#[allow(non_camel_case_types)]
type Pfn_DirectOutput_PageChange = unsafe extern "C" fn (device_ptr: DevicePtr, page: DWORD, is_activated: bool, prg_ctx: PrgCtx);
#[allow(non_camel_case_types)]
type Pfn_DirectOutput_SoftButtonChange = unsafe extern "C" fn (device_ptr: DevicePtr, buttons_state: DWORD, prg_ctx: PrgCtx);

pub const S_OK: HRESULT = 0x00000000;
pub const E_HANDLE: HRESULT = 0x80070006;
pub const E_INVALIDARG: HRESULT = 0x80070057;
pub const E_OUTOFMEMORY: HRESULT = 0x80007000e;
pub const E_NOTIMPL: HRESULT = 0x80004001;
// library errors
pub const E_BUFFERTOOSMALL: HRESULT = 0xff04006f;
pub const E_PAGENOTACTIVE: HRESULT = 0xff040001;

pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

#[cfg(target_arch = "x86")]
macro_rules! directoutputlib_export {
    ($($toks: tt)+) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "stdcall" $($toks)+
    };
}
#[cfg(target_arch = "x86_64")]
macro_rules! directoutputlib_export {
    ($($toks: tt)+) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern $($toks)+
    };
}

static STATE: Mutex<Option<devices::State>> = Mutex::new(None);

directoutputlib_export! {
    fn DirectOutput_Initialize(app_name: *const libc::wchar_t) -> HRESULT {
        pretty_env_logger::init();
        let mut state = STATE.lock().expect("State is poisoned");
        if state.is_none() {
            state.replace(devices::init().expect("Cannot perform library initialization"));
        }
        return S_OK
    }
}

directoutputlib_export! {
    fn DirectOutput_Deinitialize() -> HRESULT {
        let mut state = STATE.lock().expect("State is poisoned");
        if state.is_some() {
            _ = state.take();
        }
        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_RegisterDeviceCallback(callback: Pfn_DirectOutput_DeviceChange, prg_ctx: PrgCtx) -> HRESULT {
        return 0  // TODO
    }
}

directoutputlib_export! {
    fn DirectOutput_Enumerate(callback: Pfn_DirectOutput_EnumerateCallback, prg_ctx: PrgCtx) -> HRESULT {
        let state_guard = STATE.lock().expect("State is poisoned");
        let state: &devices::State = match *state_guard {
            Some(ref x) => x,
            None => {
                log::error!("Library function has been called, but the library is not initialized");
                return E_HANDLE;
            }
        };

        state.display_addrs().iter().for_each(move |addr| {
            let device_ptr = embed_addr(*addr);
            unsafe { callback(device_ptr, prg_ctx); }
        });

        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_RegisterPageCallback(device_ptr: DevicePtr, callback: Pfn_DirectOutput_PageChange, prg_ctx: PrgCtx) -> HRESULT {
        // TODO
        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_RegisterSoftButtonCallback(device_ptr: DevicePtr, callback: Pfn_DirectOutput_SoftButtonChange, prg_ctx: PrgCtx) -> HRESULT {
        // TODO
        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_GetDeviceType(device_ptr: DevicePtr, guid: *mut GUID) -> HRESULT {
        let state_guard = STATE.lock().expect("State is poisoned");
        let state: &devices::State = match *state_guard {
            Some(ref x) => x,
            None => {
                log::error!("Library function has been called, but the library is not initialized");
                return E_HANDLE;
            }
        };

        let display = match get_display(state, &device_ptr) {
            Ok(display) => display,
            Err(err) => return err,
        };

        let uuid = display.device_type_uuid();
        let mut guid = unsafe {&mut *guid };

        let fields = uuid.as_fields();
        (guid.data1, guid.data2, guid.data3, _) = fields;
        guid.data4.copy_from_slice(fields.3);

        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_GetDeviceInstance(device_ptr: DevicePtr, guid: *mut GUID) -> HRESULT {
        // TODO
        return E_NOTIMPL;
    }
}

directoutputlib_export! {
    fn DirectOutput_SetProfile(device_ptr: DevicePtr, debug_profile_name_size: usize, debug_profile_name: *mut libc::wchar_t) -> HRESULT {
        // TODO
        return E_NOTIMPL;
    }
}

directoutputlib_export! {
    fn DirectOutput_AddPage(device_ptr: DevicePtr, page_number: DWORD, page_flags: DWORD) -> HRESULT {
        // TODO
        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_RemovePage(device_ptr: DevicePtr, page_number: DWORD) -> HRESULT {
        // TODO
        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_SetLed(device_ptr: DevicePtr, page_number: DWORD, led_index: DWORD, led_value: DWORD) -> HRESULT {
        let state_guard = STATE.lock().expect("State is poisoned");
        let state: &devices::State = match *state_guard {
            Some(ref x) => x,
            None => {
                log::error!("Library function has been called, but the library is not initialized");
                return E_HANDLE;
            }
        };

        let display = match get_display(state, &device_ptr) {
            Ok(display) => display,
            Err(err) => return err,
        };

        let page = match page_number.try_into() {
            Ok(page) => page,
            Err(_) => return E_INVALIDARG,
        };
        let led_index = match led_index.try_into() {
            Ok(led_index) => led_index,
            Err(_) => return E_INVALIDARG,
        };
        let led_value = match led_value {
            0 => false,
            1 => true,
            _ => return E_INVALIDARG,
        };
        _ = display.set_led(page, led_index, led_value);
        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_SetString(device_ptr: DevicePtr, page_number: DWORD, string_index: DWORD, string_size: DWORD, string: *const libc::wchar_t) -> HRESULT {
        // TODO (seemingly not implemented in FIP)
        return E_NOTIMPL;
    }
}

directoutputlib_export! {
    fn DirectOutput_SetImage(device_ptr: DevicePtr, page_number: DWORD, image_index: DWORD, image_size: DWORD, image: *const u8) -> HRESULT {
        let state_guard = STATE.lock().expect("State is poisoned");
        let state: &devices::State = match *state_guard {
            Some(ref x) => x,
            None => {
                log::error!("Library function has been called, but the library is not initialized");
                return E_HANDLE;
            }
        };

        let display = match get_display(state, &device_ptr) {
            Ok(display) => display,
            Err(err) => return err,
        };

        if image.is_null() {
            return E_INVALIDARG;
        }
        if image_size != 0x38400 {  // TODO
            return E_BUFFERTOOSMALL;
        }
        {
            let image_data = unsafe { slice::from_raw_parts(image, 0x38400) };  //.expect("Null pointer to image data is passed");
            let page = match page_number.try_into() {
                Ok(page) => page,
                Err(_) => return E_INVALIDARG,
            };
            _ = display.set_image_data(page, arrayref::array_ref![image_data, 0, 0x38400]);
        }
        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_SetImageFromFile(device_ptr: DevicePtr, page_number: DWORD, image_index: DWORD, filename_size: DWORD, filename: *const libc::wchar_t) -> HRESULT {
        // TODO
        return S_OK;
    }
}

directoutputlib_export! {
    fn DirectOutput_GetSerialNumber(device_ptr: DevicePtr, res_serial_number: *mut libc::wchar_t, res_serial_number_size: usize) -> HRESULT {
        let state_guard = STATE.lock().expect("State is poisoned");
        let state: &devices::State = match *state_guard {
            Some(ref x) => x,
            None => {
                log::error!("Library function has been called, but the library is not initialized");
                return E_HANDLE;
            }
        };

        let display = match get_display(state, &device_ptr) {
            Ok(display) => display,
            Err(err) => return err,
        };

        let serial_number = display.serial_number();
        let serial_number_wide = widestring::WideCString::from_str(serial_number).expect("Could not convert serial number to wide c string");
        if serial_number_wide.len() > res_serial_number_size {
            return E_BUFFERTOOSMALL;
        }
        let res_serial_number_wide = unsafe { widestring::WideCStr::from_ptr_unchecked_mut(<*mut libc::wchar_t>::cast(res_serial_number), serial_number_wide.len()) };
        unsafe { res_serial_number_wide.as_mut_slice() }.copy_from_slice(&serial_number_wide.as_slice());

        return S_OK;
    }
}


fn extract_addr(device_ptr: &DevicePtr) -> Result<devices::UsbDeviceAddress, HRESULT> {
    if *device_ptr as u16 == 0 || *device_ptr as u16 >= u16::MAX {
        return Err(E_HANDLE);
    }
    let casted: u16 = *device_ptr as u16;
    Ok(((casted >> 8) as u8, (casted & 0xff) as u8))
}

fn embed_addr(device_addr: devices::UsbDeviceAddress) -> DevicePtr {
    ((device_addr.0 as u16) << 8 | (device_addr.1 as u16)) as DevicePtr
}

fn get_display(state: &devices::State, device_ptr: &DevicePtr) -> Result<Arc<dyn devices::ManagedDisplay>, HRESULT> {
    let addr = extract_addr(&device_ptr);
    if addr.is_err() {
        log::error!("Library function has been called with an invalid device pointer");
        return Err(E_HANDLE);
    }
    let display = state.display_by_addr(&addr.unwrap());
    if display.is_none() {
        log::error!("Library function has been called with a device pointer that doesn't exists");
        return Err(E_HANDLE);
    }
    let display = display.unwrap();
    if !display.ready() {
        log::error!("Library function has been called with a device that has been not yet initialized or has been errored");
        return Err(E_HANDLE);
    }
    return Ok(display);
}
