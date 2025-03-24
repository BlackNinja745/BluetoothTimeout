use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use anyhow::bail;
use tracing_subscriber::fmt::format::FmtSpan;
use windows::Win32::Devices::Bluetooth::{BluetoothFindDeviceClose, BluetoothFindFirstDevice, BluetoothFindNextDevice, BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS};
use windows::Win32::Foundation::{FALSE, HANDLE, TRUE};

#[tracing::instrument]
fn discover() -> anyhow::Result<()> {
    let search_params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
        dwSize: size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as _,
        fReturnAuthenticated: TRUE,
        fReturnRemembered: TRUE,
        fReturnUnknown: TRUE,
        fReturnConnected: TRUE,
        fIssueInquiry: FALSE,
        cTimeoutMultiplier: 5,
        hRadio: HANDLE(ptr::null_mut()),
    };
    
    let mut device_info = BLUETOOTH_DEVICE_INFO {
        dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as _,
        .. unsafe {std::mem::zeroed() } // SAFETY: all of BLUETOOTH_DEVICE_INFO's fields can and should be zeroed 
    };
    
    // SAFETY: params & device info have been properly initialized; not being concurrently modified
    let Ok(h_find) = ( unsafe { BluetoothFindFirstDevice(&search_params, &mut device_info) } ) else {
        bail!("No bluetooth devices found");
    };
    
    loop {
        let name = OsString::from_wide(&device_info.szName).to_string_lossy().into_owned();
    
        // SAFETY: all bit patterns are valid for both fields of the union
        println!("Device: {name}, Address: {}", unsafe { device_info.Address.Anonymous.ullLong });

        // SAFETY: handle is valid & device info's state was given by OS, so assumed to be safe; not being concurrently modified
        if unsafe { BluetoothFindNextDevice(h_find, &mut device_info) }.is_err() {
            break;
        }
    }
    
    // SAFETY: handle is open and valid, search is complete, not called concurrently
    unsafe { BluetoothFindDeviceClose(h_find)? };
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();
    
    discover()
}