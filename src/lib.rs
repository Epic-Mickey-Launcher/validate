use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const ALLOWED_GAMES: [&str; 3] = ["EM1", "EM2", "EMR"];
const ALLOWED_PLATFORMS: [&str; 2] = ["wii", "pc"];
const BANNED_EXTENSIONS: [&str; 6] = ["dll", "so", "exe", "sh", "bat", "scr"]; // not technically
                                                                               // banned, but will
                                                                               // require analysis
                                                                               // by a moderator

pub fn validate(path: &PathBuf) -> Result<ModInfo, Box<dyn std::error::Error>> {
    let mut final_mod_info: ModInfo = ModInfo {
        name: "".to_string(),
        game: "".to_string(),
        platform: "".to_string(),
        description: "".to_string(),
        shortdescription: "".to_string(),
        dependencies: Vec::new(),
        custom_textures_path: "".to_string(),
        custom_game_files_path: "".to_string(),
        scripts_path: "".to_string(),
        icon_path: "".to_string(),
        auto_generated_tags: Vec::new(),
    };
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
    let mod_info: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&mod_info_buffer)?;

    let name = mod_info.get("name").unwrap().as_str().unwrap();
    println!("{}", name);
    if name.trim().is_empty() {
        return Err("mod name is empty.".into());
    }

    final_mod_info.name = name.to_string();

    let short_description_value = mod_info.get("shortdescription");
    let mut no_short_description = false;

    match short_description_value {
        Some(x) => {
            let short_description = x.as_str().unwrap().trim().to_string();
            final_mod_info.shortdescription = short_description;
        }
        None => no_short_description = true,
    }

    if mod_description_path.exists() {
        let mut mod_description_file = File::open(mod_description_path)?;
        let mut mod_description = String::new();

        mod_description_file.read_to_string(&mut mod_description)?;

        if mod_description.trim().is_empty() {
            return Err("mod description is empty.".into());
        }

        final_mod_info.description = mod_description.trim().to_string();

        if no_short_description {
            final_mod_info.shortdescription = "clone".to_string();
        }
    }

    let game = &mod_info
        .get("game")
        .unwrap()
        .as_str()
        .unwrap()
        .to_uppercase();
    let platform = &mod_info
        .get("platform")
        .unwrap()
        .as_str()
        .unwrap()
        .to_uppercase();

    final_mod_info.game = game.to_string();
    final_mod_info.platform = platform.to_string();

    println!("{}", game);

    if !ALLOWED_GAMES.contains(&game.to_uppercase().as_str()) {
        return Err("could not recognize defined game.".into());
    }

    if !ALLOWED_PLATFORMS.contains(&platform.to_uppercase().as_str()) {
        return Err("could not recognize defined platform.".into());
    }

    if game.to_string() == "EMR" && platform.to_string() == "WII" {
        return Err("impossible combination (emr/wii)".into());
    }

    if game.to_string() == "EM1" && platform.to_string() == "PC" {
        return Err("impossible combination (em1/pc)".into());
    }

    let mut no_custom_textures = false;
    let mut no_custom_files = false;
    let mut no_scripts = false;

    let custom_textures_path = match mod_info.get("custom_textures_path") {
        Some(x) => x.as_str().unwrap().to_string(),
        None => {
            no_custom_textures = true;
            "".to_string()
        }
    };

    let custom_game_files_path = match mod_info.get("custom_game_files_path") {
        Some(x) => x.as_str().unwrap().to_string(),
        None => {
            no_custom_files = true;
            "".to_string()
        }
    };

    let scripts_path = match mod_info.get("scripts_path") {
        Some(x) => x.as_str().unwrap().to_string(),
        None => {
            no_scripts = true;
            "".to_string()
        }
    };

    if platform == "PC" && !no_custom_textures {
        return Err("custom textures not allowed on pc.".into());
    }

    if platform != "PC" || game != "EMR" {
        return Err("custom scripts only available with EMR".into());
    }
    final_mod_info.scripts_path = scripts_path.clone();
    final_mod_info.custom_textures_path = custom_textures_path.clone();
    final_mod_info.custom_game_files_path = custom_game_files_path.clone();

    if !no_custom_files {
        if custom_game_files_path.trim().is_empty() {
            return Err("custom game files path is empty.".into());
        }
        if PathBuf::from(&custom_game_files_path).is_absolute() {
            return Err("you are not allowed to have absolute paths on custom file path.".into());
        }
        if !PathBuf::from(&path).join(&custom_game_files_path).exists() {
            return Err("custom game files path does not exist.".into());
        }

        final_mod_info
            .auto_generated_tags
            .push("gamefile-mod".to_string())
    }

    if !no_custom_textures {
        if custom_textures_path.trim().is_empty() {
            return Err("custom textures path is empty.".into());
        }
        if PathBuf::from(&custom_textures_path).is_absolute() {
            return Err(
                "you are not allowed to have absolute paths on custom textures path.".into(),
            );
        }
        if !PathBuf::from(&path).join(&custom_textures_path).exists() {
            return Err("custom textures path does not exist.".into());
        }

        final_mod_info
            .auto_generated_tags
            .push("texture-mod".to_string())
    }

    if !no_scripts {
        if scripts_path.trim().is_empty() {
            return Err("scripts path is empty.".into());
        }
        if PathBuf::from(&scripts_path).is_absolute() {
            return Err("you are not allowed to have absolute paths on custom script path.".into());
        }
        if !PathBuf::from(&path).join(&scripts_path).exists() {
            return Err("custom script path does not exist.".into());
        }

        final_mod_info
            .auto_generated_tags
            .push("script-mod".to_string())
    }
    let icon_path = mod_info.get("icon_path").unwrap().as_str().unwrap();

    final_mod_info.icon_path = icon_path.trim().to_string();

    if icon_path.trim().is_empty() {
        return Err("mod icon path is empty.".into());
    }

    if PathBuf::from(&icon_path).is_absolute() {
        return Err("you are not allowed to have absolute paths on mod icon.".into());
    }

    if PathBuf::from(&icon_path).exists() {
        return Err("mod icon does not exist.".into());
    }

    match mod_info.get("dependencies") {
        Some(x) => {
            let array = x.as_array().unwrap();
            for element in array {
                let dependency = element.as_str().unwrap().to_string();
                for char in dependency.trim().chars() {
                    if !char.is_alphanumeric() {
                        return Err("only alphanumerics are allowed in dependency list.".into());
                    }
                }

                final_mod_info.dependencies.push(dependency.to_string());
            }
        }
        None => {}
    };

    for entry in walkdir::WalkDir::new(path).into_iter() {
        let res = entry?;
        if res.path().is_dir() {
            continue;
        }

        let extension = match res.path().extension() {
            Some(s) => s,
            None => OsStr::new(""),
        };

        if !extension.is_empty() {
            let formatted_extension = extension.to_str().unwrap().to_string().to_lowercase();
            if BANNED_EXTENSIONS.contains(&formatted_extension.as_str()) {
                return Err(format!("mod contains illegal file ({})", formatted_extension).into());
            }
        }
    }

    Ok(final_mod_info)
}

