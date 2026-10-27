//! The assistant's command surface for the OneOS host bridge (`craft:command {method: "command.batch"}`).
//!
//! The host relays an allow-listed run: `{label, steps: [{name, args?}]}`. The whole run is judged BEFORE anything runs (an id
//! outside the list, or an argument that names a file or an address, refuses the run), every step runs through the same registry
//! the menus, the command line, the control channel and the MCP server use (so a step the drawing forbids is refused exactly as it
//! would be for the person), and the run is ONE undo step named by `label`: a step that fails takes back every step before it, so
//! a run is all or nothing. There is no file-path, dialog, script or undo command in the list: files move only through the host.
//!
//! This list is the one declared in the host's `platform-craft` registry as `cad` commands (pinned equal there).

use serde_json::{Value, json};

use crate::Session;

/// Registry commands the assistant may run. Each acts at once on the drawing (no dialog, no file picker, no pointer gesture), and
/// none is Undo or Redo: undoing a run is the person's.
pub const ASSISTANT_COMMANDS: &[&str] = &[
    // Draw.
    "line",
    "pline",
    "circle",
    "arc",
    "rectang",
    "polygon",
    "ellipse",
    "point",
    "xline",
    "ray",
    "spline",
    "donut",
    "text",
    "divide",
    "measure",
    "revcloud",
    "hatch",
    "gradient",
    "boundary",
    "region",
    "table",
    "table.set",
    "mleader",
    "leader",
    // Modify.
    "move",
    "copy",
    "rotate",
    "scale",
    "mirror",
    "stretch",
    "offset",
    "trim",
    "extend",
    "fillet",
    "chamfer",
    "explode",
    "break",
    "breakatpoint",
    "join",
    "lengthen",
    "align",
    "blend",
    "reverse",
    "erase",
    "flatten",
    "overkill",
    "textedit",
    "array",
    "arrayrect",
    "arraypolar",
    "union",
    "subtract",
    "intersect",
    "group",
    "ungroup",
    "draworder",
    // Dimension.
    "dimlinear",
    "dimaligned",
    "dimradius",
    "dimdiameter",
    "dimangular",
    "dimarc",
    "dimcontinue",
    "dimbaseline",
    // Properties, layers and styles.
    "properties.set",
    "matchprop",
    "color",
    "lweight",
    "ltscale",
    "layer.new",
    "layer.set",
    "layer.current",
    "layer.delete",
    "laymch",
    "laycur",
    "layiso",
    "layfrz",
    "layoff",
    "laylck",
    "layulk",
    "style",
    "style.current",
    "dimstyle",
    "dimstyle.current",
    "block",
    "units",
    "limits",
    // Choosing what the next step acts on.
    "select",
    "selectall",
    "ai_deselect",
    // The view (changes nothing in the drawing).
    "zoom",
    "pan",
    "view.set",
    // Read-only.
    "list",
    "dist",
    "area",
    "id",
    "properties",
    "entities",
    "texts",
    "drawing.inspect",
    "layer",
    "style.list",
    "dimstyle.list",
    "groups.list",
    "measuregeom",
    "getvar",
];

/// Commands the window itself provides.
pub const ARGUMENT_COMMANDS: &[&str] = &[
    // Read-only: the drawing as a PNG, base64, fitted to its extents (the drawing's own EXPORT, with no path).
    "render",
];

/// Every command id a run may name, in the order the host's registry declares them.
pub fn allowed() -> Vec<&'static str> {
    ASSISTANT_COMMANDS.iter().chain(ARGUMENT_COMMANDS).copied().collect()
}

const MAX_STEPS: usize = 50;
const MAX_LABEL: usize = 120;
/// Argument names that would point a command at a file or a network address: refused anywhere in a step's arguments.
const FORBIDDEN_ARGS: &[&str] =
    &["path", "dir", "folder", "file", "url", "uri", "href", "server", "data", "template", "patternFile", "patternText", "text_file"];

