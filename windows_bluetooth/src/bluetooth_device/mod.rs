mod mac_address;

use chrono::NaiveDateTime;
pub use mac_address::MacAddress;


// TODO: don't make fields public; getter?
#[derive(Debug)]
pub struct BluetoothDevice {
    pub mac_address: MacAddress,
    pub name: Option<String>,
    pub class: u32,
    pub connected: bool,
    pub remembered: bool,
    pub authenticated: bool,
    pub last_seen: Option<NaiveDateTime>,
    pub last_used: Option<NaiveDateTime>,
}