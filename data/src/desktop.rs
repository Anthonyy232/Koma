//! The frozen desktop, for what is painted from its pixels.

use crate::{ink::Screen, shapes::ScreenCapture};
use std::{fs::File, io, path::Path};

/// As pixelation reads it. `scratch` is `app:/tmp`; `scale` pixels to a point.
pub fn read(stills: &[ScreenCapture], scratch: &Path, scale: f64) -> io::Result<Screen> {
    let origin = |still: &ScreenCapture| origin(still, scale);
    let far = |edge: &dyn Fn(&ScreenCapture) -> usize| stills.iter().map(edge).max().unwrap_or(0);
    let mut desktop = Desktop::new(
        far(&|still| origin(still).0 + still.width as usize),
        far(&|still| origin(still).1 + still.height as usize),
    );
    for still in stills {
        let name = still.path.trim_start_matches("app:/tmp/");
        let mut reader =
            png::Decoder::new(io::BufReader::new(File::open(scratch.join(name))?)).read_info()?;
        let mut pixels = vec![0; reader.output_buffer_size().unwrap_or_default()];
        let info = reader.next_frame(&mut pixels)?;
        let (x, y) = origin(still);
        desktop.place(
            x,
            y,
            info.width as usize,
            &pixels,
            info.color_type.samples(),
        );
    }
    Ok(Screen::new(
        &desktop.pixels,
        desktop.width,
        desktop.height,
        scale,
    ))
}

/// A capture's stills, laid out as the one desktop the window covers.
struct Desktop {
    /// Opaque RGBA rows, top first.
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Desktop {
    fn new(width: usize, height: usize) -> Desktop {
        Desktop {
            pixels: vec![255; width * height * 4],
            width,
            height,
        }
    }

    /// `pixels`: rows `width` across, `samples` bytes a pixel; top left at (`x`, `y`).
    fn place(&mut self, x: usize, y: usize, width: usize, pixels: &[u8], samples: usize) {
        for (row, line) in pixels.chunks_exact(width * samples).enumerate() {
            let to = ((y + row) * self.width + x) * 4;
            for (pixel, from) in self.pixels[to..to + width * 4]
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(line.chunks_exact(samples))
            {
                pixel[..3].copy_from_slice(&from[..3]);
            }
        }
    }
}

/// Where `still`'s top left is on the desktop, in pixels.
fn origin(still: &ScreenCapture, scale: f64) -> (usize, usize) {
    let pixel = |points: f64| (points * scale).round().max(0.0) as usize;
    match &still.screen.client_rect {
        Some(rect) => (pixel(rect.x), pixel(rect.y)),
        None => (0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_still_lands_on_its_own_pixels_of_the_desktop() {
        let mut desktop = Desktop::new(5, 2);
        // 3 RGBA columns left, 2 RGB right, one row deep.
        desktop.place(0, 0, 3, &[2; 3 * 2 * 4], 4);
        desktop.place(3, 0, 2, &[1; 2 * 3], 3);
        let reds: Vec<u8> = desktop
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[0])
            .collect();
        assert_eq!(reds, [2, 2, 2, 1, 1, 2, 2, 2, 255, 255]);
        assert!(desktop
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| p[3] == 255));
    }
}
