use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "tofu",
    about = "Forward webhooks to local services and replay them when needed.",
    long_about = "Tofu is a lightweight webhook relay for local development. It lets you receive webhook events through a shared public hook, forward them to your local services, inspect incoming requests, and replay previous events."
)]
pub struct Cli {
    #[arg(long, global = true, help = "Output JSON")]
    pub json: bool,

    #[arg(long, global = true, help = "API base URL override.")]
    pub api_base_url: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Checks API health
    Health,
    /// Log in with your browser approval or an API token
    Login {
        #[arg(long, help = "Personal API token")]
        token: Option<String>,
        #[arg(long, help = "API base URL to save")]
        api_base_url: Option<String>,
        #[arg(long, help = "Print the device login URL without opening a browser")]
        no_browser: bool,
    },
}
