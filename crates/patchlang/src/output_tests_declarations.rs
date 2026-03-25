//! Output tests for declaration statement types: Signal, Flag, Stream, Config,
//! Use, Ring, and multi-file compile_project.
//!
//! Each test drives `patchlang::check()` (or `compile_project`) with a
//! concrete PatchLang snippet and asserts on the serialized JSON shape.

#[cfg(test)]
mod tests {
    use crate::check;
    use crate::multi_file::compile_project;
    use crate::output_test_helpers::{all_stmts_of as all_stmts_of_type, first_stmt_of as first_stmt_of_type, get_json};
    use std::collections::HashMap;

    // ── Test 1: Signal declaration ───────────────────────────────────

    #[test]
    fn signal_declaration_fields() {
        let src = r#"
template Dev { ports { Mic_In[1..4]: in } }
instance SL is Dev
signal Lead_Vocal {
  origin: SL.Mic_In[1]
  channel: "1"
  description: "Worship leader vocal"
}
"#;
        let json = get_json(src);
        let stmt = first_stmt_of_type(&json, "Signal");

        assert_eq!(stmt["type"], "Signal");
        assert_eq!(stmt["name"], "Lead_Vocal");
        assert_eq!(stmt["properties"]["channel"], "1");
        assert_eq!(stmt["properties"]["description"], "Worship leader vocal");

        let origin = &stmt["origin"];
        assert_eq!(origin["instance"], "SL");
        assert_eq!(origin["port"], "Mic_In");

        let index_spec = origin["indexSpec"].as_array().unwrap();
        assert_eq!(index_spec.len(), 1);
        assert_eq!(index_spec[0]["type"], "single");
        assert_eq!(index_spec[0]["value"], 1);
    }

    // ── Test 2: Flag declaration ──────────────────────────────────────

    #[test]
    fn flag_declaration_fields() {
        let src = r#"
flag Genlock_OK {
  description: "All cameras locked to house sync"
  severity: "warning"
}
"#;
        let json = get_json(src);
        let stmt = first_stmt_of_type(&json, "Flag");

        assert_eq!(stmt["type"], "Flag");
        assert_eq!(stmt["name"], "Genlock_OK");
        assert_eq!(
            stmt["properties"]["description"],
            "All cameras locked to house sync"
        );
        assert_eq!(stmt["properties"]["severity"], "warning");
    }

    // ── Test 3: Stream declaration with source ────────────────────────

    #[test]
    fn stream_declaration_with_source() {
        let src = r#"
template Dev { ports { Dante_Out[1..32]: out } }
instance SL is Dev
stream SL_Dante_Primary {
  source: SL.Dante_Out
  channels: "32"
  protocol: "Dante"
}
"#;
        let json = get_json(src);
        let stmt = first_stmt_of_type(&json, "Stream");

        assert_eq!(stmt["type"], "Stream");
        assert_eq!(stmt["name"], "SL_Dante_Primary");
        assert_eq!(stmt["properties"]["channels"], "32");
        assert_eq!(stmt["properties"]["protocol"], "Dante");

        let source = &stmt["source"];
        assert_eq!(source["instance"], "SL");
        assert_eq!(source["port"], "Dante_Out");
    }

    // ── Test 4: Stream without body ───────────────────────────────────

    #[test]
    fn stream_declaration_without_body() {
        let src = "stream Ambient_Feed";
        let json = get_json(src);
        let stmt = first_stmt_of_type(&json, "Stream");

        assert_eq!(stmt["type"], "Stream");
        assert_eq!(stmt["name"], "Ambient_Feed");

        // properties must be an empty object
        assert_eq!(
            stmt["properties"],
            serde_json::Value::Object(serde_json::Map::new())
        );

        // source must be absent (skip_serializing_if = Option::is_none)
        assert!(
            stmt.get("source").is_none() || stmt["source"].is_null(),
            "expected no source field, got: {}",
            stmt["source"]
        );
    }

