use core::fmt::Display;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};

#[derive(Debug, Clone, Copy)]
pub struct NetDataTrafficSpeed {
    pub up: u32,
    pub down: u32,
}

impl Display for NetDataTrafficSpeed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "up: {}B, down: {}B", self.up, self.down)
    }
}

pub static NET_DATA_TRAFFIC_SPEED_PUB_SUB: PubSubChannel<
    CriticalSectionRawMutex,
    NetDataTrafficSpeed,
    4,
    4,
    1,
> = PubSubChannel::new();
