use crate::builder::canvas_load::load_from_patch;

// ---------------------------------------------------------------------------
// C3 — Instance with unknown template silently skipped (silent data loss)
// ---------------------------------------------------------------------------

/// An instance that references a template not defined anywhere in the .patch
/// file must return a validation error. Previously it was silently skipped,
/// which caused the canvas to render an empty project without any indication
/// that devices were lost.
#[test]
fn load_instance_with_unknown_template_returns_error() {
    let patch = r#"
instance FOH is UnknownConsole {
  location: "Front of House"
}
"#;
    let result = load_from_patch(patch, "");
    assert!(
        result.is_err(),
        "expected error for unknown template, got Ok with {} instances",
        result.as_ref().map(|o| o.instances.len()).unwrap_or(0)
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("UnknownConsole"),
        "error should name the missing template, got: {err}"
    );
}

/// Multiple instances, some with known templates and some with unknown — the
/// first unknown template encountered must trigger an error rather than being
/// silently discarded.
#[test]
fn load_mixed_known_and_unknown_templates_returns_error() {
    let patch = r#"
template Rio3224 {
  meta { manufacturer: "Yamaha", model: "Rio3224" }
  ports {
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
  }
}
instance SL is Rio3224
instance FOH is UndefinedConsole {
  location: "Front of House"
}
"#;
    let result = load_from_patch(patch, "");
    assert!(
        result.is_err(),
        "expected error for unknown template 'UndefinedConsole'"
    );
}

// ---------------------------------------------------------------------------
// C4 — Streams orphaned when source has no instance qualifier
// ---------------------------------------------------------------------------

/// A stream declaration with no `source:` line has no instance to attach to.
/// This is invalid PatchLang — the canvas can't display a sourceless stream.
/// Previously the stream was silently dropped; now it must return an error.
#[test]
fn load_stream_with_no_source_is_not_silently_dropped() {
    let patch = r#"
template Rio3224 {
  meta { manufacturer: "Yamaha", model: "Rio3224" }
  ports {
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
  }
}
instance SL is Rio3224

stream My_Stream {
  channels: 32
  protocol: "Dante"
}
"#;
    // A sourceless stream can't be attached to any instance — must be an error.
    let result = load_from_patch(patch, "");
    assert!(
        result.is_err(),
        "stream with no source should return an error, not silently drop"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("My_Stream"),
        "error should name the orphaned stream, got: {err}"
    );
}
