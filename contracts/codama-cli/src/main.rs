// //! Codama IDL build script.
// use {
//     codama::Codama,
//     std::{env, fs, path::Path},
// };

// fn main() {
//     // Only print cargo directives when running as a build script
//     if env::var("CARGO").is_ok() {
//         println!("cargo:rerun-if-changed=src/");
//         println!("cargo:rerun-if-env-changed=GENERATE_IDL");
//     }

//     match generate_idl() {
//         Ok(()) => {
//             if env::var("CARGO").is_ok() {
//                 println!("cargo:warning=IDL generation completed successfully");
//             } else {
//                 println!("IDL generation completed successfully");
//             }
//         }
//         Err(e) => {
//             if env::var("CARGO").is_ok() {
//                 println!("cargo:warning=Failed to generate IDL: {}", e);
//             } else {
//                 eprintln!("Failed to generate IDL: {}", e);
//                 std::process::exit(1);
//             }
//         }
//     }
// }

// fn generate_idl() -> Result<(), Box<dyn std::error::Error>> {
//     // Generate IDL.
//     let manifest_dir = if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
//         dir
//     } else {
//         // When running manually, use current directory or find Cargo.toml
//         find_manifest_dir()?
//     };

//     let crate_path = Path::new(&manifest_dir);
//     println!("crate_path: {:#?}", crate_path);

//     let crate_path =
//         Path::new("/home/m_daeva/dev/pinocchio-solveil/contracts/packages/registry-cpi/Cargo.toml");
//     println!("packages_path: {:#?}", crate_path);

//     println!("0 ++++++++++++++++++++++++++++++++++++++++++++++");
//     let codama = Codama::load(crate_path)?;
//     println!("1 ++++++++++++++++++++++++++++++++++++++++++++++");
//     let idl_json = codama.get_json_idl()?;

//     // Parse and format the JSON with pretty printing.
//     let parsed: serde_json::Value = serde_json::from_str(&idl_json)?;
//     let mut formatted_json = serde_json::to_string_pretty(&parsed)?;
//     formatted_json.push('\n');

//     // Write IDL file.
//     let idl_path = Path::new(&manifest_dir).join("idl.json");
//     fs::write(&idl_path, &formatted_json)?;

//     let message = format!("IDL written to: {}", idl_path.display());
//     if env::var("CARGO").is_ok() {
//         println!("cargo:warning={}", message);
//     } else {
//         println!("{}", message);
//     }

//     Ok(())
// }

// fn find_manifest_dir() -> Result<String, Box<dyn std::error::Error>> {
//     let current_dir = env::current_dir()?;

//     // Check current directory first
//     if current_dir.join("Cargo.toml").exists() {
//         return Ok(current_dir.to_string_lossy().to_string());
//     }

//     // Walk up the directory tree to find Cargo.toml
//     let mut path = current_dir.as_path();
//     while let Some(parent) = path.parent() {
//         if parent.join("Cargo.toml").exists() {
//             return Ok(parent.to_string_lossy().to_string());
//         }
//         path = parent;
//     }

//     Err("Could not find Cargo.toml in current directory or any parent directory".into())
// }

//! Codama IDL build script.
use {
    codama::Codama,
    std::{env, fs, path::Path},
};

fn main() {
    // Only print cargo directives when running as a build script
    if env::var("CARGO").is_ok() {
        println!("cargo:rerun-if-changed=src/");
        println!("cargo:rerun-if-env-changed=GENERATE_IDL");
    }

    // Get target path from command line argument or default
    let args: Vec<String> = env::args().collect();
    let target_path = if args.len() > 1 {
        args[1].clone()
    } else {
        // Default behavior - use current directory or CARGO_MANIFEST_DIR
        if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
            dir
        } else {
            ".".to_string()
        }
    };

    match generate_idl(&target_path) {
        Ok(()) => {
            if env::var("CARGO").is_ok() {
                println!("cargo:warning=IDL generation completed successfully");
            } else {
                println!("IDL generation completed successfully");
            }
        }
        Err(e) => {
            if env::var("CARGO").is_ok() {
                println!("cargo:warning=Failed to generate IDL: {}", e);
            } else {
                eprintln!("Failed to generate IDL: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn generate_idl(target_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let crate_path = Path::new(target_path);
    println!(
        "Attempting to load Codama from path: {}",
        crate_path.display()
    );

    // Check if Cargo.toml exists
    let cargo_toml = crate_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(format!("Cargo.toml not found at: {}", cargo_toml.display()).into());
    }
    println!("Found Cargo.toml at: {}", cargo_toml.display());

    // Check for common source files
    let lib_rs = crate_path.join("src/lib.rs");
    let main_rs = crate_path.join("src/main.rs");

    if lib_rs.exists() {
        println!("Found lib.rs at: {}", lib_rs.display());
    } else if main_rs.exists() {
        println!("Found main.rs at: {}", main_rs.display());
    } else {
        println!("Warning: Neither lib.rs nor main.rs found in src/");
    }

    // Try to load with more detailed error reporting
    let codama = match Codama::load(crate_path) {
        Ok(codama) => codama,
        Err(e) => {
            // Try to provide more specific error information
            println!("Codama load failed. Attempting to diagnose...");

            // Check if we can read the lib.rs file directly
            let lib_rs_path = crate_path.join("src/lib.rs");
            if let Err(read_err) = fs::read_to_string(&lib_rs_path) {
                return Err(format!(
                    "Cannot read lib.rs at '{}': {}. Original Codama error: {}",
                    lib_rs_path.display(),
                    read_err,
                    e
                )
                .into());
            } else {
                println!("lib.rs is readable, so the issue is likely in parsing or module loading");
            }

            // Check for potential issues with the Cargo.toml
            let cargo_toml_content = fs::read_to_string(crate_path.join("Cargo.toml"))?;
            println!(
                "Cargo.toml content preview:\n{}",
                cargo_toml_content
                    .lines()
                    .take(10)
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            return Err(format!(
                "Failed to load Codama from '{}': {}",
                crate_path.display(),
                e
            )
            .into());
        }
    };
    let idl_json = codama.get_json_idl()?;

    // Parse and format the JSON with pretty printing.
    let parsed: serde_json::Value = serde_json::from_str(&idl_json)?;
    let mut formatted_json = serde_json::to_string_pretty(&parsed)?;
    formatted_json.push('\n');

    // Write IDL file.
    let idl_path = crate_path.join("idl.json");
    fs::write(&idl_path, &formatted_json)?;

    let message = format!("IDL written to: {}", idl_path.display());
    if env::var("CARGO").is_ok() {
        println!("cargo:warning={}", message);
    } else {
        println!("{}", message);
    }

    Ok(())
}
