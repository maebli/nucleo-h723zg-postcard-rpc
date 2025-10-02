# Nucleo H723zg postcards-rpc

# How to run this project

## Important

When you connect your make sure you connect it on both USB ports and that the cables are data cables and not just power cables.
Make sure there is power on the usb port. k

If you see the following error it is likely that there is not enough power 

``` bash
28.477722 [TRACE] read ep=EndpointAddress(1): doepctl 00000000 (embassy_usb_synopsys_otg embassy-usb-synopsys-otg-0.2.0/src/lib.rs:1061)
28.477874 [TRACE] read ep=EndpointAddress(1) error disabled (embassy_usb_synopsys_otg embassy-usb-synopsys-otg-0.2.0/src/lib.rs:1063)
```

## Firmware
You must run `cd firmware` and then `cargo run` to flash the firmware. 
You need to have probe-rs installed and rustup target add armv7-unknown-linux-gnueabihf

## Software
Then in another terminal window run:

`cd software`

`cargo run --bin comms-01`

this will send ping 0 to 9 to the device and the device will return the ping

or
 
`cargo run --bin comms-02`

enter `help` to list all commands


## Important Links

- https://docs.embassy.dev/embassy-stm32/git/stm32h725zg/index.html
- https://github.com/embassy-rs/embassy/blob/main/examples/stm32l4/src/bin/usb_serial.rs



### Open points

- [ ] Add an counter stream that just bumps a number as fast as possible
- [ ] Add a complex stream of "system status" where the three led states is present
- [ ] Send firmware pages as bin on request