    // ── Test 5: Config with labels and properties ─────────────────────

    #[test]
    fn config_declaration_labels_and_properties() {
        let src = r#"
template Dev { ports { Dante_In[1..72]: in } }
instance FOH is Dev
config FOH {
  label Dante_In[1]: "Lead Vocal" { phantom: "true", stand: "Short Boom" }
  label Dante_In[2]: "Kick Drum"
}
"#;
        let json = get_json(src);
        let stmt = first_stmt_of_type(&json, "Config");

        assert_eq!(stmt["type"], "Config");
        assert_eq!(stmt["name"], "FOH");

        let labels = stmt["labels"].as_array().unwrap();
        assert_eq!(labels.len(), 2, "expected 2 labels, got {}", labels.len());

        // First label
        let lbl0 = &labels[0];
        assert_eq!(lbl0["label"], "Lead Vocal");
        assert_eq!(lbl0["port"]["port"], "Dante_In");
        assert_eq!(lbl0["port"]["instance"], "");

        let index_spec_0 = lbl0["port"]["indexSpec"].as_array().unwrap();
        assert_eq!(index_spec_0.len(), 1);
        assert_eq!(index_spec_0[0]["type"], "single");
        assert_eq!(index_spec_0[0]["value"], 1);

        assert_eq!(lbl0["properties"]["phantom"], "true");
        assert_eq!(lbl0["properties"]["stand"], "Short Boom");

        // Second label — empty properties
        let lbl1 = &labels[1];
        assert_eq!(lbl1["label"], "Kick Drum");
        assert_eq!(lbl1["port"]["port"], "Dante_In");

        let index_spec_1 = lbl1["port"]["indexSpec"].as_array().unwrap();
        assert_eq!(index_spec_1[0]["value"], 2);

        assert_eq!(
            lbl1["properties"],
            serde_json::Value::Object(serde_json::Map::new())
        );
    }

    // ── Test 6: Use declaration — all forms ───────────────────────────

    #[test]
    fn use_declaration_all_forms() {
        // Need a template to satisfy the parser (some forms are top-level only).
        let src = r#"
use yamaha { CL5, Rio3224 }
use buildings.foh { FOH_System }
use shure.*
use infrastructure.dante
template T { ports { X: out } }
"#;
        let json = get_json(src);
        let uses = all_stmts_of_type(&json, "Use");
        assert_eq!(uses.len(), 4, "expected 4 Use statements, got {}", uses.len());

        // use yamaha { CL5, Rio3224 }
        let u0 = uses[0];
        assert_eq!(u0["namespace"], "yamaha");
        let templates_0 = u0["templates"].as_array().unwrap();
        assert!(
            templates_0.iter().any(|t| t == "CL5"),
            "expected CL5 in templates"
        );
        assert!(
            templates_0.iter().any(|t| t == "Rio3224"),
            "expected Rio3224 in templates"
        );
        assert_eq!(u0["wildcard"], false);

        // use buildings.foh { FOH_System }
        let u1 = uses[1];
        assert_eq!(u1["namespace"], "buildings.foh");
        let templates_1 = u1["templates"].as_array().unwrap();
        assert!(
            templates_1.iter().any(|t| t == "FOH_System"),
            "expected FOH_System in templates"
        );
        assert_eq!(u1["wildcard"], false);

        // use shure.*
        let u2 = uses[2];
        assert_eq!(u2["namespace"], "shure");
        assert_eq!(u2["wildcard"], true);
        assert_eq!(
            u2["templates"].as_array().unwrap().len(),
            0,
            "wildcard use should have empty templates array"
        );

        // use infrastructure.dante  (no braces — namespace-only import)
        let u3 = uses[3];
        assert_eq!(u3["namespace"], "infrastructure.dante");
        assert_eq!(u3["wildcard"], false);
        assert_eq!(
            u3["templates"].as_array().unwrap().len(),
            0,
            "namespace-only use should have empty templates array"
        );
    }

