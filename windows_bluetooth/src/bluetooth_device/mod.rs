mod mac_address;

pub use mac_address::MacAddress;


#[derive(Debug)]
pub struct BluetoothDevice {
    pub mac_address: MacAddress,
    pub name: Option<String>,
}