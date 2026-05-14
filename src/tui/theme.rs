// SPDX-License-Identifier: GPL-3.0-or-later

use ratatui::style::Color;

/// Spacecraft Software v1.2 six-token palette.
pub struct Palette;

impl Palette {
    pub const VOID_NAVY: Color = Color::Rgb(0x00, 0x00, 0x27);
    pub const MOLTEN_AMBER: Color = Color::Rgb(0xD9, 0x8E, 0x32);
    pub const STEEL_BLUE: Color = Color::Rgb(0x4B, 0x7E, 0xB0);
    pub const RADIUM_GREEN: Color = Color::Rgb(0x50, 0xFA, 0x7B);
    pub const LIQUID_COOLANT: Color = Color::Rgb(0x8B, 0xE9, 0xFD);
    #[allow(dead_code)]
    pub const RED_OXIDE: Color = Color::Rgb(0xFF, 0x5C, 0x5C);
}
