//! Rhai-powered script hooks for pre-request and tests.
//!
//! Doctag:scripting

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rhai::{Dynamic, Engine, EvalAltResult, Map, Scope};

/// Script execution report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptReport {
    pub logs: Vec<String>,
    pub error: Option<String>,
}

fn map_to_rhai(map: &HashMap<String, String>) -> Map {
    map.iter()
        .map(|(key, value)| (key.clone().into(), value.clone().into()))
        .collect()
}

fn execute(
    script: &str,
    vars: &HashMap<String, String>,
) -> Result<ScriptReport, Box<EvalAltResult>> {
    let logs = Arc::new(Mutex::new(Vec::<String>::new()));

    let mut engine = Engine::new();
    let logs_capture = Arc::clone(&logs);
    engine.register_fn("log", move |message: &str| {
        if let Ok(mut guard) = logs_capture.lock() {
            guard.push(message.to_string());
        }
    });

    let mut scope = Scope::new();
    scope.push_dynamic("vars", Dynamic::from_map(map_to_rhai(vars)));

    let _ = engine.eval_with_scope::<Dynamic>(&mut scope, script)?;

    let report = ScriptReport {
        logs: logs.lock().map(|guard| guard.clone()).unwrap_or_default(),
        error: None,
    };

    Ok(report)
}

/// Executes a pre-request script if present.
pub fn run_pre_request_script(
    script: Option<&str>,
    vars: &HashMap<String, String>,
) -> ScriptReport {
    match script {
        None => ScriptReport::default(),
        Some(source) => execute(source, vars).unwrap_or_else(|err| ScriptReport {
            logs: Vec::new(),
            error: Some(err.to_string()),
        }),
    }
}

/// Executes a test script if present.
pub fn run_test_script(script: Option<&str>, vars: &HashMap<String, String>) -> ScriptReport {
    match script {
        None => ScriptReport::default(),
        Some(source) => execute(source, vars).unwrap_or_else(|err| ScriptReport {
            logs: Vec::new(),
            error: Some(err.to_string()),
        }),
    }
}