    // ── Test 7: Ring with implicit (port-free) members ────────────────

    #[test]
    fn ring_declaration_implicit_members() {
        let src = r#"
template SD12 { ports { OptoCore_A: io [OptoCore] } }
instance Console is SD12
instance Rack1 is SD12
instance Rack2 is SD12
ring OptoCore_Primary {
  protocol: "OptoCore"
  member Console
  member Rack1
  member Rack2
}
"#;
        let json = get_json(src);
        let stmt = first_stmt_of_type(&json, "Ring");

        assert_eq!(stmt["type"], "Ring");
        assert_eq!(stmt["name"], "OptoCore_Primary");
        assert_eq!(stmt["properties"]["protocol"], "OptoCore");

        let members = stmt["members"].as_array().unwrap();
        assert_eq!(members.len(), 3);

        let instance_names: Vec<&str> = members
            .iter()
            .map(|m| m["instanceName"].as_str().unwrap())
            .collect();
        assert!(instance_names.contains(&"Console"));
        assert!(instance_names.contains(&"Rack1"));
        assert!(instance_names.contains(&"Rack2"));

        // Implicit members have no explicit portName
        for member in members {
            assert!(
                member.get("portName").is_none() || member["portName"].is_null(),
                "implicit ring member should have null portName, got: {}",
                member["portName"]
            );
        }
    }

    // ── Test 8: Ring with explicit port members ───────────────────────

    #[test]
    fn ring_declaration_explicit_port_members() {
        let src = r#"
template SD12 {
  ports { OptoCore_A: io [OptoCore]  OptoCore_B: io [OptoCore] }
}
instance Console is SD12
instance Rack is SD12
ring Redundant {
  protocol: "OptoCore"
  member Console.OptoCore_B
  member Rack.OptoCore_B
}
"#;
        let json = get_json(src);
        let stmt = first_stmt_of_type(&json, "Ring");

        let members = stmt["members"].as_array().unwrap();
        assert_eq!(members.len(), 2);

        for member in members {
            assert_eq!(
                member["portName"], "OptoCore_B",
                "explicit ring member should carry portName"
            );
        }

        let instance_names: Vec<&str> = members
            .iter()
            .map(|m| m["instanceName"].as_str().unwrap())
            .collect();
        assert!(instance_names.contains(&"Console"));
        assert!(instance_names.contains(&"Rack"));
    }

    // ── Test 9: Multi-file compile_project output shape ───────────────

    #[test]
    fn compile_project_output_shape() {
        let mut files: HashMap<String, String> = HashMap::new();
        files.insert(
            "campus.patch".into(),
            "use buildings.foh { FOH }\ninstance F is FOH".into(),
        );
        files.insert(
            "buildings/foh.patch".into(),
            "use yamaha { CL5 }\ntemplate FOH { ports { Out: out } }\ninstance Console is CL5"
                .into(),
        );
        files.insert(
            "yamaha.patch".into(),
            "template CL5 { ports { Dante: out } }".into(),
        );

        let result = compile_project(files, "campus.patch");
        assert!(result.errors.is_empty(), "unexpected errors: {:?}", result.errors);

        // All 3 files must appear in the file table
        assert_eq!(result.files.len(), 3);
        assert!(result.files.contains(&"campus.patch".to_string()));
        assert!(result.files.contains(&"buildings/foh.patch".to_string()));
        assert!(result.files.contains(&"yamaha.patch".to_string()));

        // Template file provenance
        assert_eq!(
            result.template_files.get("CL5").map(String::as_str),
            Some("yamaha.patch"),
            "CL5 should be from yamaha.patch"
        );
        assert_eq!(
            result.template_files.get("FOH").map(String::as_str),
            Some("buildings/foh.patch"),
            "FOH should be from buildings/foh.patch"
        );

        // Use graph
        let campus_deps = result.use_graph.get("campus.patch").unwrap();
        assert!(
            campus_deps.contains(&"buildings.foh".to_string()),
            "campus.patch should use buildings.foh"
        );

        let foh_deps = result.use_graph.get("buildings/foh.patch").unwrap();
        assert!(
            foh_deps.contains(&"yamaha".to_string()),
            "buildings/foh.patch should use yamaha"
        );

        let yamaha_deps = result.use_graph.get("yamaha.patch").unwrap();
        assert!(
            yamaha_deps.is_empty(),
            "yamaha.patch should have no use dependencies"
        );

        // Merged program — Use statements are dropped, templates remain
        let ts_json = serde_json::to_value(&result).unwrap();
        let statements = ts_json["program"]["statements"].as_array().unwrap();
        // Use stmts filtered out — only Template + Instance statements remain
        let use_count = statements.iter().filter(|s| s["type"] == "Use").count();
        assert_eq!(use_count, 0, "Use statements must be dropped from merged program");

        let template_count = statements.iter().filter(|s| s["type"] == "Template").count();
        assert!(
            template_count >= 2,
            "expected at least FOH and CL5 templates in merged program, got {template_count}"
        );

        // No errors
        assert!(result.errors.is_empty());
    }

