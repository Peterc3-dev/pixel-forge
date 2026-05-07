use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Pixel color — stored as RGB, rendered via ratatui Color::Rgb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PixelColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_ratatui(self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }

    /// ANSI escape for 24-bit foreground.
    pub fn ansi_fg(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// ANSI escape for 24-bit background.
    pub fn ansi_bg(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }
}

/// The "no color" / transparent sentinel.
pub const TRANSPARENT: PixelColor = PixelColor::new(0, 0, 0);

/// Build the default palette: 16 classic ANSI + 24 extended.
pub fn default_palette() -> Vec<PixelColor> {
    let mut p = Vec::with_capacity(40);

    // 16 standard ANSI colors
    let ansi16: [(u8, u8, u8); 16] = [
        (0, 0, 0),       // black
        (170, 0, 0),     // red
        (0, 170, 0),     // green
        (170, 85, 0),    // yellow/brown
        (0, 0, 170),     // blue
        (170, 0, 170),   // magenta
        (0, 170, 170),   // cyan
        (170, 170, 170), // white
        (85, 85, 85),    // bright black
        (255, 85, 85),   // bright red
        (85, 255, 85),   // bright green
        (255, 255, 85),  // bright yellow
        (85, 85, 255),   // bright blue
        (255, 85, 255),  // bright magenta
        (85, 255, 255),  // bright cyan
        (255, 255, 255), // bright white
    ];
    for (r, g, b) in ansi16 {
        p.push(PixelColor::new(r, g, b));
    }

    // 24 extended colors — a nice spread
    let extended: [(u8, u8, u8); 24] = [
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (255, 165, 0),   // orange
        (255, 192, 203), // pink
        (75, 0, 130),    // indigo
        (238, 130, 238), // violet
        (0, 255, 127),   // spring green
        (64, 224, 208),  // turquoise
        (255, 215, 0),   // gold
        (210, 105, 30),  // chocolate
        (220, 20, 60),   // crimson
        (0, 191, 255),   // deep sky blue
        (34, 139, 34),   // forest green
        (255, 99, 71),   // tomato
        (106, 90, 205),  // slate blue
        (244, 164, 96),  // sandy brown
        (46, 139, 87),   // sea green
        (176, 196, 222), // light steel blue
        (139, 69, 19),   // saddle brown
    ];
    for (r, g, b) in extended {
        p.push(PixelColor::new(r, g, b));
    }

    p
}
