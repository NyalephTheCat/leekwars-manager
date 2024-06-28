use reqwest::Client;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::future::Future;
use std::pin::Pin;

use crate::api::{AiInfo, FarmerAi, FolderInfo};

pub enum FolderOrAi {
    Folder(FolderInfo),
    Ai(AiInfo),
}

pub fn get_path_for(folder_or_ai: &FolderOrAi, ais: &FarmerAi) -> String {
    match folder_or_ai {
        FolderOrAi::Folder(folder) => {
            if folder.folder <= 0 {
                return folder.name.clone();
            }
            let parent = ais.folders.iter().find(|f| f.id == folder.folder);
            match parent {
                Some(f) => if f.folder <= 0 { format!("/{}", f.name.clone()) } else { 
                    format!("{}/{}", get_path_for(&FolderOrAi::Folder(f.clone()), ais), folder.name.clone())
                },
                None => folder.name.clone(),
            }
        }
        FolderOrAi::Ai(ai) => {
            if ai.folder <= 0 {
                return ai.name.clone();
            }
            let folder = ais.folders.iter().find(|f| f.id == ai.folder).unwrap();
            format!("{}/{}", get_path_for(&FolderOrAi::Folder(folder.clone()), ais), ai.name)
        }
    }
}

pub fn download_folder<'a>(client: &'a Client, ais: &'a FarmerAi, folder: &'a FolderInfo, token: &'a str, path: &'a str) -> Pin<Box<dyn Future<Output = Result<(), reqwest::Error>> + 'a>> {
    Box::pin(async move {
        create_dir_all(&path).unwrap();

        for ai in ais.ais.iter().filter(|ai| ai.folder == folder.id) {
            let ai_path = format!("{}/{}", path, ai.name);
            download_ai_to_path(client, ai.id, &ai_path, token).await?;
        }

        for sub_folder in ais.folders.iter().filter(|f| f.folder == folder.id) {
            download_folder(client, ais, sub_folder, token, path).await?;
        }

        Ok(())
    })
}

pub fn download_ai_to_path<'a>(client: &'a Client, ai_id: i32, path: &'a str, token: &'a str) -> Pin<Box<dyn Future<Output = Result<(), reqwest::Error>> + 'a>> {
    Box::pin(async move {
        let res = client
            .get(&format!("https://leekwars.com/api/ai/get/{}", ai_id))
            .bearer_auth(token)
            .send().await?;

        let ai_response = res.json::<crate::api::AiResponse>().await?;
        let ai_details = ai_response.ai;

        let file_path = Path::new(path);
        let mut file = File::create(&file_path).unwrap();

        println!("Downloading AI {} to {}", ai_details.name, file_path.display());

        file.write_all(ai_details.code.as_bytes()).unwrap();

        Ok(())
    })
}