pub fn generate_project(_game: String, _platform: String, path: String) -> std::io::Result<()> {
    let full_path = PathBuf::from(path);

    let mut meta_file = File::create(Path::new(&full_path).join("mod.json"))?;

    let game = _game.to_uppercase();
    let platform = _platform.to_uppercase();

    if game == "EM1" && platform == "PC" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "impossible combination (EM1/PC)",
        ));
    }

    if game == "EMR" && platform == "WII" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "impossible combination (EMR/WII)",
        ));
    }

    let mut mod_info = ModInfo {
        name: "Auto Generated Mod".to_string(),
        game: game.to_string(),
        platform: platform.to_string(),
        description: "".to_string(),
        shortdescription: "Generated with eml-validate".to_string(),
        dependencies: Vec::new(),
        custom_textures_path: "textures".to_string(),
        custom_game_files_path: "games".to_string(),
        scripts_path: "".to_string(),
        icon_path: "icon.png".to_string(),
        auto_generated_tags: Vec::new(),
    };

    if platform == "PC" {
        mod_info.custom_textures_path = "".to_string();
    }

    if game == "EMR" {
        mod_info.scripts_path = "scripts".to_string();
    }

    if !mod_info.scripts_path.is_empty() {
        std::fs::create_dir_all(Path::new(&full_path).join(mod_info.scripts_path.clone()))?;
    }
    if !mod_info.custom_game_files_path.is_empty() {
        std::fs::create_dir_all(
            Path::new(&full_path).join(mod_info.custom_game_files_path.clone()),
        )?;
    }
    if !mod_info.custom_textures_path.is_empty() {
        std::fs::create_dir_all(Path::new(&full_path).join(mod_info.custom_textures_path.clone()))?;
    }

    let stringified = serde_json::to_string(&mod_info)?;
    meta_file.write_all(stringified.as_bytes())?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub game: String,
    pub platform: String,
    pub description: String,
    pub shortdescription: String,
    pub dependencies: Vec<String>,
    pub custom_textures_path: String,
    pub custom_game_files_path: String,
    pub scripts_path: String,
    pub icon_path: String,
    pub auto_generated_tags: Vec<String>,
}
