use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "Rust Ironclad")]
#[command(about = "Laravel-style CLI for Rust Framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new controller
    Make {
        #[command(subcommand)]
        resource: MakeResource,
    },
}

#[derive(Subcommand)]
enum MakeResource {
    /// Create a new controller
    Controller {
        /// Name of the controller
        name: String,
    },
    /// Create a new migration
    Migration {
        /// Name of the migration
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Make { resource } => match resource {
            MakeResource::Controller { name } => {
                make_controller(&name);
            }
            MakeResource::Migration { name } => {
                make_migration(&name);
            }
        },
    }
}

fn make_controller(name: &str) {
    let snake_name = to_snake_case(name);
    let controller_path = format!("src/infrastructure/http/{}.rs", snake_name);

    if Path::new(&controller_path).exists() {
        eprintln!("Controller {} already exists!", controller_path);
        return;
    }

    let template = format!(
        r#"use std::sync::Arc;
use actix_web::{{web, HttpResponse}};
use crate::application::dtos::*;
use crate::errors::ApiResult;
use crate::infrastructure::http::AuthUser;

pub struct {}Controller;

impl {}Controller {{
    // Implement methods here
}}
"#,
        to_pascal_case(name),
        to_pascal_case(name)
    );

    fs::write(&controller_path, template).expect("Failed to create controller");
    println!("✅ Controller created: {}", controller_path);
}

fn make_migration(name: &str) {
    use chrono::Local;

    let timestamp = Local::now().format("%Y%m%d%H%M%S");
    let migration_filename = format!("migrations/{}_{}.sql", timestamp, to_snake_case(name));

    let template = format!(
        "-- Migration: {}\n-- Created at: {}\n\n-- Write your SQL here:\n",
        name, timestamp
    );

    fs::write(&migration_filename, template).expect("Failed to create migration");
    println!("✅ Migration created: {}", migration_filename);
}

fn to_snake_case(s: &str) -> String {
    s.chars()
        .fold(String::new(), |mut acc, c| {
            if c.is_uppercase() {
                acc.push('_');
                acc.push(c.to_lowercase().next().unwrap());
            } else {
                acc.push(c);
            }
            acc
        })
        .trim_start_matches('_')
        .to_string()
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;

    for c in s.chars() {
        if c == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_uppercase().next().unwrap());
            capitalize = false;
        } else {
            result.push(c);
        }
    }

    result
}