/// The first forbidden argument name anywhere in `v`.
fn forbidden_arg(v: &Value) -> Option<String> {
    match v {
        Value::Object(o) => o.iter().find_map(|(k, x)| if FORBIDDEN_ARGS.contains(&k.as_str()) { Some(k.clone()) } else { forbidden_arg(x) }),
        Value::Array(a) => a.iter().find_map(forbidden_arg),
        _ => None,
    }
}

impl Session {
    /// One `command.batch`: all of it, as ONE undo step, or none of it.
    pub fn run_command_batch(&mut self, params: &Value) -> Result<Value, String> {
        let label = params.get("label").and_then(Value::as_str).map(str::trim).unwrap_or("");
        if label.chars().count() > MAX_LABEL {
            return Err(format!("label-too-long: a run's label is at most {MAX_LABEL} characters"));
        }
        let steps = params.get("steps").and_then(Value::as_array).ok_or("run-empty: a run states its steps")?;
        if steps.is_empty() {
            return Err("run-empty: a run states at least one command".into());
        }
        if steps.len() > MAX_STEPS {
            return Err(format!("run-too-long: a run is at most {MAX_STEPS} steps"));
        }
        // Judge the whole run first; nothing has run when one id is outside the list.
        let mut named = Vec::with_capacity(steps.len());
        for step in steps {
            let name = step.get("name").and_then(Value::as_str).unwrap_or("");
            if !ASSISTANT_COMMANDS.contains(&name) && !ARGUMENT_COMMANDS.contains(&name) {
                return Err(format!("command-not-allowed: CAD does not allow the command '{name}' -- nothing was run"));
            }
            let args = step.get("args").cloned().unwrap_or_else(|| json!({}));
            if let Some(k) = forbidden_arg(&args) {
                return Err(format!(
                    "argument-not-allowed: step '{name}' names '{k}', and a run does not reach files or addresses -- nothing was run"
                ));
            }
            named.push((name, args));
        }
        self.state().map_err(|_| "no-document: no drawing is open in this window".to_string())?;
        if self.running.is_some() {
            return Err("command-running: a command is waiting for its next answer -- finish or cancel it first; nothing was run".into());
        }
        let label = if label.is_empty() { "Assistant" } else { label };

        // What to put back when a step fails: the drawing, its selection, both history stacks and the group the run opens.
        let uid = self.state().map(|s| s.uid).unwrap_or(0);
        let (doc, selection, undo, redo, undo_ctl, revision) = {
            let st = self.state().map_err(|e| e.to_string())?;
            (st.doc.clone(), st.selection.clone(), st.undo.clone(), st.redo.clone(), st.undo_ctl.clone(), st.revision)
        };
        let depth = undo.len();
        let mut results: Vec<Value> = Vec::with_capacity(named.len());
        let mut failure: Option<String> = None;
        for (i, (name, args)) in named.iter().enumerate() {
            let outcome = match *name {
                "render" => self.execute("export", &json!({ "format": "png" })),
                other => self.execute(other, args),
            };
            match outcome {
                Ok(v) => results.push(json!({ "name": name, "result": v })),
                Err(e) => {
                    failure = Some(format!("step-failed: step {} ({name}): {e} -- nothing was changed", i + 1));
                    break;
                }
            }
        }
        if let Some(why) = failure {
            if let Some(st) = self.docs.iter_mut().find(|d| d.uid == uid) {
                st.doc = doc;
                st.selection = selection;
                st.undo = undo;
                st.redo = redo;
                st.undo_ctl = undo_ctl;
                st.revision = revision + 1;
            }
            return Err(why);
        }
        // The steps pushed one undo step each; fold them into the one the run is. The first holds the drawing as it was before the
        // run, so the folded step goes back to exactly there.
        let mut folded = false;
        if let Some(st) = self.docs.iter_mut().find(|d| d.uid == uid)
            && st.undo.len() > depth
        {
            st.undo.truncate(depth + 1);
            if let Some(first) = st.undo.get_mut(depth) {
                first.label = label.to_string();
            }
            st.undo_ctl.marks.retain(|m| *m <= depth + 1);
            folded = true;
        }
        Ok(json!({ "label": label, "steps": results, "undoStep": folded }))
    }
}
