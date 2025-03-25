use crate::{connect_to_device_os, remove_device, BluetoothDevice, ConnectToDeviceError, RemoveDeviceError};

pub trait BluetoothDeviceExt {
    fn connect(&self) -> Result<(), ConnectToDeviceError>;
    
    fn remove(&self) -> Result<(), RemoveDeviceError>;
}

impl BluetoothDeviceExt for BluetoothDevice {
    fn connect(&self) -> Result<(), ConnectToDeviceError> {
        connect_to_device_os(self.mac_address)
    }

    fn remove(&self) -> Result<(), RemoveDeviceError> {
        remove_device(self.mac_address)
    }
}