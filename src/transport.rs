//! Transport layer for serial communication using WebSocket framing.

use std::time::Duration;

use serialport::{SerialPort, SerialPortType::UsbPort};

use crate::{
    constants::{DevicePID, VendorID},
    error::Result,
};

/// High-level serial transport with WebSocket framing.
///
/// This struct manages the underlying serial port and handles the framing
/// required by the Loupedeck protocol (which uses a subset of WebSocket framing).
pub struct WebsocketSerial {
    serial: Box<dyn SerialPort>,
    buffer: Vec<u8>,
}

/// Information about a discovered Loupedeck device.
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    /// The vendor identifier (Loupedeck or Razer).
    pub vendor: VendorID,
    /// The product identifier for the specific device model.
    pub device: DevicePID,
    /// The name of the serial port (e.g., "/dev/ttyUSB0" or "COM3").
    pub port_name: String,
}

impl WebsocketSerial {
    /// Opens a new serial connection to the specified device.
    pub fn new(dev: DeviceInfo) -> Result<Self> {
        Ok(WebsocketSerial {
            serial: serialport::new(dev.port_name, 256000)
                .timeout(Duration::from_millis(100))
                .open()?,
            buffer: Vec::new(),
        })
    }

    /// Discovers all connected Loupedeck-compatible devices.
    pub fn discover() -> Result<Vec<DeviceInfo>> {
        let mut devices: Vec<DeviceInfo> = Vec::new();

        let ports = serialport::available_ports()?;
        for p in ports {
            if let UsbPort(info) = p.port_type
                && let Some(vendor) = VendorID::from_u16(info.vid)
                && let Some(device) = DevicePID::from_u16(info.pid)
            {
                devices.push(DeviceInfo {
                    vendor,
                    device,
                    port_name: p.port_name,
                });
            }
        }

        Ok(devices)
    }

    /// Sends a raw payload wrapped in a WebSocket frame.
    pub fn send(&mut self, raw: &[u8]) -> Result<()> {
        // 0x82 is the websocket header
        let mut buff = vec![0x82];

        // Ignore 0xFE case for simplicity
        if raw.len() < 127 {
            buff.push(0x80 + raw.len() as u8);
        } else {
            buff.push(0xff);
            let bytes = (raw.len() as u64).to_be_bytes();
            buff.extend_from_slice(&bytes);
        }

        buff.extend(vec![0, 0, 0, 0]); // XOR Mask 
        buff.extend_from_slice(raw); // Payload
        self.serial.write_all(&buff)?;
        Ok(())
    }

    /// Sends raw data without any framing.
    pub fn send_raw(&mut self, data: &[u8]) -> Result<()> {
        self.serial.write_all(data)?;
        Ok(())
    }

    /// Clears both the input and output serial buffers.
    pub fn clear_buffers(&mut self) -> Result<()> {
        self.serial.clear(serialport::ClearBuffer::All)?;
        Ok(())
    }

    /// Reads raw data from the serial port.
    pub fn read_raw(&mut self, buf: &mut [u8]) -> Result<usize> {
        Ok(self.serial.read(buf)?)
    }

    /// Attempts to read and decode a single WebSocket frame from the input buffer.
    pub fn read(&mut self) -> Result<Option<Vec<u8>>> {
        let mut temp = [0u8; 1024];

        let n = self.serial.read(&mut temp)?;
        self.buffer.extend_from_slice(&temp[..n]);

        if let Some(pos) = self.buffer.iter().position(|&b| b == 0x82_u8) {
            self.buffer.drain(..pos);

            // Not enough for header
            if self.buffer.len() < 2 {
                return Ok(None);
            }

            let len: usize = self.buffer[1] as usize;

            // Packet + header
            if self.buffer.len() >= len + 2 {
                let data = self.buffer[2..len + 2].to_vec();
                self.buffer.drain(..len + 2);
                return Ok(Some(data));
            }
        };

        Ok(None)
    }
}
