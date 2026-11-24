//! What the interface asks of the desktop.

use crate::{
    desktop,
    ink::{self, Screen},
    modules::clipboard,
    shapes::{ContractValue, Done, Mark, ScreenCapture},
};
use exact_data::storage;
use exact_runner::{
    canvas_capture::captured_image,
    exact_canvas::{Context2d, DrawError, Frame},
    Answer, DataError, DataSource, Outcome, Request, Store, Value,
};
use serde_json::json;
use std::path::PathBuf;

#[derive(Default)]
pub struct Koma {
    /// What pixelation reads, and the showing (`turn`) it is for.
    desktop: Option<(f64, Screen)>,
    /// Where `app:/tmp` is on this machine.
    scratch: PathBuf,
}

impl Koma {
    fn screen(
        &mut self,
        stills: &[ScreenCapture],
        turn: f64,
        scale: f64,
    ) -> Result<&Screen, DrawError> {
        let (_, screen) = match self.desktop.take() {
            Some(kept) if kept.0 == turn => self.desktop.insert(kept),
            _ => {
                let screen = desktop::read(stills, &self.scratch, scale)
                    .map_err(|error| DrawError::Message(error.to_string()))?;
                self.desktop.insert((turn, screen))
            }
        };
        Ok(screen)
    }
}

fn text(value: &Value) -> Result<&str, DataError> {
    value
        .as_str()
        .ok_or_else(|| DataError::BadArguments(format!("{value:?} is not text")))
}

impl DataSource for Koma {
    // `app.json`'s `app.id`: a hand-written host gets no storage without it.
    fn app_id(&self) -> &str {
        "com.anthonyla.koma"
    }

    fn grants(&self) -> &str {
        crate::GRANTS
    }

    fn canvas_surfaces(&self) -> Vec<(String, usize)> {
        vec![("screen".into(), 1), ("ink".into(), 6)]
    }

    fn draw_2d(
        &mut self,
        surface: &str,
        args: &[Value],
        ctx: &Context2d,
        frame: &Frame,
    ) -> Result<bool, DrawError> {
        let bad = || DrawError::Message(format!("`{surface}` cannot be drawn with {args:?}"));
        match (surface, args) {
            ("screen", [stills]) => {
                let stills = Vec::<ScreenCapture>::from_value(stills).ok_or_else(bad)?;
                ink::desktop(ctx, &stills, 0.0, 0.0);
            }
            ("ink", [marks, left, top, stills, turn, scale]) => {
                let (Some(marks), Some(left), Some(top), Some(stills), Some(turn), Some(scale)) = (
                    Vec::<Mark>::from_value(marks),
                    left.as_number(),
                    top.as_number(),
                    Vec::<ScreenCapture>::from_value(stills),
                    turn.as_number(),
                    scale.as_number(),
                ) else {
                    return Err(bad());
                };
                // The canvas keeps what earlier draws left.
                ctx.clear_rect(0.0, 0.0, frame.width, frame.height);
                ink::desktop(ctx, &stills, left, top);
                let screen = if marks.iter().any(|mark| mark.tool == "pixelate") {
                    Some(self.screen(&stills, turn, scale)?)
                } else {
                    None
                };
                ink::paint(ctx, &marks, left, top, screen);
            }
            _ => return Err(bad()),
        }
        Ok(false)
    }

    fn query(&mut self, source: &str, _: &[Value]) -> Result<Value, DataError> {
        Err(DataError::UnknownSource(source.into()))
    }

    fn configure_storage(
        &mut self,
        _: PathBuf,
        _: PathBuf,
        temporary: PathBuf,
    ) -> Result<(), DataError> {
        self.scratch = temporary;
        Ok(())
    }

    fn answer(&mut self, _: &mut Store, source: &str, args: &[Value]) -> Result<Answer, DataError> {
        match (source, args) {
            ("picture", [surface, path]) => Ok(Answer::Later(Request::capture_canvas(
                text(surface)?,
                text(path)?,
            ))),
            ("copy", [path]) => {
                let item = clipboard::ClipboardItem {
                    image_png: Some(text(path)?.into()),
                    ..Default::default()
                };
                let write = clipboard::WriteExactOptions {
                    data: vec![item],
                    ..Default::default()
                };
                Ok(Answer::Later(clipboard::write(&write)?))
            }
            ("save", [path, name]) => {
                let destination = format!("screenshots:/{}", text(name)?);
                let copy = json!({"path": text(path)?, "destination": destination});
                Ok(Answer::Later(storage::request("fs.copyFile", copy)))
            }
            ("forget", [folder]) => {
                self.desktop = None;
                Ok(Answer::Later(storage::request(
                    "fs.rm",
                    json!({"path": text(folder)?}),
                )))
            }
            _ => Err(DataError::UnknownSource(source.into())),
        }
    }

    fn parse(
        &mut self,
        _: &mut Store,
        source: &str,
        _: &[Value],
        outcome: Outcome,
    ) -> Result<Answer, DataError> {
        match source {
            "picture" => {
                captured_image(&outcome).map_err(DataError::Unavailable)?;
            }
            "copy" => clipboard::write_reply(&outcome)?,
            _ => {
                storage::response(outcome).map_err(DataError::Unavailable)?;
            }
        }
        Ok(Answer::Now(Done { ok: true }.to_value()))
    }
}
