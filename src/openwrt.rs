use embassy_executor::Spawner;
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Stack,
};
use embassy_time::{Duration, Instant, Timer};
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use libm::Libm;
use reqwless::{client::HttpClient, request::Method};
use static_cell::make_static;

use crate::{
    bus::{
        NetDataTrafficSpeed, WiFiConnectStatus, NET_DATA_LAN_TRAFFIC_SPEED,
        NET_DATA_WAN_TRAFFIC_SPEED, WIFI_CONNECT_STATUS,
    },
    openwrt_types,
};

#[embassy_executor::task]
pub async fn poll_info(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    let tcp_client_state: &'static TcpClientState<2, 1024, 1024> =
        make_static!(TcpClientState::new());
    let tcp_client: TcpClient<'static, WifiDevice<'static, WifiStaDevice>, 2> =
        TcpClient::new(stack, &tcp_client_state);
    let dns_socket = DnsSocket::new(&stack);

    let tcp_client = make_static!(tcp_client);
    let dns_socket = make_static!(dns_socket);

    let spawner = Spawner::for_current_executor().await;

    spawner.spawn(poll_wan_info(tcp_client, dns_socket)).ok();
    // spawner.spawn(poll_clash_info(tcp_client, dns_socket)).ok();
}

#[embassy_executor::task]
pub async fn poll_wan_info(
    tcp_client: &'static TcpClient<'static, WifiDevice<'static, WifiStaDevice>, 2>,
    dns_socket: &'static DnsSocket<'static, WifiDevice<'static, WifiStaDevice>>,
) {
    let mut header_rx_buf = [0; 512];
    let mut body_rx_buf = [0; 4096];

    let mut prev_fetch_at: Instant = Instant::now();

    loop {
        let wifi_status_guard = WIFI_CONNECT_STATUS.lock().await;

        if matches!(*wifi_status_guard, WiFiConnectStatus::Connecting) {
            drop(wifi_status_guard);
            // println!("Waiting for wifi...");
            Timer::after(Duration::from_millis(1_00)).await;
            continue;
        }
        drop(wifi_status_guard);
        let tcp_client_state: TcpClientState<1, 1024, 1024> = TcpClientState::new();

        let mut client = HttpClient::new(&tcp_client, &dns_socket); // Types implementing embedded-nal-async

        let url = "http://192.168.31.1:19990/api/v1/data?after=-1&chart=net.pppoe-wan&dimensions=received|sent&format=json&group=average&gtime=0&options=absolute|jsonwrap|nonzero&points=1&timeout=100";
        let mut request = client.request(Method::GET, &url).await.unwrap();
        let response = request.send(&mut header_rx_buf).await.unwrap();
        let mut reader = response.body().reader();

        let size = reader.read_to_end(&mut body_rx_buf).await.unwrap();
        let (data, _) =
            serde_json_core::de::from_slice::<'_, openwrt_types::Data>(&body_rx_buf[..size])
                .unwrap();

        let pub_msg = NetDataTrafficSpeed {
            up: Libm::<f32>::fabs(data.latest_values[1]) as u32,
            down: Libm::<f32>::fabs(data.latest_values[0]) as u32,
        };

        let mut speed = NET_DATA_WAN_TRAFFIC_SPEED.lock().await;
        *speed = pub_msg;
        drop(speed);

        {
            let mut client = HttpClient::new(&tcp_client, &dns_socket); // Types implementing embedded-nal-async

            let url = "http://192.168.31.1:19990/api/v1/data?after=-1&chart=net.br-lan&dimensions=received|sent&format=json&group=average&gtime=0&options=absolute|jsonwrap|nonzero&points=1&timeout=100";
            let mut request = client.request(Method::GET, &url).await.unwrap();
            let response = request.send(&mut header_rx_buf).await.unwrap();
            let mut reader = response.body().reader();

            let size = reader.read_to_end(&mut body_rx_buf).await.unwrap();
            let (data, _) =
                serde_json_core::de::from_slice::<'_, openwrt_types::Data>(&body_rx_buf[..size])
                    .unwrap();

            let pub_msg = NetDataTrafficSpeed {
                up: Libm::<f32>::fabs(data.latest_values[1]) as u32,
                down: Libm::<f32>::fabs(data.latest_values[0]) as u32,
            };

            let mut speed = NET_DATA_LAN_TRAFFIC_SPEED.lock().await;
            *speed = pub_msg;
            drop(speed);
        }

        // let wait = prev_fetch_at.checked_add(Duration::from_secs(data.update_every as u64));
        // prev_fetch_at = Instant::now();

        println!("curr: {:?}", Instant::now());
        // if let Some(wait) = wait {
        //     Timer::at(wait).await;
        // } else {
        // Timer::after_millis(200).await;
        // }
    }
}

