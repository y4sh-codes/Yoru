//! CLI entrypoint and command dispatcher.
//!
//! Doctag:cli-entry

use clap::Parser;

use crate::app::state::AppState;
use crate::cli::args::{Cli, Command, SendArgs};
use crate::core::models::{
    AuthStrategy, EnvVar, Environment, HttpMethod, KeyValue, RequestBody, RequestTemplate,
};
use crate::http::client::build_http_client;
use crate::http::executor::HttpExecutor;
use crate::storage::fs_store::{FsWorkspaceStore, WorkspaceStore};
use crate::tui::run_tui;
use crate::util::logging::init_logging;
use crate::{YoruError, YoruResult};

/// Parses args and executes selected command.
pub async fn run() -> color_eyre::Result<()> {
    let cli = Cli::parse();
    init_logging().ok();

    let store = FsWorkspaceStore::new(cli.data_dir.clone())?;

    match cli.command.unwrap_or(Command::Tui) {
        Command::Tui => {
            let workspace = store.load_workspace()?;
            let app_state = AppState::new(workspace);
            let executor = HttpExecutor::new(build_http_client()?);
            run_tui(app_state, executor, &store).await?;
        }
        Command::Init { name } => {
            let mut workspace = crate::core::models::Workspace::sample();
            if let Some(name) = name {
                workspace.name = name;
            }
            store.save_workspace(&workspace)?;
            println!(
                "Initialized workspace at {}",
                store.workspace_file().display()
            );
        }
        Command::Import { file } => {
            let workspace = store.import_workspace(&file)?;
            store.save_workspace(&workspace)?;
            println!("Imported workspace from {}", file.display());
        }
        Command::Export { file } => {
            let workspace = store.load_workspace()?;
            store.export_workspace(&workspace, &file)?;
            println!("Exported workspace to {}", file.display());
        }
        Command::Send(args) => {
            execute_send_command(args).await?;
        }
    }

    Ok(())
}

async fn execute_send_command(args: SendArgs) -> YoruResult<()> {
    let method = args
        .method
        .parse::<HttpMethod>()
        .map_err(YoruError::Validation)?;

    let mut request = RequestTemplate::new(
        args.name.unwrap_or_else(|| "CLI Request".to_string()),
        method,
        args.url,
    );

    request.headers = args
        .headers
        .iter()
        .map(|header| parse_header(header))
        .collect::<YoruResult<Vec<_>>>()?;
    request.query = args
        .query
        .iter()
        .map(|item| parse_key_value(item))
        .collect::<YoruResult<Vec<_>>>()?;
    request.timeout_ms = args.timeout_ms;
    request.pre_request_script = args.pre_script;
    request.test_script = args.test_script;

    if let Some(json_payload) = args.json {
        let value = serde_json::from_str::<serde_json::Value>(&json_payload)
            .map_err(|err| YoruError::Validation(format!("invalid --json payload: {err}")))?;
        request.body = RequestBody::Json { value };
    } else if let Some(data) = args.data {
        request.body = RequestBody::Raw {
            mime_type: "text/plain".to_string(),
            content: data,
        };
    }

    request.auth = if let Some(token) = args.bearer {
        AuthStrategy::Bearer { token }
    } else if let (Some(username), Some(password)) = (args.basic_user, args.basic_password) {
        AuthStrategy::Basic { username, password }
    } else if let Some(api_key) = args.api_key {
        let parsed = parse_key_value(&api_key)?;
        AuthStrategy::ApiKey {
            key: parsed.key,
            value: parsed.value,
            in_header: !args.api_key_in_query,
        }
    } else {
        AuthStrategy::None
    };

    let environment = if args.env.is_empty() {
        None
    } else {
        Some(Environment {
            id: uuid::Uuid::new_v4(),
            name: "cli".to_string(),
            variables: args
                .env
                .iter()
                .map(|item| {
                    parse_key_value(item).map(|pair| EnvVar {
                        key: pair.key,
                        value: pair.value,
                        secret: false,
                    })
                })
                .collect::<YoruResult<Vec<_>>>()?,
        })
    };

    let executor = HttpExecutor::new(build_http_client()?);
    let response = executor
        .execute_request(&request, environment.as_ref())
        .await?;

    println!("Status: {} {}", response.status, response.status_text);
    println!("Duration: {} ms", response.duration_ms);
    println!("Size: {} bytes", response.size_bytes);
    println!();
    println!("Headers:");
    for (name, value) in response.headers {
        println!("  {name}: {value}");
    }
    println!();
    println!("Body:");
    println!("{}", response.body);

    if !response.script_logs.is_empty() {
        println!();
        println!("Script Logs:");
        for line in response.script_logs {
            println!("  - {line}");
        }
    }

    Ok(())
}

fn parse_header(input: &str) -> YoruResult<KeyValue> {
    let Some((key, value)) = input.split_once(':') else {
        return Err(YoruError::Validation(format!(
            "invalid header '{input}', expected Key:Value"
        )));
    };

    Ok(KeyValue {
        key: key.trim().to_string(),
        value: value.trim().to_string(),
        enabled: true,
    })
}

fn parse_key_value(input: &str) -> YoruResult<KeyValue> {
    let Some((key, value)) = input.split_once('=') else {
        return Err(YoruError::Validation(format!(
            "invalid key-value '{input}', expected key=value"
        )));
    };

    Ok(KeyValue {
        key: key.trim().to_string(),
        value: value.trim().to_string(),
        enabled: true,
    })
}
