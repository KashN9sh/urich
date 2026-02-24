//! CLI for urich-rs: add-aggregate and scaffolding.

use std::fs;
use std::path::Path;

use clap::{Parser, Subcommand};

fn snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_lowercase().next().unwrap());
        } else {
            out.push(c);
        }
    }
    out
}

#[derive(Parser)]
#[command(name = "urich")]
#[command(about = "Urich Rust CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add an aggregate to a bounded context (generates domain, application, infrastructure, module).
    AddAggregate {
        /// Context name (e.g. orders)
        context: String,
        /// Aggregate name in PascalCase (e.g. Order)
        aggregate: String,
    },
}

const DOMAIN_RS: &str = r#"//! Domain: aggregate and events.
use urich_rs::{AggregateRoot, DomainEvent};

#[derive(Clone, Debug)]
pub struct AGGREGATE_CREATED {
    pub AGGREGATE_LOWER_id: String,
}

impl DomainEvent for AGGREGATE_CREATED {}

pub struct AGGREGATE;

impl AggregateRoot for AGGREGATE {
    fn name() -> &'static str {
        "AGGREGATE_LOWER"
    }
}
"#;

const APPLICATION_RS: &str = r#"//! Application: commands, queries, handlers.
use serde_json::{json, Value};
use urich_core::CoreError;
use urich_rs::{Command, Query};

use crate::domain::{AGGREGATE, AGGREGATE_CREATED};
use crate::infrastructure::IAGGREGATERepository;

#[derive(Clone, Debug, serde::Deserialize, Command)]
pub struct CreateAGGREGATE {
    pub AGGREGATE_LOWER_id: String,
}

#[derive(Clone, Debug, serde::Deserialize, Query)]
pub struct GetAGGREGATE {
    pub AGGREGATE_LOWER_id: String,
}

pub fn create_AGGREGATE_LOWER(cmd: CreateAGGREGATE) -> Result<Value, CoreError> {
    Ok(json!({ "ok": true, "id": cmd.AGGREGATE_LOWER_id }))
}

pub fn get_AGGREGATE_LOWER(query: GetAGGREGATE) -> Result<Value, CoreError> {
    Ok(json!({ "id": query.AGGREGATE_LOWER_id }))
}
"#;

const INFRASTRUCTURE_RS: &str = r#"//! Infrastructure: repository implementation.
use urich_core::CoreError;
use urich_rs::Repository;

use crate::domain::AGGREGATE;

pub trait IAGGREGATERepository: Send {
    fn get(&self, id: &str) -> Result<Option<AGGREGATE>, CoreError>;
    fn add(&mut self, aggregate: AGGREGATE) -> Result<(), CoreError>;
    fn save(&mut self, _aggregate: &AGGREGATE) -> Result<(), CoreError> {
        Ok(())
    }
}

pub struct AGGREGATERepositoryImpl {
    _store: std::collections::HashMap<String, ()>,
}

impl AGGREGATERepositoryImpl {
    pub fn new() -> Self {
        Self {
            _store: std::collections::HashMap::new(),
        }
    }
}

impl IAGGREGATERepository for AGGREGATERepositoryImpl {
    fn get(&self, _id: &str) -> Result<Option<AGGREGATE>, CoreError> {
        Ok(None)
    }
    fn add(&mut self, _aggregate: AGGREGATE) -> Result<(), CoreError> {
        Ok(())
    }
}
"#;

const MODULE_RS: &str = r#"//! Bounded context «CONTEXT»: module definition.
use urich_rs::DomainModule;

use crate::application::{create_AGGREGATE_LOWER, get_AGGREGATE_LOWER, CreateAGGREGATE, GetAGGREGATE};

pub fn module() -> DomainModule {
    DomainModule::new("CONTEXT")
        .command_type::<CreateAGGREGATE>(create_AGGREGATE_LOWER)
        .query_type::<GetAGGREGATE>(get_AGGREGATE_LOWER)
}
"#;

const MAIN_RS: &str = r#"//! Entry point: app is composed from modules.
use urich_rs::Application;

// mod CONTEXT;
// use CONTEXT::module as CONTEXT_module;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = Application::new();
    // app.register(&mut CONTEXT_module())?;
    println!("Listening on http://127.0.0.1:8000");
    app.run("127.0.0.1", 8000, "API", "0.1.0")
}
"#;

fn replace_template(template: &str, context: &str, aggregate: &str, aggregate_lower: &str) -> String {
    template
        .replace("CONTEXT", context)
        .replace("AGGREGATE", aggregate)
        .replace("AGGREGATE_LOWER", aggregate_lower)
        .replace("IAGGREGATERepository", &format!("I{}Repository", aggregate))
        .replace("AGGREGATERepositoryImpl", &format!("{}RepositoryImpl", aggregate))
}

fn run_add_aggregate(context: &str, aggregate: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let aggregate_lower = snake_case(aggregate);
    let dir = Path::new(context);
    fs::create_dir_all(dir)?;

    let domain_rs = replace_template(DOMAIN_RS, context, aggregate, &aggregate_lower);
    let application_rs = replace_template(APPLICATION_RS, context, aggregate, &aggregate_lower);
    let infrastructure_rs = replace_template(INFRASTRUCTURE_RS, context, aggregate, &aggregate_lower);
    let module_rs = replace_template(MODULE_RS, context, aggregate, &aggregate_lower);

    fs::write(dir.join("domain.rs"), domain_rs)?;
    fs::write(dir.join("application.rs"), application_rs)?;
    fs::write(dir.join("infrastructure.rs"), infrastructure_rs)?;
    fs::write(dir.join("module.rs"), module_rs)?;

    let main_rs = MAIN_RS.replace("CONTEXT", context);
    if !Path::new("main.rs").exists() {
        fs::write("main.rs", main_rs)?;
    }

    println!(
        "Generated {}: domain.rs, application.rs, infrastructure.rs, module.rs",
        context
    );
    println!("Add to your main/Cargo: mod {}; use {}::module; app.register(&mut module())?;", context, context);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::AddAggregate { context, aggregate } => run_add_aggregate(&context, &aggregate),
    }
}
