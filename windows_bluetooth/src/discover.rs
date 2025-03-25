use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use windows::core::HRESULT;
use windows::Win32::Devices::Bluetooth::{BluetoothFindDeviceClose, BluetoothFindFirstDevice, BluetoothFindNextDevice, BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS};
use windows::Win32::Foundation::{ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_NO_MORE_ITEMS, ERROR_OUTOFMEMORY, ERROR_REVISION_MISMATCH, FALSE, HANDLE, SYSTEMTIME, TRUE, WIN32_ERROR};
use windows::core::Error as WinErr;
use crate::bluetooth_device::{BluetoothDevice, MacAddress};

#[inline]
fn err_eq(win_err: &WinErr, win32_err: WIN32_ERROR) -> bool {
    win_err.code() == HRESULT::from_win32(win32_err.0)
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoverDevicesError {
    #[error("Not enough storage is available to complete this operation")]
    OutOfMemory,
    #[error("Unhandled error from windows api: {0}")]
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

    let h_find = match unsafe { BluetoothFindFirstDevice(&search_params, &mut device_info) } {
        Ok(h_find) => h_find,
        Err(err) if err_eq(&err, ERROR_REVISION_MISMATCH) => unreachable!("dwSize should be correct"),
        Err(err) if err_eq(&err, ERROR_INVALID_PARAMETER) => unreachable!("params properly initialized"),
        Err(err) if err_eq(&err, ERROR_OUTOFMEMORY) => return Err(DiscoverDevicesError::OutOfMemory),
        Err(err) if err_eq(&err, ERROR_NO_MORE_ITEMS) => return Ok(found_devices),
        Err(err) => return Err(DiscoverDevicesError::UnhandledWin32ApiErr(err)),
    };

    loop {
        let name = OsString::from_wide(&device_info.szName)
            .to_string_lossy()
            .trim_end_matches('\0')
            .to_owned();
        
        
        found_devices.push(BluetoothDevice {
            // SAFETY: all bit patterns are valid for both fields of the union
            mac_address: MacAddress::from(unsafe { device_info.Address.Anonymous.rgBytes }),
            name: (!name.is_empty()).then_some(name),
            class: device_info.ulClassofDevice,
            connected: device_info.fConnected.as_bool(),
            remembered: device_info.fRemembered.as_bool(),
            authenticated: device_info.fAuthenticated.as_bool(),
            last_seen: into_opt_naive_date(device_info.stLastSeen),
            last_used: into_opt_naive_date(device_info.stLastUsed),
        });

        // SAFETY: handle is valid & device info's state was given by OS, so assumed to be safe; not being concurrently modified
        match unsafe { BluetoothFindNextDevice(h_find, &mut device_info) } {
            Ok(_) => {} // data valid
            Err(err) if err_eq(&err, ERROR_NO_MORE_ITEMS) => break,
            Err(err) => {
                // even if closing the handle errors, only return the first error
                let _ = unsafe { BluetoothFindDeviceClose(h_find) };
                
                match err {
                    err if err_eq(&err, ERROR_INVALID_HANDLE) => unreachable!("handle should be valid"),
                    err if err_eq(&err, ERROR_OUTOFMEMORY) => return Err(DiscoverDevicesError::OutOfMemory),
                    err => return Err(DiscoverDevicesError::UnhandledWin32ApiErr(err)),
                }
            }
        }
    }

    // SAFETY: handle is open and valid, search is complete, not called concurrently
    match unsafe { BluetoothFindDeviceClose(h_find) } {
        Ok(_) => Ok(found_devices),
        Err(err) => Err(DiscoverDevicesError::UnhandledWin32ApiErr(err))
    }
}

fn into_opt_naive_date(system_time: SYSTEMTIME) -> Option<NaiveDateTime> {
    const EPOCH: SYSTEMTIME = SYSTEMTIME { wYear: 1601, wMonth: 1, wDayOfWeek: 1, wDay: 1, wHour: 0, wMinute: 0, wSecond: 0, wMilliseconds: 0 };
    
    match system_time {
        EPOCH => None,
        st => Some(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(st.wYear as _, st.wMonth as _, st.wDay as _)
                .expect("should be valid date"),
            NaiveTime::from_hms_milli_opt(st.wHour as _, st.wMinute as _, st.wSecond as _, st.wMilliseconds as _)
                .expect("should be valid date"),
        ))
    }
}