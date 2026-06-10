#![warn(clippy::pedantic)]
use clap::Parser;

use crate::{
    api::ApiClient,
    commands::{
        Cli, Commands, ConfigCommands, EventsCommands, HooksCommands, MembersCommands,
        TargetsCommand, WorkspaceCommands, auth, cli_config, events, health, hooks, replay,
        targets::{self, TargetStatus},
        usage, watch, workspaces,
    },
    config::Config,
};

mod api;
mod commands;
mod config;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut config = Config::load();
    let api_base_url = config.resolve_api_base_url(cli.api_base_url.clone());
    let client = ApiClient::new(api_base_url, config.resolve_token());

    match cli.command {
        Commands::Version => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "version": env!("CARGO_PKG_VERSION"),
                    })
                );
            } else {
                println!("{}", env!("CARGO_PKG_VERSION"));
            }
        }
        Commands::Health => health::run(&client, cli.json).await,
        Commands::Login {
            token,
            api_base_url: login_base_url,
            no_browser,
        } => {
            auth::login(
                &mut config,
                token,
                login_base_url.or(cli.api_base_url),
                no_browser,
                cli.json,
            )
            .await;
        }
        Commands::Logout => auth::logout(&mut config, cli.json),
        Commands::Whoami => auth::whoami(&client, cli.json).await,
        Commands::Usage => usage::run(&client, cli.json).await,
        Commands::Config { command } => match command {
            ConfigCommands::Show => cli_config::show(&config, cli.json),
        },
        Commands::Workspaces { command } => match command {
            WorkspaceCommands::List => workspaces::list(&client, cli.json).await,
            WorkspaceCommands::Use { slug } => workspaces::cli_use(&client, slug, cli.json).await,
            WorkspaceCommands::Create { slug, name } => {
                workspaces::create(&client, slug, name, cli.json).await;
            }
            WorkspaceCommands::Members { command } => match command {
                MembersCommands::List => workspaces::members_list(&client, cli.json).await,
                MembersCommands::Add { email } => {
                    workspaces::members_add(&client, email, cli.json).await;
                }
            },
        },
        Commands::Hooks { command } => match command {
            HooksCommands::List => hooks::list(&client, cli.json).await,
            HooksCommands::Create { slug, name } => {
                hooks::create_hook(&client, slug, name, cli.json).await;
            }
            HooksCommands::Url { slug } => hooks::url(&client, slug, cli.json).await,
            HooksCommands::Status { slug } => hooks::status(&client, slug, cli.json).await,
        },
        Commands::Targets { command } => match command {
            TargetsCommand::Add { name, url, hook } => {
                targets::add(&client, name, url, hook, cli.json).await;
            }
            TargetsCommand::Set { name, url, hook } => {
                targets::set(&client, name, url, hook, cli.json).await;
            }
            TargetsCommand::List { hook } => targets::list(&client, hook, cli.json).await,
            TargetsCommand::Enable { name, hook } => {
                targets::toggle(TargetStatus::On, &client, name, hook, cli.json).await;
            }
            TargetsCommand::Disable { name, hook } => {
                targets::toggle(TargetStatus::Off, &client, name, hook, cli.json).await;
            }
            TargetsCommand::Delete { name, hook } => {
                targets::delete(&client, name, hook, cli.json).await;
            }
        },
        Commands::Events { command } => match command {
            EventsCommands::List { hook, limit } => {
                events::list(&client, hook, limit, cli.json).await;
            }
            EventsCommands::Show { event_id } => events::show(&client, event_id, cli.json).await,
            EventsCommands::Latest { hook } => events::latest(&client, hook, cli.json).await,
            EventsCommands::Expire { event_id } => {
                events::expire(&client, event_id, cli.json).await;
            }
        },
        Commands::Replay {
            event,
            hook,
            target,
        } => replay::run(&client, event, hook, target, cli.json).await,
        Commands::Watch {
            slug,
            deliveries,
            target,
        } => watch::run(&client, slug, deliveries, target, cli.json).await,
    }
}
