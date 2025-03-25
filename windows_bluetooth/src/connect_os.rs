use std::error::Error;
use windows::Win32::Devices::Bluetooth::{BLUETOOTH_MITM_ProtectionNotRequired, BluetoothAuthenticateDeviceEx, AUTHENTICATION_REQUIREMENTS, BLUETOOTH_DEVICE_INFO};
use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_SUCCESS};
use windows::core::Error as WinErr;
use crate::bluetooth_device::MacAddress;


pub fn connect_to_device_os(mac_address: MacAddress) -> Result<(), Box<dyn Error>> {
    unsafe {
        let mut device_info = BLUETOOTH_DEVICE_INFO {
            dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as _,
            .. unsafe { std::mem::zeroed() }
        };

        device_info.Address.Anonymous.rgBytes = mac_address.into();

        let res = BluetoothAuthenticateDeviceEx(
            None,
            None,
            &mut device_info,
            None,
            AUTHENTICATION_REQUIREMENTS(BLUETOOTH_MITM_ProtectionNotRequired.0)
        );

        if res != ERROR_SUCCESS.0 {
            if res == ERROR_ACCESS_DENIED.0 {
                println!("access denied");
            }
            panic!("failed to connect, {res}");
        }
    }

    Ok(())
}

pub async fn uwp_connect(mac_address: &MacAddress) -> Result<(), WinErr> {
    let mac_address = format!("{mac_address:X}");

    todo!()
}