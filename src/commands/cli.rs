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
    /// Log out and clear token
    Logout,
    /// Show the current authenticated user
    Whoami,
    /// Show the usage for the current user
    Usage,
    /// Manage config
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Manage workspaces
    Workspaces {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current config
    Show,
}

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    /// List workspaces
    List,
    /// Set active workspace
    Use { slug: String },
    /// Create a workspace
    Create {
        #[arg(
            value_name = "NAME_OR_SLUG",
            help = "workspace name or slug. Name-like values will be slugified automatically"
        )]
        slug: String,
        #[arg(long, help = "workspace display name")]
        name: Option<String>,
    },
    /// Manage workspace members
    Members {
        #[command(subcommand)]
        command: MembersCommands,
    },
}

#[derive(Subcommand)]
pub enum MembersCommands {
    /// List members
    List,
    /// Add a member by email
    Add { email: String }
}
