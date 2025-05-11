use windows::core::HRESULT;
use windows::Win32::Devices::Bluetooth::{BLUETOOTH_MITM_ProtectionNotRequired, BluetoothAuthenticateDeviceEx, BluetoothGetDeviceInfo, BluetoothRemoveDevice, AUTHENTICATION_REQUIREMENTS, BLUETOOTH_ADDRESS, BLUETOOTH_ADDRESS_0, BLUETOOTH_DEVICE_INFO};
use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_CANCELLED, ERROR_GEN_FAILURE, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_NOT_AUTHENTICATED, ERROR_NOT_FOUND, ERROR_NO_MORE_ITEMS, ERROR_REVISION_MISMATCH, ERROR_SUCCESS, HANDLE, WIN32_ERROR};
use windows::core::Error as WinErr;
use crate::bluetooth_device::MacAddress;
use crate::err_eq;
use crate::with_err::{self, BluetoothGetDeviceInfoErr, BluetoothRemoveDeviceErr};

#[derive(Debug, thiserror::Error)]
pub enum ConnectToDeviceError {
    #[error("The provided MAC Address was invalid")]
    InvalidMacAddress,
    #[error("Error with bluetooth adapter, bluetooth may be off, or no bluetooth adapter exists.")]
    BluetoothError,
    #[error("The user wasn't authenticated or canceled the operation")]
    Cancelled,
    #[error("Insufficient permissions or bluetooth device cannot be paired to")]
    AccessDenied,
    #[error("Unhandled error from Windows api: {0}")]
    UnhandledWin32ApiErr(WinErr),
}

// TODO: if the device is paired but isn't connected, we have to remove it first before connecting, which requires the device to be in pairing mode; fix this
pub fn connect_to_device_os(mac_address: MacAddress) -> Result<(), ConnectToDeviceError> {
    let mut device_info = BLUETOOTH_DEVICE_INFO {
        dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as _,
        Address: BLUETOOTH_ADDRESS {
            Anonymous: {
                BLUETOOTH_ADDRESS_0 { rgBytes: mac_address.into() }
            },
        },
        .. unsafe { std::mem::zeroed() } // SAFETY: all of BLUETOOTH_DEVICE_INFO's fields can and should be zeroed
    };

    // SAFETY: device info is properly initialized, not being used concurrently
    unsafe { with_err::bluetooth_get_device_info(None, &mut device_info)?; }

    if device_info.fConnected.as_bool() {
        return Ok(());
    }
    
    if device_info.fAuthenticated.as_bool() {
        // SAFETY: not being used concurrently
        unsafe { with_err::bluetooth_remove_device(&device_info.Address)?; }

        // SAFETY: device info's state was given by OS, so assumed to be safe; not being concurrently modified
        unsafe { with_err::bluetooth_get_device_info(None, &mut device_info)?; }
    }

    // SAFETY: device info's state was given by OS, so assumed to be safe; not being concurrently modified
    let win32err = unsafe { BluetoothAuthenticateDeviceEx(None, None, &mut device_info, None, AUTHENTICATION_REQUIREMENTS(BLUETOOTH_MITM_ProtectionNotRequired.0)) };

    // invalid parameter if null, wait timeout, access denied
    match HRESULT::from_win32(win32err) {
        res if err_eq(res, ERROR_REVISION_MISMATCH) => unreachable!("dwSize should be correct"),
        res if err_eq(res, ERROR_NO_MORE_ITEMS) => unreachable!("device should've been removed if authenticated"),
        res if err_eq(res, ERROR_INVALID_PARAMETER) => Err(ConnectToDeviceError::InvalidMacAddress),
        res if err_eq(res, ERROR_NOT_FOUND) => Err(ConnectToDeviceError::InvalidMacAddress),
        res if err_eq(res, ERROR_INVALID_HANDLE) => Err(ConnectToDeviceError::BluetoothError),
        res if err_eq(res, ERROR_GEN_FAILURE) => Err(ConnectToDeviceError::BluetoothError),
        res if err_eq(res, ERROR_CANCELLED) => Err(ConnectToDeviceError::Cancelled),
        res if err_eq(res, ERROR_NOT_AUTHENTICATED) => Err(ConnectToDeviceError::Cancelled),
        res if err_eq(res, ERROR_ACCESS_DENIED) => Err(ConnectToDeviceError::AccessDenied),
        res if err_eq(res, ERROR_SUCCESS) => Ok(()), // successfully got device info,
        res => Err(ConnectToDeviceError::UnhandledWin32ApiErr(WinErr::from_hresult(res))),
    }
}

impl From<BluetoothGetDeviceInfoErr> for ConnectToDeviceError {
    fn from(err: BluetoothGetDeviceInfoErr) -> Self {
        match err {
            BluetoothGetDeviceInfoErr::InvalidMacAddress => ConnectToDeviceError::InvalidMacAddress,
            BluetoothGetDeviceInfoErr::BluetoothError => ConnectToDeviceError::BluetoothError,
            BluetoothGetDeviceInfoErr::Other(err) => ConnectToDeviceError::UnhandledWin32ApiErr(err),
        }
    }
}

impl From<BluetoothRemoveDeviceErr> for ConnectToDeviceError {
    fn from(err: BluetoothRemoveDeviceErr) -> Self {
        match err {
            BluetoothRemoveDeviceErr::InvalidMacAddress => ConnectToDeviceError::InvalidMacAddress,
            BluetoothRemoveDeviceErr::BluetoothError => ConnectToDeviceError::BluetoothError,
            BluetoothRemoveDeviceErr::Other(err) => ConnectToDeviceError::UnhandledWin32ApiErr(err),
        }
    }
}