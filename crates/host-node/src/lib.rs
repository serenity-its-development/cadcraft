//! One CAD editor window's host side for node: the engine the browser build runs, with the file formats it reads and writes, so OneOS's
//! `craft-cad-ai-drive.test.ts` can drive a window through the SAME `command.batch` the browser build answers `craft:command` with
//! (`cadcraft_engine::host_commands`), and read back the DXF or DWG the editor saves.

use cadcraft_engine::Session;
use cadcraft_engine::cmd::file::{EntityCodec, IoHooks, base64_decode, base64_encode, set_entity_codec, set_io, set_write_as};
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

fn err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

#[wasm_bindgen]
pub struct Window {
    session: Session,
}

#[wasm_bindgen]
impl Window {
    /// A window with a new, empty drawing (metric when `metric`).
    #[wasm_bindgen(constructor)]
    pub fn new(metric: bool) -> Window {
        // The same hooks the browser build installs (`apps/cadcraft-web/src/main.rs`).
        set_io(IoHooks {
            read: |b, name| cadcraft_io::read(b, name).map_err(|e| e.to_string()),
            write: |d, name| cadcraft_io::write(d, name).map_err(|e| e.to_string()),
            plot: Some(|d, space, opts| cadcraft_io::plotter::plot(d, space, opts).map_err(|e| e.to_string())),
        });
        set_write_as(|d, name, o| cadcraft_io::write_with(d, name, &cadcraft_io::SaveOptions { version: o.version, binary: o.binary }).map_err(|e| e.to_string()));
        set_entity_codec(EntityCodec { groups: cadcraft_io::entity_groups, entity: cadcraft_io::entity_from_groups });
        let mut session = Session::empty();
        session.new_drawing(metric);
        Window { session }
    }

    /// Open a drawing the host sent (`craft:open`): a DXF or a DWG, base64.
    pub fn open_b64(&mut self, name: &str, b64: &str) -> Result<(), JsError> {
        self.session.execute("open", &json!({ "data": b64, "name": name })).map(|_| ()).map_err(err)
    }

    /// The assistant's run (`craft:command`, `command.batch`): the answer as JSON text, or an error `<code>: <reason>`.
    pub fn command_batch(&mut self, params_json: &str) -> Result<String, JsError> {
        let params: Value = serde_json::from_str(params_json).map_err(err)?;
        self.session.run_command_batch(&params).map(|v| v.to_string()).map_err(|e| JsError::new(&e))
    }

    /// The drawing as the editor saves it: `ext` is `dxf` or `dwg`; base64.
    pub fn file_b64(&mut self, ext: &str) -> Result<String, JsError> {
        let r = self.session.execute("export", &json!({ "format": ext })).map_err(err)?;
        r["data"].as_str().map(str::to_owned).ok_or_else(|| JsError::new("the export made no data"))
    }

    /// How many undo steps the open drawing holds.
    pub fn undo_depth(&self) -> usize {
        self.session.state().map(|s| s.undo.len()).unwrap_or(0)
    }

    /// Undo one step (the person's Undo).
    pub fn undo(&mut self) -> Result<(), JsError> {
        self.session.undo().map(|_| ()).map_err(err)
    }

    /// Whether the drawing has unsaved changes.
    pub fn dirty(&self) -> bool {
        self.session.state().is_ok_and(|s| s.is_dirty())
    }
}

/// `base64` of `bytes`, for a test that has a file in hand.
#[wasm_bindgen]
pub fn to_b64(bytes: &[u8]) -> String {
    base64_encode(bytes)
}

/// The bytes of a base64 text.
#[wasm_bindgen]
pub fn from_b64(b64: &str) -> Result<Vec<u8>, JsError> {
    base64_decode(b64).ok_or_else(|| JsError::new("not base64"))
}
