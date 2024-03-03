use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Stack,
};

use embassy_time::{Duration, Timer};
use embedded_io_async::Read;
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use reqwless::{client::HttpClient, request::Method};

use crate::wifi::NETWORK_CONFIG;

#[embassy_executor::task]
pub async fn netdata_info(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    let mut header_rx_buf = [0; 512];
    let mut body_rx_buf = [0; 1024];

    loop {
        if NETWORK_CONFIG.lock().is_none() {
            println!("Waiting for network config...");
            Timer::after(Duration::from_millis(1_000)).await;
            continue;
        }

        Timer::after(Duration::from_millis(1_000)).await;

        let tcp_client_state: TcpClientState<1, 1024, 1024> = TcpClientState::new();
        let tcp_client = TcpClient::new(stack, &tcp_client_state);
        let dns_socket = DnsSocket::new(&stack);

        println!("Fetching...");

        let url = "http://192.168.31.1:19990/api/v1/data?after=-60&chart=net.pppoe-wan&dimensions=received&format=json&group=average&gtime=0&options=absolute|jsonwrap|nonzero&points=1020&timeout=100";
        let mut client = HttpClient::new(&tcp_client, &dns_socket); // Types implementing embedded-nal-async

        let mut request = client.request(Method::GET, &url).await.unwrap();
        let response = request.send(&mut header_rx_buf).await.unwrap();
        let mut reader = response.body().reader();

        // read body
        while let Ok(bytes_read) = reader.read(&mut body_rx_buf).await {
            if bytes_read == 0 {
                break;
            }

            println!(
                "{}",
                core::str::from_utf8(&body_rx_buf[..bytes_read]).unwrap()
            );
        }

        Timer::after(Duration::from_millis(3000)).await;
    }
}
