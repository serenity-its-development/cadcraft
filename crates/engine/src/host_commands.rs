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

/// Registry commands the assistant may run: EVERY command of the registry except those in [`LEFT_OUT`], each with its reason.
/// The drawing's whole command registry (draw, modify, annotate, layers, styles, blocks, tables, layouts, view, inquiry). A step the document's state forbids, or one that needs a dialog or a pointer, is refused by the registry exactly as it would be for
/// the person, and the run is taken back whole.
pub const ASSISTANT_COMMANDS: &[&str] = &[
    // general
    "scalelistedit",
    "objectscale",
    "aiobjectscaleadd",
    "aiobjectscaleremove",
    "annoreset",
    "annoupdate",
    // annovisible
    "annovisible.current",
    "annovisible.all",
    "annovisible.toggle",
    // general
    "dimlinear",
    "dimaligned",
    "dimradius",
    "dimdiameter",
    "dimangular",
    "dimarc",
    "dimordinate",
    "dimcontinue",
    "dimbaseline",
    "qdim",
    "dim",
    "mleader",
    "leader",
    // dimstyle
    "dimstyle.update",
    // general
    "dimoverride",
    "dimreassociate",
    "dimdisassociate",
    // dimtedit
    "dimtedit.home",
    "dimtedit.angle",
    "dimtedit.left",
    "dimtedit.center",
    "dimtedit.right",
    // general
    "dimtedit",
    "dimspace",
    "dimedit",
    // dimedit
    "dimedit.oblique",
    // general
    "dimjogged",
    "dimjogline",
    "dimbreak",
    "diminspect",
    "arrayedit",
    "arrayclose",
    "array",
    "arrayrect",
    "arraypolar",
    "audit",
    "attedit",
    "attdisp",
    // attdisp
    "attdisp.normal",
    "attdisp.on",
    "attdisp.off",
    // general
    "attsync",
    "bedit",
    "bsave",
    "bsaveas",
    "bclose",
    "blockreplace",
    "burst",
    "eattext",
    "refedit",
    "refset",
    "refclose",
    "xbind",
    "block",
    "insert",
    "attdef",
    "base",
    // blocks
    "blocks.dynamic",
    "blocks.reference",
    "blocks.reference.set",
    "blocks.reference.grips",
    "blocks.reference.grip",
    // general
    "purge",
    // blocks
    "blocks.list",
    // general
    "geomconstraint",
    "autoconstrain",
    "constraintbar",
    // constraintbar
    "constraintbar.showall",
    "constraintbar.hideall",
    // general
    "dimconstraint",
    "dclinear",
    "dcdisplay",
    // dcdisplay
    "dcdisplay.showall",
    "dcdisplay.hideall",
    // general
    "delconstraint",
    "parameters",
    "constraintsettings",
    "constraintinfer",
    // constraints
    "constraints.inspect",
    "constraints.dof",
    // general
    "line",
    "pline",
    "circle",
    "arc",
    "rectang",
    "polygon",
    "ellipse",
    "point",
    // point
    "point.multiple",
    // general
    "xline",
    "ray",
    "spline",
    "donut",
    "text",
    "mtext",
    "breakline",
    "mline",
    "mlstyle",
    "mledit",
    // arc
    "arc.sce",
    "arc.sca",
    "arc.scl",
    "arc.sea",
    "arc.sed",
    "arc.ser",
    "arc.cse",
    "arc.csa",
    "arc.csl",
    "arc.continue",
    // circle
    "circle.cd",
    "circle.2p",
    "circle.3p",
    "circle.ttr",
    "circle.ttt",
    // ellipse
    "ellipse.axis",
    "ellipse.arc",
    // general
    "divide",
    "measure",
    "revcloud",
    // revcloud
    "revcloud.polygonal",
    "revcloud.freehand",
    // general
    "wipeout",
    "3dpoly",
    "helix",
    // spline
    "spline.cv",
    // general
    "centermark",
    "centerline",
    "draworder",
    // draworder
    "draworder.front",
    "draworder.back",
    "draworder.above",
    "draworder.under",
    // general
    "texttofront",
    // texttofront
    "texttofront.text",
    "texttofront.dimensions",
    "texttofront.leaders",
    "texttofront.all",
    // general
    "hatchtoback",
    "cutclip",
    "cutbase",
    "copyclip",
    "copybase",
    "pasteclip",
    "pasteblock",
    "pasteorig",
    // erase
    "erase.selection",
    // general
    "selectall",
    "ai_deselect",
    "select",
    "preview",
    // grip
    "grip.move",
    "grip.stretch",
    "grip.rotate",
    "grip.scale",
    "grip.mirror",
    "grip.option",
    // general
    "grip",
    "pickstyle",
    "group",
    "ungroup",
    "groupedit",
    // groups
    "groups.list",
    // general
    "hatch",
    "gradient",
    "boundary",
    "region",
    "hatchedit",
    "hatchpick",
    "union",
    "subtract",
    "intersect",
    "massprop",
    "dist",
    "id",
    "list",
    "area",
    "time",
    "status",
    // drawing
    "drawing.inspect",
    // general
    "entities",
    "shapes",
    "usage",
    "loops",
    "texts",
    "countlist",
    "counttable",
    "count",
    "layer",
    // layer
    "layer.new",
    "layer.set",
    "layer.current",
    "layer.delete",
    // general
    "laymcur",
    "laymch",
    "laycur",
    "layiso",
    "layuniso",
    "layfrz",
    "layoff",
    "laylck",
    "layulk",
    "layon",
    "laythw",
    "layerp",
    // layerstate
    "layerstate.save",
    "layerstate.restore",
    "layerstate.list",
    "layerstate.delete",
    "layerstate.rename",
    // general
    "laymrg",
    "laydel",
    "plotstyle",
    "layout",
    // layout
    "layout.new",
    "layout.delete",
    "layout.rename",
    "layout.copy",
    "layout.list",
    // general
    "mview",
    // mview
    "mview.polygonal",
    // general
    "vpclip",
    "vports",
    // vports
    "vports.1",
    "vports.2",
    "vports.3",
    "vports.4",
    // viewport
    "viewport.set",
    // general
    "vplayer",
    "pagesetup",
    "mspace",
    "pspace",
    "mleaderedit",
    // mleaderedit
    "mleaderedit.add",
    "mleaderedit.remove",
    // general
    "mleaderalign",
    "mleadercollect",
    "erase",
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
    "pedit",
    "splinedit",
    "lengthen",
    "align",
    "blend",
    "reverse",
    "ncopy",
    "mocoro",
    "chspace",
    "flatten",
    "arraypath",
    "textedit",
    "overkill",
    "properties",
    // properties
    "properties.set",
    // general
    "matchprop",
    "color",
    "linetype",
    // hatch
    "hatch.patterns",
    // general
    "lweight",
    "ltscale",
    "units",
    "thickness",
    "limits",
    "style",
    // style
    "style.list",
    "style.rename",
    "style.delete",
    "style.current",
    // general
    "dimstyle",
    // dimstyle
    "dimstyle.dimension",
    "dimstyle.list",
    "dimstyle.rename",
    "dimstyle.delete",
    "dimstyle.current",
    "dimstyle.override",
    // general
    "tablestyle",
    "mleaderstyle",
    "ddptype",
    "rename",
    "qselect",
    // qselect
    "qselect.info",
    // general
    "selectsimilar",
    "isolateobjects",
    "hideobjects",
    "unisolateobjects",
    "ortho",
    "grid",
    "snap",
    "polar",
    "otrack",
    "dynmode",
    "lwdisplay",
    "isodraft",
    "isoplane",
    "transparencydisplay",
    "qpmode",
    "selectioncycling",
    "osnap",
    "setvar",
    "getvar",
    "sysvars",
    // table
    "table.break",
    // tablestyle
    "tablestyle.cellstyle",
    // table
    "table.cellstyle",
    "table.formula",
    "table.recalc",
    // general
    "tabledit",
    "table",
    // table
    "table.set",
    "table.insertrow",
    "table.deleterow",
    "table.insertcol",
    "table.deletecol",
    "table.merge",
    "table.unmerge",
    // general
    "tolerance",
    "txt2mtxt",
    "cal",
    "find",
    "measuregeom",
    // radius
    "radius.measure",
    // angle
    "angle.measure",
    // volume
    "volume.measure",
    // general
    "view",
    // vports
    "vports.activate",
    // general
    "zoom",
    // zoom
    "zoom.realtime",
    "zoom.dynamic",
    "zoom.previous",
    "zoom.window",
    "zoom.in",
    "zoom.out",
    "zoom.all",
    "zoom.extents",
    "zoom.object",
    // general
    "pan",
    // pan
    "pan.left",
    "pan.right",
    "pan.up",
    "pan.down",
    // general
    "regen",
    "regenall",
    "redraw",
    // view
    "view.set",
    "view.get",
    // layout
    "layout.set",
    // general
    "dcaligned",
    "dcangular",
    "dcdiameter",
    "dchorizontal",
    "dcradius",
    "dcvertical",
    "gccoincident",
    "gccollinear",
    "gcconcentric",
    "gcequal",
    "gcfix",
    "gchorizontal",
    "gcparallel",
    "gcperpendicular",
    "gcsmooth",
    "gcsymmetric",
    "gctangent",
    "gcvertical",
];

