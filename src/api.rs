use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::pin::Pin;
use std::future::Future;
use std::io::Read;
use std::fs;

#[derive(Deserialize)]
pub struct FarmerAi {
    pub ais: Vec<AiInfo>,
    pub folders: Vec<FolderInfo>,
    pub leek_ais: HashMap<i32, i32>,
    pub bin: Vec<AiInfo>,
}

#[derive(Deserialize, Clone)]
pub struct AiInfo {
    pub id: i32,
    pub name: String,
    pub valid: Option<bool>,
    pub folder: i32,
    pub version: Option<i32>,
    pub strict: Option<bool>,
    pub includes_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, Clone)]
pub struct FolderInfo {
    pub id: i32,
    pub name: String,
    pub folder: i32,
}

#[derive(Deserialize)]
pub struct Session {
    pub token: String,
}

#[derive(Serialize)]
pub struct Login<'a> {
    pub login: &'a str,
    pub password: &'a str,
}

#[derive(Deserialize)]
pub struct AiResponse {
    pub ai: AiDetails,
}

#[derive(Deserialize, Serialize)]
pub struct AiDetails {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub folder: Option<i32>,
    pub level: Option<i32>,
}

#[derive(Serialize)]
pub struct NewFolder<'a> {
    pub name: &'a str,
    pub folder_id: i32,
}

#[derive(Serialize)]
pub struct NewAi<'a> {
    pub name: &'a str,
    pub folder_id: i32,
    pub version: i32,
}

#[derive(Deserialize)]
pub struct NewAiResponse {
    pub ai: AiInfo,
}

#[derive(Serialize)]
pub struct AiSave<'a> {
    pub ai_id: i32,
    pub code: &'a str,
}

#[derive(Deserialize)]
pub struct IdResponse {
    pub id: i32,
}

pub async fn login(client: &Client, login: &str, password: &str) -> Result<Session, reqwest::Error> {
    let res = client
        .post("https://leekwars.com/api/farmer/login-token")
        .json(&Login { login, password })
        .send().await?;

    let session = res.json::<Session>().await?;
    Ok(session)
}

pub async fn list_ais(client: &Client, session: &mut Session) -> Result<FarmerAi, reqwest::Error> {
    let res = client
        .get("https://leekwars.com/api/ai/get-farmer-ais")
        .bearer_auth(&session.token)
        .send().await?;

    let ais = res.json::<FarmerAi>().await?;
    Ok(ais)
}

pub fn upload_path<'a>(client: &'a Client, session: &'a Session, farmer: &'a FarmerAi, path: &'a Path, folder: i32) -> Pin<Box<dyn Future<Output = Result<(), reqwest::Error>> + 'a>> {
    Box::pin(async move {
        if path.is_dir() {
            let folder = create_folder(client, session, farmer, path, folder).await?;

            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                upload_path(client, session, farmer, &path, folder).await?;
            }
        } else {
            create_ai(client, session, farmer, path, folder).await?;
        }

        Ok(())
    })
}

pub fn create_folder<'a>(client: &'a Client, session: &'a Session, farmer: &'a FarmerAi, path: &'a Path, folder: i32) -> Pin<Box<dyn Future<Output = Result<i32, reqwest::Error>> + 'a>> {
    Box::pin(async move {
        println!("Checking folder {:?}...", path.file_name());

        if folder == -1 {
            return Ok(0);
        }

        let folder_name = path.file_name().unwrap().to_str().unwrap();
        
        if let Some(folder) = farmer.folders.iter().find(|f| f.name == folder_name && f.folder == folder) {
            return Ok(folder.id);
        }

        println!("Creating folder {}...", folder_name);

        let res = client
            .post("https://leekwars.com/api/ai-folder/new-name")
            .bearer_auth(&session.token)
            .json(&NewFolder { name: folder_name, folder_id: folder })
            .send().await?;

        let folder = res.json::<IdResponse>().await?;

        Ok(folder.id)
    })
}

pub fn create_ai<'a>(client: &'a Client, session: &'a Session, farmer: &'a FarmerAi, path: &'a Path, folder: i32) -> Pin<Box<dyn Future<Output = Result<(), reqwest::Error>> + 'a>> {
    Box::pin(async move {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let mut file = File::open(path).unwrap();
   
        let mut ai = farmer.ais.iter().find(|ai| ai.name == file_name && ai.folder == folder).cloned();

        if ai.is_none() {
            println!("Creating AI {}...", file_name);

            let res = client
                .post("https://leekwars.com/api/ai/new-name")
                .bearer_auth(&session.token)
                .json(&NewAi { name: file_name, folder_id: folder, version: 4 })
                .send().await?;

            let new_ai = res.json::<NewAiResponse>().await?.ai;
            ai = Some(new_ai);
        }

        println!("Uploading AI {}...", file_name);

        let ai = ai.unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();

        let res = client
            .post("https://leekwars.com/api/ai/save/")
            .bearer_auth(&session.token)
            .json(&AiSave { ai_id: ai.id, code: &code })
            .send().await?;

        Ok(())
    })
}

