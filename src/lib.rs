use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::PathBuf};

const ALLOWED_GAMES: [&str; 2] = ["EM1", "EM2"];
const ALLOWED_PLATFORMS: [&str; 2] = ["wii", "pc"];
const BANNED_EXTENSIONS: [&str; 6] = ["dll", "so", "exe", "sh", "bat", "scr"];

pub fn validate(path: &PathBuf) -> Result<ModInfo, Box<dyn std::error::Error>> {
    let mut mod_info_path = path.clone();
    mod_info_path.push("mod.json");

    let mut mod_description_path = path.clone();
    mod_description_path.push("description.md");

    if !mod_info_path.exists() {
        return Err("mod.json does not exist.".into());
    }
    let mut mod_info_file = File::open(mod_info_path)?;
    let mut mod_info_buffer = String::new();
    mod_info_file.read_to_string(&mut mod_info_buffer)?;
    let mut mod_info: ModInfo = serde_json::from_str(&mod_info_buffer)?;

    if mod_info.name.trim().is_empty() {
        return Err("mod name is empty.".into());
    }

    if mod_info.description.trim().is_empty() {
        if mod_description_path.exists() {
            let mut mod_description_file = File::open(mod_description_path)?;
            let mut mod_description = String::new();

            mod_description_file.read_to_string(&mut mod_description)?;

            if mod_description.trim().is_empty() {
                return Err("mod description is empty.".into());
            }

            mod_info.description = mod_description;
        }
    }

    if !ALLOWED_GAMES.contains(&mod_info.game.as_str()) {
        return Err("could not recognize defined game.".into());
    }

    if !ALLOWED_PLATFORMS.contains(&mod_info.platform.as_str()) {
        return Err("could not recognize defined platform.".into());
    }

    if mod_info.custom_game_files_path.trim().is_empty() {
        return Err("custom game files path is empty.".into());
    }

    if mod_info.custom_textures_path.trim().is_empty() {
        return Err("custom textures path is empty.".into());
    }

    if PathBuf::from(&mod_info.custom_textures_path).is_absolute()
        || PathBuf::from(&mod_info.custom_game_files_path).is_absolute()
    {
        return Err("you are not allowed to have absolute paths on custom file path.".into());
    }

    if PathBuf::from(&mod_info.custom_textures_path).exists() {
        return Err("custom textures path does not exist.".into());
    }

    if PathBuf::from(&mod_info.custom_game_files_path).exists() {
        return Err("custom game files path does not exist.".into());
    }

    if mod_info.icon_path.trim().is_empty() {
        return Err("mod icon path is empty.".into());
    }

    if PathBuf::from(&mod_info.icon_path).is_absolute() {
        return Err("you are not allowed to have absolute paths on mod icon.".into());
    }

    if PathBuf::from(&mod_info.icon_path).exists() {
        return Err("mod icon does not exist.".into());
    }

    for entry in walkdir::WalkDir::new(path).into_iter() {
        let res = entry?;
        if !res.path().is_file() {
            continue;
        }

        let formatted_extension = res
            .path()
            .extension()
            .unwrap()
            .to_ascii_lowercase()
            .to_str()
            .unwrap()
            .to_string();
        if BANNED_EXTENSIONS.contains(&formatted_extension.as_str()) {
            return Err(format!("mod contains illegal file ({})", formatted_extension).into());
        }
    }

    Ok(mod_info)
}

#[derive(Serialize, Deserialize)]
pub struct ModInfo {
    name: String,
    game: String,
    platform: String,
    description: String,
    dependencies: Vec<String>,
    custom_textures_path: String,
    custom_game_files_path: String,
    icon_path: String,
}
