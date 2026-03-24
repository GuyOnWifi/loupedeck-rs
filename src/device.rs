//! High-level controller for Loupedeck and Razer Stream Controller hardware.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::{
    constants::{Message, SCREEN_ADDRESS},
    error::Result,
    transport::{DeviceInfo, WebsocketSerial},
};

const WS_UPGRADE_HEADER: &str = r#"GET /index.html
HTTP/1.1
Connection: Upgrade
Upgrade: websocket
Sec-WebSocket-Key: 123abc

"#;

const WS_UPGRADE_RESPONSE: &str = "HTTP/1.1 101 Switching Protocols";

/// Represents an event originating from the device.
#[derive(Debug)]
pub enum Event {
    /// A button was pressed or released.
    ButtonPress {
        /// ID of the button (0-11 for main buttons).
        button_id: u8,
        /// 1 for press, 0 for release.
        press: u8,
    },
    /// A knob was rotated.
    KnobRotate {
        /// ID of the knob.
        knob_id: u8,
        /// Delta value (positive for clockwise, negative for counter-clockwise).
        delta: i8,
    },
    /// A touch event started on the screen.
    Touch {
        /// X coordinate.
        x: u16,
        /// Y coordinate.
        y: u16,
        /// Touch identifier (for multi-touch).
        id: u8,
    },
    /// A touch event ended.
    TouchRelease {
        /// Last X coordinate.
        x: u16,
        /// Last Y coordinate.
        y: u16,
        /// Touch identifier.
        id: u8,
    },
    /// A raw message that couldn't be parsed into a specific event.
    Raw {
        /// The raw payload bytes.
        data: Vec<u8>,
    },
}

/// The main controller for interacting with a Loupedeck device.
pub struct RazerStreamController {
    transport: WebsocketSerial,
    transaction_id: u8, // Stores the NEXT transaction id
    event_buffer: VecDeque<Event>,
}

impl RazerStreamController {
    /// Creates a new controller instance and performs the WebSocket upgrade handshake.
    pub fn new(dev: DeviceInfo) -> Result<Self> {
        let mut controller = Self {
            transport: WebsocketSerial::new(dev)?,
            transaction_id: 1,
            event_buffer: VecDeque::new(),
        };

        controller.transport.clear_buffers()?;

        if let Ok(Some(_)) = controller.get_serial() {
            // We know it was able to return a serial, so probably already connected
            return Ok(controller);
        }

        // Connect
        controller
            .transport
            .send_raw(WS_UPGRADE_HEADER.as_bytes())?;

        // TODO: BETTER BUFFER HANDLING
        loop {
            let mut buf = [0u8; 1024];

            let _ = controller.transport.read_raw(&mut buf)?;
            if buf.starts_with(WS_UPGRADE_RESPONSE.as_bytes()) {
                return Ok(controller);
            }
        }
    }

    fn send_raw_command(&mut self, command_id: u8, payload: &[u8]) -> Result<u8> {
        // Clamp value to 0xFF. Make sure minimum is 3
        let length = std::cmp::min(payload.len(), 0xff - 3) as u8 + 3;
        let mut packet: Vec<u8> = vec![length, command_id, self.transaction_id];
        packet.extend(payload);
        self.transport.send(&packet)?;

        // Device doesn't like transaction id 0
        let current_id = self.transaction_id;
        if self.transaction_id == 255 {
            self.transaction_id = 0;
        }
        self.transaction_id += 1;
        Ok(current_id)
    }

    /// Sets the RGB color of a button LED.
    pub fn set_color(&mut self, button_id: u8, r: u8, g: u8, b: u8) -> Result<()> {
        let packet = [button_id, r, g, b];
        self.send_raw_command(Message::SetColor as u8, &packet)?;
        Ok(())
    }

    /// Sets the screen brightness (0-255).
    pub fn set_brightness(&mut self, brightness: u8) -> Result<()> {
        let packet = [brightness];
        self.send_raw_command(Message::SetBrightness as u8, &packet)?;
        Ok(())
    }

