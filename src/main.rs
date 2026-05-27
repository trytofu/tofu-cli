#![warn(clippy::pedantic)]
use clap::Parser;

use crate::{
    api::ApiClient,
    commands::{
        Cli, Commands, ConfigCommands, MembersCommands, WorkspaceCommands, auth, cli_config,
        health, usage, workspaces,
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
    let api = ApiClient::new(api_base_url, config.resolve_token());

    match cli.command {
        Commands::Health => health::run(&api, cli.json).await,
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
        Commands::Whoami => auth::whoami(&api, cli.json).await,
        Commands::Usage => usage::run(&api, cli.json).await,
        Commands::Config { command } => match command {
            ConfigCommands::Show => cli_config::show(&config, cli.json),
        },
        Commands::Workspaces { command } => match command {
            WorkspaceCommands::List => workspaces::list(&api, cli.json).await,
            WorkspaceCommands::Use { slug } => workspaces::cli_use(&api, slug, cli.json).await,
            WorkspaceCommands::Create { slug, name } => {
                workspaces::create(&api, slug, name, cli.json).await;
            }
            WorkspaceCommands::Members { command } => match command {
                MembersCommands::List => workspaces::members_list(&api, cli.json).await,
                MembersCommands::Add { email } => workspaces::members_add(&api, email, cli.json).await,
            },
        },
    }
}
