# Network-Monitor-esp-rs-no-std

This is the MCU software part of an Network speed monitor based on ESP32C3. The project collects network speed information from OpenWRT and OpenClash through a Rust server and sends the result to the monitor via UDP to display to the user.

![Finished Product](https://s3.ivanli.cc/ivan-public/uPic/2024/2atMTj.png)

## Hardwares

- MCU: ESP3-C3FH4
- Display: ST7735 80x160 RGB 0.96 inch

## Dependencies

- [esp-hal](https://github.com/esp-rs/esp-hal) (`no-std`)
- [esp-wifi](https://github.com/esp-rs/esp-wifi)
- [embassy](https://embassy.dev/)
- [st7735](https://github.com/kalkyl/st7735-embassy) (forked)

## Other Resources

- [Server Code](https://git.ivanli.cc/display-ambient-light/network-monitor);
- [Shell model](https://s.ivanli.cc/s/network-monitor-shell);

## License

MIT.