/// Commands the window itself provides.
pub const ARGUMENT_COMMANDS: &[&str] = &[
    // Read-only: the drawing as a PNG, base64, fitted to its extents (the drawing's own EXPORT, with no path).
    "render",
];

/// Registry commands left OUT of the allow-list, grouped by the reason (the audit lists them; every registry command is in exactly one of the two).
pub const LEFT_OUT: &[(&str, &[&str])] = &[
    (
        "files: opens, saves, exports, imports, plots or attaches a file, or runs a script or LISP program; or needs a second drawing file (files move only through the host)",
        &["recover", "drawingrecovery", "dataextraction", "xattach", "xref", "xopen", "wblock", "import", "export", "publish", "pdfbook", "attach", "compareexport", "compare", "compare.next", "compare.previous", "compareclose", "aliasedit", "reinit", "new", "open", "close", "closeall", "qsave", "saveas", "document.switch", "document.bytes", "script", "plot", "exportpdf", "appload", "lisp", "plotwindow", "plottermanager", "datalink", "datalinkupdate", "tableexport"],
    ),
    (
        "history: Undo and Redo are the person's",
        &["u", "undo", "redo", "mredo"],
    ),
    (
        "command-line prompt forms (a leading dash): the same command is listed without the dash and takes JSON arguments",
        &["-attedit", "-xref", "-block", "-group", "-plottermanager", "-linetype", "-vports"],
    ),
    (
        "not available in this build (the engine answers that it is not implemented)",
        &["share", "3dorbit"],
    ),
    (
        "opens a dialog or palette of the application, not an edit of the drawing",
        &["battman", "stylesmanager", "options", "dsettings", "quickcalc.pick"],
    ),
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
