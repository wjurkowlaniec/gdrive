use crate::files::upload::{Config, Error, upload, upload_directory};
use crate::files::list::{ListFilesConfig, ListQuery};
use crate::hub::Hub;
use std::io::{self, Write};
use std::str::FromStr;

/// Upload with overwrite check
/// 
/// This function checks if files with the same name already exist in the destination
/// and prompts the user for confirmation before overwriting.
pub async fn upload_with_overwrite_check(hub: &Hub, config: Config) -> Result<(), Error> {
    if config.file_path.is_dir() && config.upload_directories {
        // For recursive directory uploads, check the top-level files/directories
        println!("Checking for existing files in destination...");
        
        // Get the parent ID
        let parent_id = match &config.parents {
            Some(parents) if !parents.is_empty() => &parents[0],
            _ => "root",
        };
        
        // List files in the destination folder
        let query = format!("'{}' in parents and trashed = false", parent_id);
        let list_config = ListFilesConfig {
            query: ListQuery::from_str(&query).map_err(|e| Error::Other(e.to_string()))?,
            order_by: Default::default(),
            max_files: 100, // Limit to 100 files for performance
        };
        
        let remote_files = crate::files::list::list_files(hub, &list_config)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        
        // Get local top-level files/directories
        let local_entries = std::fs::read_dir(&config.file_path)
            .map_err(|e| Error::Other(format!("Failed to read directory: {}", e)))?;
        
        let mut potential_overwrites = Vec::new();
        
        // Check for potential overwrites
        for entry in local_entries {
            if let Ok(entry) = entry {
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Check if this name exists in remote files
                for remote_file in &remote_files {
                    if let Some(remote_name) = &remote_file.name {
                        if remote_name == &name {
                            let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                "directory"
                            } else {
                                "file"
                            };
                            
                            potential_overwrites.push(format!("{} ({})", name, file_type));
                            break;
                        }
                    }
                }
            }
        }
        
        if !potential_overwrites.is_empty() {
            println!("The following files/directories will be overwritten:");
            for item in &potential_overwrites {
                println!("  - {}", item);
            }
        } else {
            println!("No existing files will be overwritten at the top level.");
            println!("Note: Subdirectories may still contain files that will be overwritten.");
        }
        
        println!("Do you want to continue? [y/N]");
        
        if !confirm_overwrite() {
            println!("Upload cancelled.");
            return Ok(());
        }
        
        upload_directory(hub, &config, Default::default()).await
    } else if config.file_path.is_dir() {
        // Non-recursive directory upload - error out
        Err(Error::IsDirectory(config.file_path.clone()))
    } else {
        // Single file upload - check if file exists
        let file_name = config.file_path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        let parent_id = match &config.parents {
            Some(parents) if !parents.is_empty() => &parents[0],
            _ => "root",
        };
        
        let query = format!(
            "'{}' in parents and name = '{}' and trashed = false",
            parent_id, file_name
        );
        
        let list_config = ListFilesConfig {
            query: ListQuery::from_str(&query).map_err(|e| Error::Other(e.to_string()))?,
            order_by: Default::default(),
            max_files: 1,
        };
        
        let files = crate::files::list::list_files(hub, &list_config)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        
        if !files.is_empty() {
            println!("File '{}' already exists in the destination.", file_name);
            println!("Do you want to overwrite it? [y/N]");
            
            if !confirm_overwrite() {
                println!("Upload cancelled.");
                return Ok(());
            }
        }
        
        upload(config).await
    }
}

/// Helper function to get user confirmation
fn confirm_overwrite() -> bool {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    
    let input = input.trim().to_lowercase();
    input == "y" || input == "yes"
}
