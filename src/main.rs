use clap::Parser;

use crate::{
    api::api_client::ApiClient,
    commands::commands::{Cli, Commands},
    config::config::Config,
};

mod api;
mod commands;
mod config;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = Config::load();
    let api_base_url = config.resolve_api_base_url(cli.api_base_url.clone());
    let api = ApiClient::new(api_base_url, config.resolve_token());

    match cli.command {
        Commands::Health => commands::health::health(&api, cli.json).await,
    }
}
