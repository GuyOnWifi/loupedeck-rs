//! Protocol constants and hardware identifiers for Loupedeck devices.

/// Supported hardware vendors.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VendorID {
    Loupedeck = 0x2ec2,
    Razer = 0x1532,
}

impl VendorID {
    /// Returns the [`VendorID`] from a 16-bit vendor identifier.
    pub fn from_u16(pid: u16) -> Option<Self> {
        match pid {
            0x2ec2 => Some(Self::Loupedeck),
            0x1532 => Some(Self::Razer),
            _ => None,
        }
    }
}

/// Known Product Identifiers (PID) for Loupedeck and compatible hardware.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePID {
    LoupedeckLive = 0x0004,
    LoupedeckCT = 0x0003,
    LoupedeckLiveS = 0x0006,
    RazerStreamController = 0x0d06,
    RazerStreamControllerX = 0x0d09,
}

impl DevicePID {
    /// Returns the [`DevicePID`] from a 16-bit product identifier.
    pub fn from_u16(pid: u16) -> Option<Self> {
        match pid {
            0x0004 => Some(Self::LoupedeckLive),
            0x0003 => Some(Self::LoupedeckCT),
            0x0006 => Some(Self::LoupedeckLiveS),
            0x0d06 => Some(Self::RazerStreamController),
            0x0d09 => Some(Self::RazerStreamControllerX),
            _ => None,
        }
    }
}

/// Command IDs used in the Loupedeck protocol.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    /// A button was pressed or released (Device -> Host).
    ButtonPress = 0x00,
    /// A knob was rotated (Device -> Host).
    KnobRotate = 0x01,
    /// Set the color of a button LED (Host -> Device)
    SetColor = 0x02,
    /// Request or receive device serial number (Host ↔ Device).
    Serial = 0x03,
    /// Reset the device (Host -> Device).
    Reset = 0x06,
    /// Request or receive firmware version (Host ↔ Device).
    Version = 0x07,
    /// Set the screen brightness (Host -> Device).
    SetBrightness = 0x09,
    /// Interaction with the MCU (Host ->  Device).
    MCU = 0x0D,
    /// Trigger a screen refresh/draw (Host -> Device).
    Draw = 0x0F,
    /// Send graphics data to the frame buffer (Host -> Device).
    FrameBuff = 0x10,
    /// Set vibration/haptic feedback (Host -> Device).
    SetVibration = 0x1B,
    /// Touch event started (Device -> Host).
    Touch = 0x4D,
    /// Touch event ended (Device -> Host).
    TouchEnd = 0x6D,
}

/// The base address for the main screen.
pub const SCREEN_ADDRESS: [u8; 2] = [0, b'M'];
