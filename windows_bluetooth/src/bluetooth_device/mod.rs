use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use windows::Win32::Devices::Bluetooth::BLUETOOTH_DEVICE_INFO;
use windows::Win32::Foundation::SYSTEMTIME;

mod mac_address;
mod ext;

pub use ext::BluetoothDeviceExt;
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

impl BluetoothDevice {
    pub(crate) unsafe fn from_win32_bluetooth_device(device: &BLUETOOTH_DEVICE_INFO) -> Self {
        let name = OsString::from_wide(&device.szName)
            .to_string_lossy()
            .trim_end_matches('\0')
            .to_owned();
        
        Self {
            // SAFETY: all bit patterns are valid for both fields of the union
            mac_address: MacAddress::from(device.Address),
            name: (!name.is_empty()).then_some(name),
            class: device.ulClassofDevice,
            connected: device.fConnected.as_bool(),
            remembered: device.fRemembered.as_bool(),
            authenticated: device.fAuthenticated.as_bool(),
            last_seen: into_opt_naive_date(device.stLastSeen),
            last_used: into_opt_naive_date(device.stLastUsed),
        }
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