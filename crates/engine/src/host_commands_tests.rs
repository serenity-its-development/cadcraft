use serde_json::{Value, json};

use crate::Session;
use crate::cmd::find_command;
use crate::host_commands::{ARGUMENT_COMMANDS, ASSISTANT_COMMANDS, LEFT_OUT, allowed};

fn count(s: &Session) -> usize {
    s.doc().unwrap().model.len()
}

fn batch(s: &mut Session, label: &str, steps: Value) -> Result<Value, String> {
    s.run_command_batch(&json!({ "label": label, "steps": steps }))
}

#[test]
fn every_listed_command_is_a_registry_command_and_none_reaches_a_file_or_history() {
    for id in ASSISTANT_COMMANDS {
        assert!(find_command(id).is_some(), "{id} is not a registry command");
        assert!(!matches!(*id, "open" | "new" | "close" | "saveas" | "qsave" | "script" | "lisp" | "undo" | "redo" | "u" | "import" | "export"), "{id}");
    }
    assert_eq!(ARGUMENT_COMMANDS, &["render"]);
    assert_eq!(allowed().len(), ASSISTANT_COMMANDS.len() + 1);
    let mut seen = std::collections::BTreeSet::new();
    for id in allowed() {
        assert!(seen.insert(id), "{id} is listed twice");
    }
}

#[test]
fn a_run_is_one_undo_step_named_by_its_label() {
    let mut s = Session::new();
    let r = batch(
        &mut s,
        "Draw a room",
        json!([
            { "name": "rectang", "args": { "p1": [0, 0], "p2": [400, 300] } },
            { "name": "circle", "args": { "center": [200, 150], "radius": 40 } },
            { "name": "text", "args": { "at": [10, 10], "text": "Room 1", "height": 12 } },
        ]),
    )
    .unwrap();
    assert_eq!(r["steps"].as_array().unwrap().len(), 3);
    assert_eq!(r["undoStep"], true);
    assert_eq!(count(&s), 3);
    let undo_steps = s.state().unwrap().undo.len();
    assert_eq!(undo_steps, 1, "the three steps are one");
    assert_eq!(s.state().unwrap().undo[0].label, "Draw a room");
    s.undo().unwrap();
    assert_eq!(count(&s), 0, "one undo takes the whole run back");
}

#[test]
fn a_failing_step_takes_back_the_steps_before_it() {
    let mut s = Session::new();
    s.execute("line", &json!({ "points": [[0, 0], [10, 0]] })).unwrap();
    let before = s.state().unwrap().undo.len();
    let e = batch(
        &mut s,
        "Half done",
        json!([
            { "name": "circle", "args": { "center": [0, 0], "radius": 5 } },
            { "name": "circle", "args": { "center": [0, 0], "radius": -1 } },
        ]),
    )
    .unwrap_err();
    assert!(e.starts_with("step-failed: step 2"), "{e}");
    assert_eq!(count(&s), 1, "the first circle is gone again");
    assert_eq!(s.state().unwrap().undo.len(), before);
}

#[test]
fn what_is_not_listed_is_refused_whole_and_nothing_runs() {
    let mut s = Session::new();
    for (steps, code) in [
        (json!([{ "name": "circle", "args": { "center": [0, 0], "radius": 5 } }, { "name": "saveas", "args": {} }]), "command-not-allowed"),
        (json!([{ "name": "undo" }]), "command-not-allowed"),
        (json!([{ "name": "script", "args": {} }]), "command-not-allowed"),
        (json!([{ "name": "circle", "args": { "center": [0, 0], "radius": 5, "file": "x.dxf" } }]), "argument-not-allowed"),
        (json!([{ "name": "hatch", "args": { "patternFile": "x.pat" } }]), "argument-not-allowed"),
        (json!([]), "run-empty"),
    ] {
        let e = batch(&mut s, "x", steps).unwrap_err();
        assert!(e.starts_with(code), "{e}");
        assert_eq!(count(&s), 0, "{e}");
    }
    let e = batch(&mut s, &"l".repeat(121), json!([{ "name": "id", "args": { "at": [0, 0] } }])).unwrap_err();
    assert!(e.starts_with("label-too-long"));
    let many: Vec<Value> = (0..51).map(|_| json!({ "name": "id", "args": { "at": [0, 0] } })).collect();
    assert!(batch(&mut s, "x", Value::Array(many)).unwrap_err().starts_with("run-too-long"));
}

#[test]
fn a_read_only_run_leaves_no_undo_step_and_reads_back_what_was_drawn() {
    let mut s = Session::new();
    batch(&mut s, "Draw", json!([{ "name": "layer.new", "args": { "name": "Walls", "color": "red" } }, { "name": "line", "args": { "points": [[0, 0], [100, 0]] } }])).unwrap();
    let depth = s.state().unwrap().undo.len();
    let r = batch(&mut s, "Look", json!([{ "name": "drawing.inspect", "args": {} }, { "name": "entities", "args": { "type": "Line" } }])).unwrap();
    assert_eq!(r["undoStep"], false);
    assert_eq!(s.state().unwrap().undo.len(), depth);
    assert_eq!(r["steps"][1]["result"]["count"], 1);
}

#[test]
fn a_run_waits_for_a_command_in_progress() {
    let mut s = Session::new();
    s.cmdline("LINE").unwrap();
    let e = batch(&mut s, "x", json!([{ "name": "id", "args": { "at": [0, 0] } }])).unwrap_err();
    assert!(e.starts_with("command-running"), "{e}");
}

#[test]
fn every_registry_command_is_allowed_or_left_out_with_a_reason() {
    let registry: std::collections::BTreeSet<&str> = crate::cmd::command_specs().iter().map(|s| s.id).collect();
    let allowed: std::collections::BTreeSet<&str> = ASSISTANT_COMMANDS.iter().copied().collect();
    let left: Vec<&str> = LEFT_OUT.iter().flat_map(|(_, ids)| ids.iter().copied()).collect();
    for (reason, ids) in LEFT_OUT {
        assert!(reason.len() > 20, "a command left out states why: {ids:?}");
    }
    let left_set: std::collections::BTreeSet<&str> = left.iter().copied().collect();
    assert_eq!(left.len(), left_set.len(), "a command is left out under two reasons");
    assert!(allowed.is_disjoint(&left_set), "allowed and left out overlap: {:?}", allowed.intersection(&left_set).collect::<Vec<_>>());
    let accounted: std::collections::BTreeSet<&str> = allowed.union(&left_set).copied().collect();
    let missing: Vec<&&str> = registry.difference(&accounted).collect();
    let unknown: Vec<&&str> = accounted.difference(&registry).collect();
    assert!(missing.is_empty(), "registry commands neither allowed nor left out: {missing:?}");
    assert!(unknown.is_empty(), "listed commands the registry does not have: {unknown:?}");
    assert_eq!(registry.len(), allowed.len() + left_set.len());
}
