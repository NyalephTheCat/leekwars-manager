use clap::{Parser, Subcommand};
use reqwest::Client;

mod api;
mod cli;
mod utils;

use crate::api::{list_ais, login, upload_path};
use crate::cli::{Cli, Command};
use crate::utils::get_path_for;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let client = Client::new();
    let mut session = login(&client, &args.login, &args.password).await.unwrap();

    match args.cmd {
        Command::List => {
            let ais = list_ais(&client, &mut session).await.unwrap();

            for ai in &ais.ais {
                println!("{}: {}", ai.id, get_path_for(&utils::FolderOrAi::Ai(ai.clone()), &ais));
            }
        }
        Command::Download { output } => {
            let output = output.unwrap_or_else(|| ".".to_string());

            std::fs::create_dir_all(&output).unwrap();

            println!("Downloading AIs to {}", output);
            let ais = list_ais(&client, &mut session).await.unwrap();

            for ai in &ais.ais {
                if ai.folder != 0 {
                    continue;
                }
                let path = format!("{}/{}", output, get_path_for(&utils::FolderOrAi::Ai(ai.clone()), &ais));
                utils::download_ai_to_path(&client, ai.id, &path, &session.token).await.unwrap();
            }

            for folder in &ais.folders {
                if folder.folder == 0 {
                    let path = format!("{}/{}", output, get_path_for(&utils::FolderOrAi::Folder(folder.clone()), &ais));
                    utils::download_folder(&client, &ais, &folder, &session.token, &path).await.unwrap();
                }
            }
        }
        Command::Upload { input } => {
            println!("Uploading AIs from {}", input);

            let path = std::path::Path::new(&input);

            let ais = list_ais(&client, &mut session).await.unwrap();

            upload_path(&client, &session, &ais, path, -1).await.unwrap();
        }
    }
}
