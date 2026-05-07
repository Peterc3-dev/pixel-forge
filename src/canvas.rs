use serde::{Deserialize, Serialize};

use crate::color::{PixelColor, TRANSPARENT};

/// A 2D pixel canvas. Coordinates: (x, y) where y=0 is the top row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    /// Row-major: pixels[y][x]
    pub pixels: Vec<Vec<Option<PixelColor>>>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        let pixels = vec![vec![None; width]; height];
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Option<PixelColor> {
        if self.in_bounds(x, y) {
            self.pixels[y][x]
        } else {
            None
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: Option<PixelColor>) {
        if self.in_bounds(x, y) {
            self.pixels[y][x] = color;
        }
    }

    /// Flood-fill from (x, y) with the given color.
    /// Returns list of changed pixels for undo.
    pub fn flood_fill(&mut self, x: usize, y: usize, color: Option<PixelColor>) -> Vec<(usize, usize, Option<PixelColor>)> {
        if !self.in_bounds(x, y) {
            return vec![];
        }
        let target = self.pixels[y][x];
        if target == color {
            return vec![];
        }
        let mut changes = Vec::new();
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            if !self.in_bounds(cx, cy) {
                continue;
            }
            if self.pixels[cy][cx] != target {
                continue;
            }
            changes.push((cx, cy, self.pixels[cy][cx]));
            self.pixels[cy][cx] = color;

            if cx > 0 {
                stack.push((cx - 1, cy));
            }
            if cx + 1 < self.width {
                stack.push((cx + 1, cy));
            }
            if cy > 0 {
                stack.push((cx, cy - 1));
            }
            if cy + 1 < self.height {
                stack.push((cx, cy + 1));
            }
        }
        changes
    }

    /// Draw a line from (x0,y0) to (x1,y1) using Bresenham.
    /// Returns the list of pixels set (for undo).
    pub fn draw_line(
        &mut self,
        x0: isize,
        y0: isize,
        x1: isize,
        y1: isize,
        color: Option<PixelColor>,
    ) -> Vec<(usize, usize, Option<PixelColor>)> {
        let mut changes = Vec::new();
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx: isize = if x0 < x1 { 1 } else { -1 };
        let sy: isize = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut cx = x0;
        let mut cy = y0;
        loop {
            if cx >= 0 && cy >= 0 {
                let ux = cx as usize;
                let uy = cy as usize;
                if self.in_bounds(ux, uy) {
                    changes.push((ux, uy, self.pixels[uy][ux]));
                    self.pixels[uy][ux] = color;
                }
            }
            if cx == x1 && cy == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                cx += sx;
            }
            if e2 <= dx {
                err += dx;
                cy += sy;
            }
        }
        changes
    }

    /// Draw a rectangle outline or filled.
    pub fn draw_rect(
        &mut self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        color: Option<PixelColor>,
        filled: bool,
    ) -> Vec<(usize, usize, Option<PixelColor>)> {
        let min_x = x0.min(x1);
        let max_x = x0.max(x1);
        let min_y = y0.min(y1);
        let max_y = y0.max(y1);
        let mut changes = Vec::new();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if !self.in_bounds(x, y) {
                    continue;
                }
                let on_edge = x == min_x || x == max_x || y == min_y || y == max_y;
                if filled || on_edge {
                    changes.push((x, y, self.pixels[y][x]));
                    self.pixels[y][x] = color;
                }
            }
        }
        changes
    }

    /// Export as ANSI art using half-block characters.
    /// Each terminal row covers 2 pixel rows.
    pub fn to_ansi_string(&self, bg_color: PixelColor) -> String {
        let mut out = String::new();
        let rows = (self.height + 1) / 2;
        for row in 0..rows {
            let y_top = row * 2;
            let y_bot = row * 2 + 1;
            for x in 0..self.width {
                let top = self.pixels[y_top][x].unwrap_or(bg_color);
                let bot = if y_bot < self.height {
                    self.pixels[y_bot][x].unwrap_or(bg_color)
                } else {
                    bg_color
                };

                if top == bot {
                    // Full block — just use bg color
                    out.push_str(&top.ansi_bg());
                    out.push(' ');
                } else {
                    // Upper half block: fg = top, bg = bot
                    out.push_str(&top.ansi_fg());
                    out.push_str(&bot.ansi_bg());
                    out.push('\u{2580}'); // ▀
                }
            }
            out.push_str("\x1b[0m\n");
        }
        out
    }
}

/// Project file format.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec<Option<PixelColor>>>,
}

impl ProjectFile {
    pub fn from_canvas(c: &Canvas) -> Self {
        Self {
            width: c.width,
            height: c.height,
            pixels: c.pixels.clone(),
        }
    }

    pub fn to_canvas(self) -> Canvas {
        Canvas {
            width: self.width,
            height: self.height,
            pixels: self.pixels,
        }
    }
}