    // ── Test 10: Multi-file with DRC error across files ───────────────

    #[test]
    fn compile_project_drc_error_across_files() {
        let mut files: HashMap<String, String> = HashMap::new();
        files.insert(
            "main.patch".into(),
            "use lib { Dev }\ninstance A is Dev\nconnect A.Out -> A.GhostPort".into(),
        );
        files.insert(
            "lib.patch".into(),
            "template Dev { ports { Out: out  In: in } }".into(),
        );

        let result = compile_project(files, "main.patch");
        assert!(result.errors.is_empty(), "unexpected parse errors: {:?}", result.errors);

        assert!(
            result.diagnostics.iter().any(|d| d.message.contains("GhostPort")),
            "expected a diagnostic about GhostPort, got: {:?}",
            result.diagnostics
        );
    }

    // ── Test 11: DRC diagnostics shape ────────────────────────────────

    #[test]
    fn drc_diagnostics_shape() {
        // Connecting an input to another input triggers D01
        let src = r#"
template Dev { ports { Out: out  In: in } }
instance A is Dev
instance B is Dev
connect A.In -> B.In
"#;
        let result = check(src);
        assert!(result.errors.is_empty(), "unexpected parse errors: {:?}", result.errors);

        assert!(
            !result.diagnostics.is_empty(),
            "expected at least one DRC diagnostic for input→input connection"
        );

        let diag = &result.diagnostics[0];
        let json = serde_json::to_value(diag).unwrap();

        // Must have severity, layer, message fields
        assert!(
            !json["severity"].is_null(),
            "diagnostic must have a 'severity' field"
        );
        assert!(
            !json["layer"].is_null(),
            "diagnostic must have a 'layer' field"
        );
        assert!(
            !json["message"].as_str().unwrap_or("").is_empty(),
            "diagnostic must have a non-empty 'message'"
        );

        // Direction layer should fire
        assert_eq!(
            json["layer"], "direction",
            "expected 'direction' layer for input→input connect"
        );
        assert_eq!(
            json["severity"], "error",
            "input→input connection must be an error"
        );
        assert!(
            json["message"].as_str().unwrap().contains("input"),
            "message should mention 'input', got: {}",
            json["message"]
        );
    }

    // ── Test 12: Empty program ─────────────────────────────────────────

    #[test]
    fn empty_program_no_errors() {
        let src = "# just a comment, nothing else\n";
        let result = check(src);

        assert!(
            result.errors.is_empty(),
            "comment-only source must produce no parse errors: {:?}",
            result.errors
        );

        let json = serde_json::to_value(&result).unwrap();
        let statements = json["program"]["statements"].as_array().unwrap();
        assert_eq!(
            statements.len(),
            0,
            "comment-only source must produce 0 statements"
        );
    }
}
