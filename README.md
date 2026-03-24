# loupedeck-rs

A Rust library for interacting with Loupedeck devices (Loupedeck Live, Live S, CT) and the Razer Stream Controller.

## Features

- **Device Discovery**: Automatically find connected Loupedeck-compatible devices.
- **Event Handling**: Listen for button presses, knob rotations, and touch screen events.
- **Graphic Rendering**: Draw bitmaps (RGB565) to the main screen.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
loupedeck-driver = { path = "." }
```

## Quick Start

```rust
use loupedeck_driver::{device::RazerStreamController, transport::WebsocketSerial, error::Result};

fn main() -> Result<()> {
    // Discover devices
    let devices = WebsocketSerial::discover()?;

    if devices.is_empty() {
        println!("No devices found!");
        return Ok(());
    }

    // Connect to the first device found
    let mut dev = RazerStreamController::new(devices[0].clone())?;
    println!("Connected to {}!", devices[0].port_name);

    loop {
        // Listen for events
        if let Some(evt) = dev.get_evt()? {
            println!("Received event: {:?}", evt);
        }
    }
}
```

## Supported Devices

- Razer Stream Controller
- Razer Stream Controller X

### Needs testing

- Loupedeck Live
- Loupedeck Live S
- Loupedeck CT

Any help or testing would be appreciated!

## License

This project is licensed under the GNU License - see the [LICENSE](LICENSE) file for details.