#[embassy_executor::task]
pub async fn poll_clash_info(
    tcp_client: &'static TcpClient<'static, WifiDevice<'static, WifiStaDevice>, 2>,
    dns_socket: &'static DnsSocket<'static, WifiDevice<'static, WifiStaDevice>>,
) {
    let mut header_rx_buf = [0; 512];
    let mut body_rx_buf = [0; 4096];

    let mut prev_fetch_at: Instant = Instant::now();

    loop {
        let wifi_status_guard = WIFI_CONNECT_STATUS.lock().await;

        if matches!(*wifi_status_guard, WiFiConnectStatus::Connecting) {
            drop(wifi_status_guard);
            // println!("Waiting for wifi...");
            Timer::after(Duration::from_millis(1_00)).await;
            continue;
        }
        drop(wifi_status_guard);
        let tcp_client_state: TcpClientState<1, 1024, 1024> = TcpClientState::new();

        let mut client = HttpClient::new(&tcp_client, &dns_socket); // Types implementing embedded-nal-async

        let url = "http://192.168.31.1:19990/api/v1/data?after=-1&chart=net.br-lan&dimensions=received|sent&format=json&group=average&gtime=0&options=absolute|jsonwrap|nonzero&points=1&timeout=100";
        let mut request = client.request(Method::GET, &url).await.unwrap();
        let response = request.send(&mut header_rx_buf).await.unwrap();
        let mut reader = response.body().reader();

        let size = reader.read_to_end(&mut body_rx_buf).await.unwrap();
        let (data, _) =
            serde_json_core::de::from_slice::<'_, openwrt_types::Data>(&body_rx_buf[..size])
                .unwrap();

        let pub_msg = NetDataTrafficSpeed {
            up: Libm::<f32>::fabs(data.latest_values[1]) as u32,
            down: Libm::<f32>::fabs(data.latest_values[0]) as u32,
        };

        let mut speed = NET_DATA_LAN_TRAFFIC_SPEED.lock().await;
        *speed = pub_msg;
        drop(speed);

        // let wait = prev_fetch_at.checked_add(Duration::from_secs(data.update_every as u64));
        // prev_fetch_at = Instant::now();

        println!("curr: {:?}", Instant::now());
        // if let Some(wait) = wait {
        //     Timer::at(wait).await;
        // } else {
        // Timer::after_millis(200).await;
        // }
    }
}

#[embassy_executor::task]
pub async fn netdata_clash_info(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    let mut header_rx_buf = [0; 512];
    let mut body_rx_buf = [0; 4096];

    let mut prev_fetch_at: Instant = Instant::now();

    loop {
        let wifi_status_guard = WIFI_CONNECT_STATUS.lock().await;

        if matches!(*wifi_status_guard, WiFiConnectStatus::Connecting) {
            drop(wifi_status_guard);
            // println!("Waiting for wifi...");
            Timer::after(Duration::from_millis(1_00)).await;
            continue;
        }
        drop(wifi_status_guard);

        {
            let tcp_client_state: TcpClientState<1, 1024, 1024> = TcpClientState::new();
            let tcp_client = TcpClient::new(stack, &tcp_client_state);
            let dns_socket = DnsSocket::new(&stack);

            let mut client = HttpClient::new(&tcp_client, &dns_socket); // Types implementing embedded-nal-async

            let url = "http://192.168.31.1:19990/api/v1/data?after=-1&chart=net.pppoe-wan&dimensions=received|sent&format=json&group=average&gtime=0&options=absolute|jsonwrap|nonzero&points=1&timeout=100";
            let mut request = client.request(Method::GET, &url).await.unwrap();
            let response = request.send(&mut header_rx_buf).await.unwrap();
            let mut reader = response.body().reader();

            let size = reader.read_to_end(&mut body_rx_buf).await.unwrap();
            let (data, _) =
                serde_json_core::de::from_slice::<'_, openwrt_types::Data>(&body_rx_buf[..size])
                    .unwrap();

            let pub_msg = NetDataTrafficSpeed {
                up: Libm::<f32>::fabs(data.latest_values[1]) as u32,
                down: Libm::<f32>::fabs(data.latest_values[0]) as u32,
            };

            let mut speed = NET_DATA_WAN_TRAFFIC_SPEED.lock().await;
            *speed = pub_msg;
            drop(speed);
        }

        // {
        //     let tcp_client_state: TcpClientState<1, 1024, 1024> = TcpClientState::new();
        //     let tcp_client = TcpClient::new(stack, &tcp_client_state);
        //     let dns_socket = DnsSocket::new(&stack);

        //     let mut client = HttpClient::new(&tcp_client, &dns_socket); // Types implementing embedded-nal-async

        //     let url = "http://192.168.31.1:19990/api/v1/data?after=-1&chart=net.br-lan&dimensions=received|sent&format=json&group=average&gtime=0&options=absolute|jsonwrap|nonzero&points=1&timeout=100";
        //     let mut request = client.request(Method::GET, &url).await.unwrap();
        //     let response = request.send(&mut header_rx_buf).await.unwrap();
        //     let mut reader = response.body().reader();

        //     let size = reader.read_to_end(&mut body_rx_buf).await.unwrap();
        //     let (data, _) =
        //         serde_json_core::de::from_slice::<'_, openwrt_types::Data>(&body_rx_buf[..size])
        //             .unwrap();

        //     let pub_msg = NetDataTrafficSpeed {
        //         up: Libm::<f32>::fabs(data.latest_values[1]) as u32,
        //         down: Libm::<f32>::fabs(data.latest_values[0]) as u32,
        //     };

        //     let mut speed = NET_DATA_LAN_TRAFFIC_SPEED.lock().await;
        //     *speed = pub_msg;
        //     drop(speed);
        // }

        // let wait = prev_fetch_at.checked_add(Duration::from_secs(data.update_every as u64));
        // prev_fetch_at = Instant::now();

        println!("curr: {:?}", Instant::now());
        // if let Some(wait) = wait {
        //     Timer::at(wait).await;
        // } else {
        // Timer::after_millis(200).await;
        // }
    }
}
