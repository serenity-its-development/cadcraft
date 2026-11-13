//! The OneOS host's ask for a drawing's bytes (`craft:export`: auto-save, save on close) is never a save: the bytes are what Save
//! writes for the host's file, and the drawing is as it was (unsaved, same name and title, nothing stored or logged).

use cadcraft_engine::Session;
use cadcraft_engine::cmd::file::{IoHooks, host_export_bytes, set_io, set_write_as};
use serde_json::json;

fn real_formats() {
    // The same hooks the browser build installs (`apps/cadcraft-web/src/main.rs`).
    set_io(IoHooks {
        read: |b, name| cadcraft_io::read(b, name).map_err(|e| e.to_string()),
        write: |d, name| cadcraft_io::write(d, name).map_err(|e| e.to_string()),
        plot: None,
    });
    set_write_as(|d, name, o| cadcraft_io::write_with(d, name, &cadcraft_io::SaveOptions { version: o.version, binary: o.binary }).map_err(|e| e.to_string()));
}

#[test]
fn an_export_for_the_host_is_the_bytes_of_a_save_and_leaves_the_drawing_unsaved() {
    real_formats();
    let source = {
        let mut s = Session::new();
        s.cmdline("line 0,0 10,10").unwrap();
        let r = s.execute("document.bytes", &json!({ "format": "dxf" })).unwrap();
        cadcraft_engine::cmd::file::base64_decode(r["data"].as_str().unwrap()).unwrap()
    };
    let mut s = Session::empty();
    assert!(host_export_bytes(&s, "plan.dxf").is_err(), "nothing is open yet");
    s.execute("open", &json!({ "data": cadcraft_engine::cmd::file::base64_encode(&source), "name": "plan.dxf" })).unwrap();
    let opened = host_export_bytes(&s, "plan.dxf").unwrap();
    s.cmdline("circle 4,4 1").unwrap();
    assert!(host_export_bytes(&s, "other.dxf").is_err(), "only the host's own file is exported");
    assert!(host_export_bytes(&s, "plan.pdf").is_err(), "never as another format");

    let before = {
        let st = s.state().unwrap();
        (st.path.clone(), st.title.clone(), st.is_dirty(), st.doc.clone())
    };
    assert!(before.2, "the edit is unsaved");
    let log = s.log.len();
    let first = host_export_bytes(&s, "plan.dxf").unwrap();
    let second = host_export_bytes(&s, "plan.dxf").unwrap();
    assert_eq!(first, second, "asked twice, the same bytes");
    assert_ne!(first, opened, "with the work not saved yet");
    let st = s.state().unwrap();
    assert_eq!((st.path.clone(), st.title.clone(), st.is_dirty()), (before.0, before.1, true), "an export is not a save");
    assert!(std::sync::Arc::ptr_eq(&st.doc, &before.3), "the very same drawing");
    assert_eq!(s.log.len(), log, "nothing was said");

    // The drawing read back holds the edit.
    let entities = |b: &[u8]| cadcraft_io::read(b, "plan.dxf").unwrap().model.len();
    assert_eq!(entities(&first), entities(&opened) + 1, "the circle is in it");

    // The person's Save of the same drawing writes exactly those bytes.
    let path = std::env::temp_dir().join(format!("cadcraft-host-export-{}", std::process::id())).join("plan.dxf");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    s.execute("saveas", &json!({ "path": path.to_string_lossy() })).unwrap();
    let saved = std::fs::read(&path).unwrap();
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
    assert_eq!(saved, first);
}
