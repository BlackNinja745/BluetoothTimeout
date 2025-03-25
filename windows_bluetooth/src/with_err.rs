use windows::core::HRESULT;
use windows::Win32::Devices::Bluetooth::{BluetoothGetDeviceInfo, BluetoothRemoveDevice, BLUETOOTH_ADDRESS, BLUETOOTH_DEVICE_INFO};
use windows::Win32::Foundation::{ERROR_GEN_FAILURE, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_NOT_FOUND, ERROR_REVISION_MISMATCH, ERROR_SUCCESS, HANDLE};
use crate::{err_eq, MacAddress};
use windows::core::Error as WinErr;

#[derive(Debug, thiserror::Error)]
#[error("Error from windows::Win32::Devices::Bluetooth::BluetoothRemoveDevice")]
pub enum BluetoothRemoveDeviceErr {
    InvalidMacAddress,
    BluetoothError,
    Other(WinErr),
}

pub unsafe fn bluetooth_remove_device(bluetooth_address: &BLUETOOTH_ADDRESS) -> Result<(), BluetoothRemoveDeviceErr> {
    use BluetoothRemoveDeviceErr as Err;
    
    // SAFETY: caller must ensure no concurrent use? invalid addresses are handled by this function
    match unsafe { HRESULT::from_win32(BluetoothRemoveDevice(bluetooth_address)) } {
        res if err_eq(res, ERROR_INVALID_PARAMETER) => Err(Err::InvalidMacAddress),
        res if err_eq(res, ERROR_NOT_FOUND) => Err(Err::InvalidMacAddress),
        res if err_eq(res, ERROR_GEN_FAILURE) => Err(Err::BluetoothError),
        res if err_eq(res, ERROR_INVALID_HANDLE) => Err(Err::BluetoothError),
        res if err_eq(res, ERROR_SUCCESS) => Ok(()), // successfully removed bluetooth device
        res => Err(Err::Other(WinErr::from_hresult(res))),
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Error from windows::Win32::Devices::Bluetooth::BluetoothGetDeviceInfo")]
pub enum BluetoothGetDeviceInfoErr {
    InvalidMacAddress,
    BluetoothError,
    Other(WinErr),
}

pub unsafe fn bluetooth_get_device_info(h_radio: Option<HANDLE>, device_info: &mut BLUETOOTH_DEVICE_INFO) -> Result<(), BluetoothGetDeviceInfoErr> {
    use BluetoothGetDeviceInfoErr as Err;
    
    let target_addr = MacAddress::from(device_info.Address);

    // SAFETY: dwSize should be set properly by caller, should not be used concurrently
    let h_res = HRESULT::from_win32(unsafe { BluetoothGetDeviceInfo(h_radio, device_info) });

    let res_addr = MacAddress::from(device_info.Address);

    match h_res {
        res if err_eq(res, ERROR_REVISION_MISMATCH) => unreachable!("caller should set dwSize correctly"),
        res if err_eq(res, ERROR_INVALID_PARAMETER) => Err(Err::InvalidMacAddress),
        res if err_eq(res, ERROR_NOT_FOUND) => Err(Err::InvalidMacAddress),
        res if err_eq(res, ERROR_SUCCESS) && target_addr != res_addr => Err(Err::InvalidMacAddress),
        res if err_eq(res, ERROR_GEN_FAILURE) => Err(Err::BluetoothError),
        res if err_eq(res, ERROR_INVALID_HANDLE) => Err(Err::BluetoothError),
        res if err_eq(res, ERROR_SUCCESS) => Ok(()), // successfully got device info
        res => Err(Err::Other(WinErr::from_hresult(res))),
    }
}