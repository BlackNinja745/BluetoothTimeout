use std::fmt;
use windows::Win32::Devices::Bluetooth::BLUETOOTH_ADDRESS;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MacAddress([u8;6]);

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(self, f)
    }
}


impl fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(self, f)
    }
}


impl fmt::UpperHex for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}


impl fmt::LowerHex for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl From<[u8; 6]> for MacAddress {
    fn from(octets: [u8; 6]) -> Self {
        MacAddress(octets)
    }
}


impl From<MacAddress> for [u8; 6] {
    fn from(mac_address: MacAddress) -> Self {
        mac_address.0
    }
}

impl From<BLUETOOTH_ADDRESS> for MacAddress {
    fn from(addr: BLUETOOTH_ADDRESS) -> Self {
        // SAFETY: all bit patterns are valid for both fields of the union
        Self(unsafe { addr.Anonymous.rgBytes })
    }
}