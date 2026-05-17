use clap::Parser;

use crate::{
    api::ApiClient,
    commands::{Cli, Commands, auth, health},
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
            .await
        },
        Commands::Logout => auth::logout(&mut config, cli.json),
    }
}
