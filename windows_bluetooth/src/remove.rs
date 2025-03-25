use windows::Win32::Devices::Bluetooth::{BLUETOOTH_ADDRESS, BLUETOOTH_ADDRESS_0, BLUETOOTH_DEVICE_INFO};
use crate::{with_err, MacAddress};
use windows::core::Error as WinErr;
use crate::with_err::{BluetoothGetDeviceInfoErr, BluetoothRemoveDeviceErr};

#[derive(Debug, thiserror::Error)]
pub enum RemoveDeviceError {
    #[error("The provided MAC Address was invalid")]
    InvalidMacAddress,
    #[error("Error with bluetooth adapter, bluetooth may be off or no bluetooth adapter exists.")]
    BluetoothError,
    #[error("The device exists, but wasn't remembered in the first place")]
    NotRemembered,
    #[error("Unhandled error from windows api: {0}")]
    UnhandledWin32ApiErr(WinErr),
}

pub fn remove_device(mac_address: MacAddress) -> Result<(), RemoveDeviceError> {
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
    
    // only have to check remembered, since every authenticated device must be remembered
    // therefore, if not remembered then it must also be not authenticated (contraposition)
    if !device_info.fRemembered.as_bool() {
        return Err(RemoveDeviceError::NotRemembered);
    }

    // SAFETY: not being used concurrently
    unsafe { with_err::bluetooth_remove_device(&device_info.Address)? }
    
    Ok(())
}

impl From<BluetoothGetDeviceInfoErr> for RemoveDeviceError {
    fn from(err: BluetoothGetDeviceInfoErr) -> Self {
        match err {
            BluetoothGetDeviceInfoErr::InvalidMacAddress => RemoveDeviceError::InvalidMacAddress,
            BluetoothGetDeviceInfoErr::BluetoothError => RemoveDeviceError::BluetoothError,
            BluetoothGetDeviceInfoErr::Other(err) => RemoveDeviceError::UnhandledWin32ApiErr(err),
        }
    }
}

impl From<BluetoothRemoveDeviceErr> for RemoveDeviceError {
    fn from(err: BluetoothRemoveDeviceErr) -> Self {
        match err {
            BluetoothRemoveDeviceErr::InvalidMacAddress => RemoveDeviceError::InvalidMacAddress,
            BluetoothRemoveDeviceErr::BluetoothError => RemoveDeviceError::BluetoothError,
            BluetoothRemoveDeviceErr::Other(err) => RemoveDeviceError::UnhandledWin32ApiErr(err),
        }
    }
}