use anyhow::Error;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const ALLOWED_GAMES: [&str; 3] = ["EM1", "EM2", "EMR"];
const ALLOWED_PLATFORMS: [&str; 2] = ["WII", "PC"];
const BANNED_EXTENSIONS: [&str; 6] = ["dll", "so", "exe", "sh", "bat", "scr"]; // not technically
                                                                               // banned, but will
                                                                               // require analysis
                                                                               // by a moderator.
                                                                               // this does not prevent file instruction injections to already trusted file formats, but it at
                                                                               // least removes the most prevalent malware attack vectors.
const BANNED_PAK_FILES: [&str; 5] = [
    "global.utoc",
    "global.ucas",
    "recolored-WindowsNoEditor.pak",
    "recolored-WindowsNoEditor.ucas",
    "recolored-WindowsNoEditor.utoc",
];
pub fn validate(path: &PathBuf, strict: bool) -> Result<ModInfo, Error> {
    let mut final_mod_info: ModInfo = ModInfo {
        name: "".to_string(),
        game: "".to_string(),
        platform: "".to_string(),
        description: "".to_string(),
        short_description: "".to_string(),
        dependencies: Vec::new(),
        custom_textures_path: "".to_string(),
        custom_game_files_path: "".to_string(),
        scripts_path: "".to_string(),
        icon_path: "".to_string(),
        auto_generated_tags: Vec::new(),
    };
    println!("{}", &path.display());
    let mut mod_info_path = path.clone();
    mod_info_path.push("mod.json");

    let mut mod_description_path = path.clone();
    mod_description_path.push("description.md");

    if !mod_info_path.exists() {
        return Err(anyhow!("mod.json does not exist."));
    }
    let mut mod_info_file = File::open(mod_info_path)?;
    let mut mod_info_buffer = String::new();
    mod_info_file.read_to_string(&mut mod_info_buffer)?;
    let mod_info: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&mod_info_buffer)?;

    let name = mod_info.get("name").unwrap().as_str().unwrap();
    println!("{}", name);
    if name.trim().is_empty() {
        return Err(anyhow!("mod name is empty."));
    }

    final_mod_info.name = name.to_string();

    let short_description_value = mod_info.get("shortdescription");
    let mut no_short_description = false;

    match short_description_value {
        Some(x) => {
            let short_description = x.as_str().unwrap().trim().to_string();
            final_mod_info.short_description = short_description;
        }
        None => no_short_description = true,
    }

    if mod_description_path.exists() {
        let mut mod_description_file = File::open(mod_description_path)?;
        let mut mod_description = String::new();

        mod_description_file.read_to_string(&mut mod_description)?;

        if mod_description.trim().is_empty() {
            return Err(anyhow!("mod description is empty."));
        }

        final_mod_info.description = mod_description.trim().to_string();

        if no_short_description {
            final_mod_info.short_description = "clone".to_string();
        }
    }

    let mut game = match mod_info.get("game") {
        Some(x) => x.as_str().unwrap().to_string().to_uppercase(),
        None => "".to_string(),
    };
    let mut platform = match mod_info.get("platform") {
        Some(x) => x.as_str().unwrap().to_string().to_uppercase(),
        None => "".to_string(),
    };

    if platform.trim().is_empty() {
        if strict {
            return Err(anyhow!("mod platform is empty."));
        } else {
            platform = "WII".to_string();
        }
    }

    if game.trim().is_empty() {
        return Err(anyhow!("mod game type is empty."));
    }

    println!("{}", platform);

    final_mod_info.game = game.to_string();
    final_mod_info.platform = platform.to_string();

    println!("{}", game);

    if !ALLOWED_GAMES.contains(&game.as_str()) {
        return Err(anyhow!("could not recognize defined game."));
    }

    if !ALLOWED_PLATFORMS.contains(&platform.as_str()) {
        return Err(anyhow!("could not recognize defined platform."));
    }

    if game.to_string() == "EMR" && platform.to_string() == "WII" {
        return Err(anyhow!("impossible combination (emr/wii)"));
    }

    if game.to_string() == "EM1" && platform.to_string() == "PC" {
        return Err(anyhow!("impossible combination (em1/pc)"));
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

    if strict {
        if platform == "PC" && !no_custom_textures {
            return Err(anyhow!("custom textures not allowed on pc."));
        }

        if (platform != "PC" || game != "EMR") && !no_scripts {
            return Err(anyhow!("custom scripts only available with EMR"));
        }
    } else {
        if platform == "PC" && !no_custom_textures {
            no_custom_textures = true
        }

        if (platform != "PC" || game != "EMR") && !no_scripts {
            no_scripts = true;
        }
    }

    final_mod_info.scripts_path = scripts_path.clone();
    final_mod_info.custom_textures_path = custom_textures_path.clone();
    final_mod_info.custom_game_files_path = custom_game_files_path.clone();

    if !no_custom_files {
        if PathBuf::from(&custom_game_files_path).is_absolute() {
            return Err(anyhow!(
                "you are not allowed to have absolute paths on custom file path."
            ));
        }

        if strict {
            if !PathBuf::from(&path).join(&custom_game_files_path).exists() {
                return Err(anyhow!("custom game files path does not exist."));
            }
            if custom_game_files_path.trim().is_empty() {
                return Err(anyhow!("custom game files path is empty."));
            }
        }

        let pak_path = PathBuf::from(&path)
            .join(custom_game_files_path)
            .join("Paks");

        if platform == "PC" && game == "EMR" && pak_path.exists() {
            for pak in BANNED_PAK_FILES {
                let path = pak_path.clone().join(pak);
                if path.exists() {
                    return Err(anyhow!(format!("you are not allowed to modify any existing/forbidden PAK files. (global.utoc,global.ucas,recolored-WindowsNoEditor.pak,recolored-WindowsNoEditor.ucas,recolored-WindowsNoEditor.utoc) (VIOLATINGFILE={})", path.display())));
                }
            }
        }

        final_mod_info
            .auto_generated_tags
            .push("gamefile-mod".to_string())
    }

    if !no_custom_textures {
        if PathBuf::from(&custom_textures_path).is_absolute() {
            return Err(anyhow!(
                "you are not allowed to have absolute paths on custom textures path."
            ));
        }

        if strict {
            if custom_textures_path.trim().is_empty() {
                return Err(anyhow!("custom textures path is empty."));
            }
            if !PathBuf::from(&path).join(&custom_textures_path).exists() {
                return Err(anyhow!("custom textures path does not exist."));
            }
        }

        final_mod_info
            .auto_generated_tags
            .push("texture-mod".to_string())
    }

    if !no_scripts {
        if scripts_path.trim().is_empty() {
            return Err(anyhow!("scripts path is empty."));
        }
        if PathBuf::from(&scripts_path).is_absolute() {
            return Err(anyhow!(
                "you are not allowed to have absolute paths on custom script path."
            ));
        }
        if !PathBuf::from(&path).join(&scripts_path).exists() {
            return Err(anyhow!("custom script path does not exist."));
        }

        final_mod_info
            .auto_generated_tags
            .push("script-mod".to_string())
    }
    let icon_path = mod_info.get("icon_path").unwrap().as_str().unwrap();

    final_mod_info.icon_path = icon_path.trim().to_string();

    if icon_path.trim().is_empty() {
        return Err(anyhow!("mod icon path is empty."));
    }

    if PathBuf::from(&icon_path).is_absolute() {
        return Err(anyhow!(
            "you are not allowed to have absolute paths on mod icon."
        ));
    }

    if PathBuf::from(&icon_path).exists() {
        return Err(anyhow!("mod icon does not exist."));
    }

    match mod_info.get("dependencies") {
        Some(x) => {
            let array = x.as_array().unwrap();
            for element in array {
                let dependency = element.as_str().unwrap().to_string();
                for char in dependency.trim().chars() {
                    if !char.is_alphanumeric() {
                        return Err(anyhow!(
                            "only alphanumerics are allowed in dependency list."
                        ));
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

        let extension = res.path().extension().unwrap_or_else(|| OsStr::new(""));

        if !extension.is_empty() {
            let formatted_extension = extension.to_str().unwrap().to_string().to_lowercase();
            if BANNED_EXTENSIONS.contains(&formatted_extension.as_str()) {
                return Err(anyhow!(format!(
                    "mod contains illegal file ({})",
                    formatted_extension
                )));
            }
        }
    }

    Ok(final_mod_info)
}

pub fn generate_project(
    _game: String,
    _platform: String,
    name: String,
    description: String,
    short_description: String,
    path: String,
) -> Result<()> {
    println!("Generating Mod");
    let full_path = PathBuf::from(path);

    let mut mod_info: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    let mut meta_file = File::create(Path::new(&full_path).join("mod.json"))?;

    let game = _game.to_uppercase();
    let platform = _platform.to_uppercase();

    if game == "EM1" && platform == "PC" {
        return Err(anyhow!("impossible combination (EM1/PC)",));
    }

    if game == "EMR" && platform == "WII" {
        return Err(anyhow!("impossible combination (EMR/WII)",));
    }

    if short_description.trim() == "" {
        return Err(anyhow!("short description must have content",));
    }
    if description.trim() == "" {
        return Err(anyhow!("description must have content",));
    }

    mod_info.insert("name".to_string(), serde_json::Value::String(name.clone()));
    mod_info.insert(
        "short_description".to_string(),
        serde_json::Value::String(short_description),
    );
    mod_info.insert("game".to_string(), serde_json::Value::String(game.clone()));
    mod_info.insert(
        "platform".to_string(),
        serde_json::Value::String(platform.clone()),
    );
    mod_info.insert(
        "custom_game_files".to_string(),
        serde_json::Value::String("files".to_string()),
    );
    mod_info.insert(
        "icon_path".to_string(),
        serde_json::Value::String("icon.png".to_string()),
    );

    fs::create_dir_all(Path::new(&full_path).join("files"))?;
    File::create(&full_path.clone().join("description.md"))?.write_all(description.as_bytes())?;

    if platform == "WII" {
        mod_info.insert(
            "custom_textures_path".to_string(),
            serde_json::Value::String("textures".to_string()),
        );
        fs::create_dir_all(Path::new(&full_path).join("textures"))?;
    }

    if game == "EMR" {
        mod_info.insert(
            "scripts_path".to_string(),
            serde_json::Value::String("scripts".to_string()),
        );
        fs::create_dir_all(Path::new(&full_path).join("scripts"))?;
    }

    meta_file.write_all(serde_json::to_string(&mod_info)?.as_bytes())?;
    println!("Finished generating mod");
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub game: String,
    pub platform: String,
    pub description: String,
    pub short_description: String,
    pub dependencies: Vec<String>,
    pub custom_textures_path: String,
    pub custom_game_files_path: String,
    pub scripts_path: String,
    pub icon_path: String,
    pub auto_generated_tags: Vec<String>,
}

impl ModInfo {
    pub fn new() -> ModInfo {
        ModInfo {
            name: "".to_string(),
            game: "".to_string(),
            platform: "".to_string(),
            scripts_path: "".to_string(),
            custom_game_files_path: "".to_string(),
            custom_textures_path: "".to_string(),
            description: "".to_string(),
            short_description: "".to_string(),
            dependencies: Vec::new(),
            icon_path: "".to_string(),
            auto_generated_tags: Vec::new(),
        }
    }
}
