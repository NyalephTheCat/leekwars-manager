use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "LeekWars Manager")]
#[command(about = "Manage your LeekWars AIs", long_about = None)]
pub struct Cli {
    pub login: String,
    pub password: String,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(name = "list")]
    #[command(about = "List your AIs")]
    List,

    #[command(name = "download")]
    #[command(about = "Download AI scripts")]
    Download {
        #[arg(short, long)]
        output: Option<String>,
    },

    #[command(name = "upload")]
    #[command(about = "Upload AI scripts")]
    Upload {
        #[arg(short, long)]
        input: String,
    },
}

