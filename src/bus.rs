use core::fmt::Display;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use heapless::String;
use numtoa::NumToA;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetDataTrafficSpeed {
    pub up: u32,
    pub down: u32,
}

impl Default for NetDataTrafficSpeed {
    fn default() -> Self {
        Self { up: 0, down: 0 }
    }
}

impl Display for NetDataTrafficSpeed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "up: {}B, down: {}B", self.up, self.down)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WiFiConnectStatus {
    Connecting,
    Connected,
    Failed,
}

impl Display for WiFiConnectStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NetSpeed {
    pub direct_up_bps: u64,
    pub direct_down_bps: u64,
    pub proxy_up_bps: u64,
    pub proxy_down_bps: u64,
    pub bypass_up_bps: u64,
    pub bypass_down_bps: u64,
}

impl Default for NetSpeed {
    fn default() -> Self {
        Self {
            direct_up_bps: 0,
            direct_down_bps: 0,
            proxy_up_bps: 0,
            proxy_down_bps: 0,
            bypass_up_bps: 0,
            bypass_down_bps: 0,
        }
    }
}

impl Display for NetSpeed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "direct_bps: ({}, {}), proxy_bps: ({}, {}), bypass_bps: ({}, {})",
            self.direct_up_bps,
            self.direct_down_bps,
            self.proxy_up_bps,
            self.proxy_down_bps,
            self.bypass_up_bps,
            self.bypass_down_bps
        )
    }
}

macro_rules! define_net_speed_to_human_readable_str_method {
    ($method_name:ident, $field_name:ident) => {
        pub fn $method_name<'f, const STR_SIZE: usize, const BUFF_SIZE: usize>(
            &self,
            str: &mut String<STR_SIZE>,
            str_buff: &mut [u8; BUFF_SIZE],
        ) {
            Self::to_human_readable_str(self.$field_name, str, str_buff);
        }
    };
}
impl NetSpeed {
    fn to_human_readable<'f>(mut value: u64) -> (u64, &'f str) {
        let mut unit = "B";

        if value > (1024 + 1024) * 1024 * 1024 * 1024 {
            value /= 1024 * 1024 * 1024 * 1024;
            unit = "T";
        } else if value > (1024 + 1024) * 1024 * 1024 {
            value /= 1024 * 1024 * 1024;
            unit = "G";
        } else if value > (1024 + 1024) * 1024 {
            value /= 1024 * 1024;
            unit = "M";
        } else if value > (1024 + 1024) {
            value /= 1024;
            unit = "K";
        }

        (value, unit)
    }

    fn to_human_readable_str<'f, const STR_SIZE: usize, const BUFF_SIZE: usize>(
        value: u64,
        str: &mut String<STR_SIZE>,
        str_buff: &mut [u8; BUFF_SIZE],
    ) {
        let (num, unit) = Self::to_human_readable(value);

        str.clear();
        str.push_str(num.numtoa_str(10, str_buff)).unwrap();
        str.push_str(unit).unwrap();
    }

    define_net_speed_to_human_readable_str_method!(get_direct_up_bps_str, direct_up_bps);
    define_net_speed_to_human_readable_str_method!(get_direct_down_bps_str, direct_down_bps);
    define_net_speed_to_human_readable_str_method!(get_proxy_up_bps_str, proxy_up_bps);
    define_net_speed_to_human_readable_str_method!(get_proxy_down_bps_str, proxy_down_bps);
    define_net_speed_to_human_readable_str_method!(get_bypass_up_bps_str, bypass_up_bps);
    define_net_speed_to_human_readable_str_method!(get_bypass_down_bps_str, bypass_down_bps);
}

pub static WIFI_CONNECT_STATUS: Mutex<CriticalSectionRawMutex, WiFiConnectStatus> =
    Mutex::new(WiFiConnectStatus::Connecting);

pub static NET_SPEED: Mutex<CriticalSectionRawMutex, NetSpeed> = Mutex::new(NetSpeed {
    direct_up_bps: 0,
    direct_down_bps: 0,
    proxy_up_bps: 0,
    proxy_down_bps: 0,
    bypass_up_bps: 0,
    bypass_down_bps: 0,
});
