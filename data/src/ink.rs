//! Marks, painted. The canvas shown and the picture copied both come from [`paint`].

use crate::shapes::{Mark, Point, ScreenCapture};
use exact_runner::exact_canvas::Context2d;
use std::hash::{BuildHasher, RandomState};

/// The frozen desktop as pixelation reads it: each cell of one grid over the screen, summed once.
pub struct Screen {
    /// Per cell: red, green and blue sums, then the pixel count; rows of `columns`, top first.
    sums: Vec<[u32; 4]>,
    columns: usize,
    /// A cell's side, in pixels.
    cell: usize,
    width: usize,
    height: usize,
    /// Physical pixels per point.
    scale: f64,
    /// Per capture, so no two pixelate alike.
    grain: RandomState,
}

impl Screen {
    /// `pixels`: RGBA rows `width` across, top first; `scale` pixels to a point.
    pub fn new(pixels: &[u8], width: usize, height: usize, scale: f64) -> Screen {
        let cell = (CELL * scale).round().max(1.0) as usize;
        let columns = width.div_ceil(cell);
        let mut sums = vec![[0u32; 4]; columns * height.div_ceil(cell)];
        for (y, line) in pixels.chunks_exact(width * 4).enumerate() {
            let sums = &mut sums[y / cell * columns..][..columns];
            for (sum, run) in sums.iter_mut().zip(line.chunks(cell * 4)) {
                let run = run.as_chunks::<4>().0;
                for pixel in run {
                    for channel in 0..3 {
                        sum[channel] += u32::from(pixel[channel]);
                    }
                }
                sum[3] += run.len() as u32;
            }
        }
        Screen {
            sums,
            columns,
            cell,
            width,
            height,
            scale,
            grain: RandomState::new(),
        }
    }

    /// The pixel column `points` across the screen, kept on it.
    fn column(&self, points: f64) -> usize {
        self.pixel(points, self.width)
    }

    /// The pixel row `points` down the screen, kept on it.
    fn row(&self, points: f64) -> usize {
        self.pixel(points, self.height)
    }

    fn pixel(&self, points: f64, limit: usize) -> usize {
        (points * self.scale).round().clamp(0.0, limit as f64) as usize
    }
}

/// Onto a canvas whose top left is (`left`, `top`) of the desktop, in points.
pub fn desktop(ctx: &Context2d, stills: &[ScreenCapture], left: f64, top: f64) {
    // Pixel for pixel: nothing to smooth.
    ctx.set_image_smoothing_enabled(false);
    for still in stills {
        if let Some(rect) = &still.screen.client_rect {
            let _ = ctx.draw_image_with_image_handle_and_dw_and_dh(
                &still.path,
                rect.x - left,
                rect.y - top,
                rect.width,
                rect.height,
            );
        }
    }
}

/// `marks` in desktop points, onto a canvas whose top left is (`left`, `top`).
/// `screen`: what pixelated marks hide.
pub fn paint(ctx: &Context2d, marks: &[Mark], left: f64, top: f64, screen: Option<&Screen>) {
    ctx.save();
    let _ = ctx.translate(-left, -top);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    for mark in marks {
        ctx.set_stroke_style_str(&mark.color);
        ctx.set_fill_style_str(&mark.color);
        ctx.set_line_width(mark.size);
        let (x0, y0, x1, y1) = (mark.x0, mark.y0, mark.x1, mark.y1);
        match mark.tool.as_str() {
            "arrow" => arrow(ctx, mark),
            "text" => {
                // Matches the text field in `app.contract`.
                ctx.set_font(&format!("600 {}px system-ui", mark.size));
                ctx.set_text_baseline("top");
                let _ = ctx.fill_text(&mark.text, x0, y0);
            }
            "pixelate" => pixelate(ctx, mark, screen.expect("a pixelated mark has its screen")),
            outline => {
                ctx.begin_path();
                match outline {
                    "pen" => pen(ctx, &mark.path),
                    "line" => {
                        ctx.move_to(x0, y0);
                        ctx.line_to(x1, y1);
                    }
                    "rect" => ctx.rect(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs()),
                    "ellipse" => {
                        let (rx, ry) = ((x1 - x0).abs() / 2.0, (y1 - y0).abs() / 2.0);
                        let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                        let _ = ctx.ellipse(cx, cy, rx, ry, 0.0, 0.0, std::f64::consts::TAU);
                    }
                    other => unreachable!("`{other}` has no arm here"),
                }
                ctx.stroke();
            }
        }
    }
    ctx.restore();
}

