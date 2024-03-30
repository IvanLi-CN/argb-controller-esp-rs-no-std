use core::fmt::Display;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

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
