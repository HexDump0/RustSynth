//! HSV/RGB color types used throughout the rendering pipeline.

use serde::{Deserialize, Serialize};

/// An RGBA color with floating-point components in `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a CSS hex color string (`#rrggbb` or `#rgb`).
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);
        match s.len() {
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
                Some(Self::new(r, g, b, 1.0))
            }
            3 => {
                let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()? as f32 / 255.0;
                Some(Self::new(r, g, b, 1.0))
            }
            _ => None,
        }
    }
}

/// A color in HSV space with an alpha component. All components in `[0.0, 1.0]`
/// except hue which is in degrees `[0.0, 360.0)`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Hsva {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}

impl Hsva {
    pub fn new(h: f32, s: f32, v: f32, a: f32) -> Self {
        Self { h, s, v, a }
    }

    /// Convert to RGBA.
    pub fn to_rgba(self) -> Rgba {
        let h = self.h / 60.0;
        let s = self.s;
        let v = self.v;
        let i = h.floor() as i32 % 6;
        let f = h - h.floor();
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);
        let (r, g, b) = match i {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        Rgba::new(r, g, b, self.a)
    }
}

impl Default for Hsva {
    fn default() -> Self {
        // White in HSV
        Self::new(0.0, 0.0, 1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parse_6() {
        let c = Rgba::from_hex("#ff0000").unwrap();
        assert!((c.r - 1.0).abs() < 1e-4);
        assert!(c.g < 1e-4);
        assert!(c.b < 1e-4);
    }

    #[test]
    fn hex_parse_3() {
        let c = Rgba::from_hex("#fff").unwrap();
        assert!((c.r - 1.0).abs() < 1e-4);
    }
}
