use {
    codama::Codama,
    std::{fs, path::Path},
    toml::Value,
};

const WORKSPACE_CONFIG: &str = "../Cargo.toml";
const IDL_PATH: &str = "idl.json";

fn main() {
    match find_codama_crates() {
        Ok(path_list) => {
            println!("Found codama crates: {:?}", path_list);
            match generate_idl(&path_list) {
                Err(e) => {
                    eprintln!("Failed to generate IDL: {}", e);
                    std::process::exit(1);
                }
                _ => {
                    println!("✅ IDL is generated successfully!");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to find codama crates: {}", e);
            std::process::exit(1);
        }
    }
}

fn find_codama_crates() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Read workspace Cargo.toml
    let workspace_content = fs::read_to_string(WORKSPACE_CONFIG)?;
    let workspace_toml: Value = toml::from_str(&workspace_content)?;

    // Extract members from workspace
    let members = workspace_toml
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .ok_or("No workspace members found")?;

    // Convert members to base directory paths
    let mut base_dirs = Vec::new();
    for member in members {
        if let Some(member_str) = member.as_str() {
            if member_str.ends_with("/*") {
                // Remove the "/*" suffix to get the base directory
                let base_dir = member_str.trim_end_matches("/*");
                base_dirs.push(format!("../{}", base_dir));
            }
        }
    }

    println!("Base directories: {:?}", base_dirs);

    // Find all crates with codama dependency
    let mut codama_crates = Vec::new();

    for base_dir in base_dirs {
        let entries = fs::read_dir(&base_dir);
        if entries.is_err() {
            continue;
        }

        for entry in entries.unwrap() {
            if entry.is_err() {
                continue;
            }

            let path = entry.unwrap().path();
            if !path.is_dir() {
                continue;
            }

            let cargo_toml_path = path.join("Cargo.toml");
            if !cargo_toml_path.exists() {
                continue;
            }

            if !has_codama_dependency(&cargo_toml_path)? {
                continue;
            }

            if let Some(path_str) = path.to_str() {
                codama_crates.push(path_str.to_string());
            }
        }
    }

    Ok(codama_crates)
}

fn has_codama_dependency(cargo_toml_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(cargo_toml_path)?;
    let toml: Value = toml::from_str(&content)?;

    let check_section = |section_name: &str| -> bool {
        toml.get(section_name)
            .and_then(|section| section.get("codama"))
            .and_then(|codama| codama.as_table())
            .and_then(|table| table.get("workspace"))
            .and_then(|workspace| workspace.as_bool())
            .unwrap_or(false)
    };

    Ok(check_section("dependencies") || check_section("dev-dependencies"))
}

fn generate_idl(path_list: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Generate IDL
    let crate_path_list: Vec<_> = path_list.iter().map(|p| Path::new(p)).collect();
    let codama = Codama::load_all(&crate_path_list)?;
    let idl_json = codama.get_json_idl()?;

    // Parse and format the JSON with pretty printing
    let parsed: serde_json::Value = serde_json::from_str(&idl_json)?;
    let mut formatted_json = serde_json::to_string_pretty(&parsed)?;
    formatted_json.push('\n');

    // Write IDL file
    let idl_path = Path::new(IDL_PATH);
    fs::write(&idl_path, &formatted_json)?;

    Ok(())
}
