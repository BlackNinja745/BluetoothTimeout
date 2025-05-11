use std::ptr;
use windows::core::HRESULT;
use windows::Win32::Devices::Bluetooth::{BluetoothFindDeviceClose, BluetoothFindFirstDevice, BluetoothFindNextDevice, BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS};
use windows::Win32::Foundation::{ERROR_GEN_FAILURE, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_NO_MORE_ITEMS, ERROR_OUTOFMEMORY, ERROR_REVISION_MISMATCH, FALSE, HANDLE, TRUE, WIN32_ERROR};
use windows::core::Error as WinErr;
use crate::bluetooth_device::BluetoothDevice;

#[inline]
fn err_eq(win_err: &WinErr, win32_err: WIN32_ERROR) -> bool {
    crate::err_eq(win_err.code(), win32_err)
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoverDevicesError {
    #[error("Not enough storage is available to complete this operation")]
    OutOfMemory,
    #[error("Error with bluetooth adapter, bluetooth may be off, or no bluetooth adapter exists.")]
    BluetoothError,
    #[error("Unhandled error from Windows api: {0}")]
    UnhandledWin32ApiErr(WinErr),
}

pub fn discover_devices() -> Result<Vec<BluetoothDevice>, DiscoverDevicesError> {
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
    
    let mut found_devices = Vec::new();

    // SAFETY: not used concurrently & dwSize has been set correctly
    let h_find = match unsafe { BluetoothFindFirstDevice(&search_params, &mut device_info) } {
        Ok(h_find) => h_find,
        Err(err) if err_eq(&err, ERROR_REVISION_MISMATCH) => unreachable!("dwSize should be correct"),
        Err(err) if err_eq(&err, ERROR_INVALID_PARAMETER) => unreachable!("params properly initialized"),
        Err(err) if err_eq(&err, ERROR_OUTOFMEMORY) => return Err(DiscoverDevicesError::OutOfMemory),
        Err(err) if err_eq(&err, ERROR_INVALID_HANDLE) => return Err(DiscoverDevicesError::BluetoothError),
        Err(err) if err_eq(&err, ERROR_GEN_FAILURE) => return Err(DiscoverDevicesError::BluetoothError),
        Err(err) if err_eq(&err, ERROR_NO_MORE_ITEMS) => return Ok(found_devices),
        Err(err) => return Err(DiscoverDevicesError::UnhandledWin32ApiErr(err)),
    };

    loop {
        // SAFETY: device info should be correctly formed
        found_devices.push(unsafe { BluetoothDevice::from_win32_bluetooth_device(&device_info) });

        // SAFETY: handle is valid & device info's state was given by OS, so assumed to be safe; not being concurrently modified
        match unsafe { BluetoothFindNextDevice(h_find, &mut device_info) } {
            Ok(_) => {} // data valid
            Err(err) if err_eq(&err, ERROR_NO_MORE_ITEMS) => break,
            Err(err) => {
                // even if closing the handle errors, only return the first error
                let _ = unsafe { BluetoothFindDeviceClose(h_find) };

                return match err {
                    // since h_find is known to not be null, I'm assuming this can only happen if bluetooth is disconnected between this call and the last call
                    err if err_eq(&err, ERROR_INVALID_HANDLE) => Err(DiscoverDevicesError::BluetoothError),
                    err if err_eq(&err, ERROR_GEN_FAILURE) => Err(DiscoverDevicesError::BluetoothError),
                    err if err_eq(&err, ERROR_OUTOFMEMORY) => Err(DiscoverDevicesError::OutOfMemory),
                    err => Err(DiscoverDevicesError::UnhandledWin32ApiErr(err)),
                }
            }
        }
    }

    // SAFETY: handle is open and valid, search is complete, not called concurrently
    match unsafe { BluetoothFindDeviceClose(h_find) } {
        Ok(_) => Ok(found_devices),
        // since h_find is known to not be null, I'm assuming this can only happen if bluetooth is disconnected between this call and the last call
        Err(err) if err_eq(&err, ERROR_INVALID_HANDLE) => Err(DiscoverDevicesError::BluetoothError),
        Err(err) if err_eq(&err, ERROR_GEN_FAILURE) => Err(DiscoverDevicesError::BluetoothError),
        Err(err) => Err(DiscoverDevicesError::UnhandledWin32ApiErr(err))
    }
}