/// Through `points`, curving between segment midpoints.
fn pen(ctx: &Context2d, points: &[Point]) {
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return;
    };
    ctx.move_to(first.x, first.y);
    // A dot, for a press that never moved.
    ctx.line_to(first.x, first.y);
    for pair in points.windows(2).skip(1) {
        let (a, b) = (&pair[0], &pair[1]);
        ctx.quadratic_curve_to(a.x, a.y, (a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    }
    ctx.line_to(last.x, last.y);
}

fn arrow(ctx: &Context2d, mark: &Mark) {
    let (x0, y0, x1, y1) = (mark.x0, mark.y0, mark.x1, mark.y1);
    let length = (x1 - x0).hypot(y1 - y0);
    if length == 0.0 {
        return;
    }
    let (ux, uy) = ((x1 - x0) / length, (y1 - y0) / length);
    let head = (mark.size * 4.0 + 6.0).min(length);
    let half = head * 0.42;
    // Shaft stops inside the head so its round cap can't blunt the tip.
    let (bx, by) = (x1 - ux * head, y1 - uy * head);
    ctx.begin_path();
    ctx.move_to(x0, y0);
    ctx.line_to(bx + ux * head * 0.2, by + uy * head * 0.2);
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(x1, y1);
    ctx.line_to(bx - uy * half, by + ux * half);
    ctx.line_to(bx + uy * half, by - ux * half);
    ctx.close_path();
    ctx.fill();
}

/// Pixelation cell side, in points.
const CELL: f64 = 12.0;
/// Cells past its own that a cell averages, each way.
const REACH: usize = 2;
/// Most a grain moves a channel, of 255.
const GRAIN: u64 = 16;

/// One flat square of a pixelated area, in pixels.
struct Cell {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    rgb: [u32; 3],
}

fn pixelate(ctx: &Context2d, mark: &Mark, screen: &Screen) {
    let points = |pixels: usize| pixels as f64 / screen.scale;
    for cell in cells(mark, screen) {
        let [red, green, blue] = cell.rgb;
        ctx.set_fill_style_str(&format!("#{red:02x}{green:02x}{blue:02x}"));
        ctx.fill_rect(
            points(cell.x),
            points(cell.y),
            points(cell.width),
            points(cell.height),
        );
    }
}

/// Irreversible: plain pixelation falls to pixelating guesses until one matches, so a cell
/// averages an area far wider than a letter, plus a grain no one can reproduce.
fn cells(mark: &Mark, screen: &Screen) -> Vec<Cell> {
    let (left, right) = (
        screen.column(mark.x0.min(mark.x1)),
        screen.column(mark.x0.max(mark.x1)),
    );
    let (top, bottom) = (
        screen.row(mark.y0.min(mark.y1)),
        screen.row(mark.y0.max(mark.y1)),
    );
    if left == right || top == bottom {
        return Vec::new();
    }
    // One grid over the screen, so cells hold still as the area is dragged.
    let (cell, columns) = (screen.cell, screen.columns);
    let rows = screen.sums.len() / columns;
    let mut cells = Vec::new();
    for row in top / cell..=(bottom - 1) / cell {
        for column in left / cell..=(right - 1) / cell {
            let mut total = [0u32; 4];
            for near_row in row.saturating_sub(REACH)..(row + REACH + 1).min(rows) {
                for near in column.saturating_sub(REACH)..(column + REACH + 1).min(columns) {
                    let sum = screen.sums[near_row * columns + near];
                    total.iter_mut().zip(sum).for_each(|(t, s)| *t += s);
                }
            }
            let grain = (screen.grain.hash_one((column, row)) % (GRAIN + 1)) as u32;
            let rgb = [0, 1, 2].map(|channel| {
                let average = total[channel] / total[3];
                // Towards mid-grey, so black and white don't clip the grain.
                if average > 127 {
                    average - grain
                } else {
                    average + grain
                }
            });
            let (x, y) = ((column * cell).max(left), (row * cell).max(top));
            cells.push(Cell {
                x,
                y,
                width: ((column + 1) * cell).min(right) - x,
                height: ((row + 1) * cell).min(bottom) - y,
                rgb,
            });
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(x0: f64, y0: f64, x1: f64, y1: f64) -> Mark {
        Mark {
            x0,
            y0,
            x1,
            y1,
            ..Default::default()
        }
    }

    #[test]
    fn a_cell_is_coloured_by_what_is_far_around_it_not_by_itself() {
        // Black but for one white cell, which its own average would leave white.
        let mut pixels = vec![0u8; 120 * 120 * 4];
        for y in 60..72 {
            pixels[(y * 120 + 60) * 4..(y * 120 + 72) * 4].fill(255);
        }
        let screen = Screen::new(&pixels, 120, 120, 1.0);
        let cells = cells(&area(60.0, 60.0, 72.0, 72.0), &screen);
        assert_eq!(cells.len(), 1);
        assert!(cells[0].rgb.iter().all(|&c| c < 64), "{:?}", cells[0].rgb);
    }

    #[test]
    fn a_cell_at_the_edge_averages_only_the_pixels_there_are() {
        // 20 px square: the second column and row of 12 px cells are 8 wide.
        let screen = Screen::new(&[255; 20 * 20 * 4], 20, 20, 1.0);
        let cells = cells(&area(12.0, 12.0, 20.0, 20.0), &screen);
        assert_eq!(cells.len(), 1);
        assert!(
            cells[0].rgb.iter().all(|&c| c >= 255 - GRAIN as u32),
            "{:?}",
            cells[0].rgb
        );
    }

    #[test]
    fn cells_cover_the_area_and_nothing_past_it_at_a_fractional_scale() {
        let pixels = vec![0u8; 200 * 200 * 4];
        let screen = Screen::new(&pixels, 200, 200, 1.25);
        // 38..88 by 13..63 in pixels, dragged from the right.
        let cells = cells(&area(70.4, 10.4, 30.4, 50.4), &screen);
        assert!(cells
            .iter()
            .all(|c| c.x >= 38 && c.x + c.width <= 88 && c.y >= 13 && c.y + c.height <= 63));
        assert_eq!(
            cells.iter().map(|c| c.width * c.height).sum::<usize>(),
            50 * 50
        );
    }
}
