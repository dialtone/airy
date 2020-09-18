# airy
Air Quality Monitor (PM 1, 2.5, 10)


# Installation

You'll need to install `probe-run` by using:

```
cargo install \
    --git https://github.com/knurling-rs/probe-run \
    --branch main \
    --features defmt
```

And then simply proceed to run

```
cargo run
```

While having your ST-Link v2 connected to your STM32F4xx board.


# Hardware

This application expects a few hardware components:

* 1 OLED screen with SSD1306 controller on I2C
* 1 HM3301 air quality sensor on I2C
* 1 STM32F4xx board

The display and air quality sensor should share the same bus,
I'm using pin 8 and 9 for I2C.

The total cost of the setup should be somewhere around $40.
