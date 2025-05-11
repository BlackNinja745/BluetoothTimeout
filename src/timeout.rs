use std::time::Duration;
use windows_bluetooth::MacAddress;

pub struct Timeout {
    pub mac_address: MacAddress,
    pub duration: Option<Duration>,
    pub remove_on_close: bool,
}

impl Timeout {
    pub fn default_from(mac_address: MacAddress) -> Self {
        Self {
            mac_address,
            duration: None,
            remove_on_close: true,
        }
    }

    pub fn duration_str(&self) -> Option<String> {
        self.duration
            .map(|d|
                humantime::format_duration(Duration::from_secs(d.as_secs())).to_string()
            )
    }

    pub fn duration_str_or_none(&self) -> String {
        self.duration_str()
            .unwrap_or("None".into())
    }
}