    /// Requests the device serial number.
    pub fn get_serial(&mut self) -> Result<Option<Vec<u8>>> {
        let packet: [u8; 0] = [];
        let id = self.send_raw_command(Message::Serial as u8, &packet)?;
        if let Some(Event::Raw { data }) = self.read_wait(id)? {
            return Ok(Some(data));
        }
        Ok(None)
    }

    /// Requests the firmware version from the device.
    pub fn get_version(&mut self) -> Result<Option<Vec<u8>>> {
        let packet: [u8; 0] = [];
        let id = self.send_raw_command(Message::Version as u8, &packet)?;
        if let Some(Event::Raw { data }) = self.read_wait(id)? {
            return Ok(Some(data));
        }
        Ok(None)
    }

    /// Triggers a screen refresh.
    pub fn refresh(&mut self) -> Result<()> {
        self.send_raw_command(Message::Draw as u8, &SCREEN_ADDRESS)?;
        Ok(())
    }

    /// Draws a bitmap to a specific region of the screen.
    pub fn draw(
        &mut self,
        x_off: u16,
        y_off: u16,
        width: u16,
        height: u16,
        pixel_buf: &[u16],
    ) -> Result<()> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend(&SCREEN_ADDRESS);
        buf.extend(x_off.to_be_bytes());
        buf.extend(y_off.to_be_bytes());
        buf.extend(width.to_be_bytes());
        buf.extend(height.to_be_bytes());
        buf.extend(pixel_buf.iter().flat_map(|px| px.to_le_bytes()));

        self.send_raw_command(Message::FrameBuff as u8, &buf)?;
        Ok(())
    }

    /// Resets the device.
    pub fn reset(&mut self) -> Result<()> {
        let packet: [u8; 0] = [];
        self.send_raw_command(Message::Reset as u8, &packet)?;
        Ok(())
    }

    /// Pops the next pending event from the device.
    pub fn get_evt(&mut self) -> Result<Option<Event>> {
        if !self.event_buffer.is_empty() {
            return Ok(Some(self.event_buffer.pop_front().unwrap()));
        }

        if let Some(evt) = self.read()? {
            return Ok(Some(evt.1));
        };

        Ok(None)
    }

    /// Internal method to read a single frame and parse it into an [`Event`].
    fn read(&mut self) -> Result<Option<(u8, Event)>> {
        if let Some(command) = self.transport.read()? {
            let _ = command[0]; // length, ignored because handled by underlying transport 
            let command_id = command[1];
            let transaction_id = command[2];
            let data = &command[3..];
            let evt = match command_id {
                0x00 => Event::ButtonPress {
                    button_id: data[0],
                    press: data[1],
                },
                0x01 => Event::KnobRotate {
                    knob_id: data[0],
                    delta: data[1] as i8,
                },
                0x4d => Event::Touch {
                    x: u16::from_be_bytes(data[1..3].try_into().unwrap()),
                    y: u16::from_be_bytes(data[3..5].try_into().unwrap()),
                    id: data[5],
                },
                0x6d => Event::TouchRelease {
                    x: u16::from_be_bytes(data[1..3].try_into().unwrap()),
                    y: u16::from_be_bytes(data[3..5].try_into().unwrap()),
                    id: data[5],
                },
                _ => Event::Raw {
                    data: data.to_vec(),
                },
            };

            return Ok(Some((transaction_id, evt)));
        };

        Ok(None)
    }

    /// Waits for a specific transaction ID response, buffering other events in the meantime.
    fn read_wait(&mut self, trans_id: u8) -> Result<Option<Event>> {
        let instant = Instant::now();
        let duration = Duration::from_millis(100);

        while instant.elapsed() < duration {
            if let Some(evt) = self.read()? {
                if evt.0 == trans_id {
                    return Ok(Some(evt.1));
                }

                self.event_buffer.push_back(evt.1);
            }
        }

        Ok(None)
    }
}
