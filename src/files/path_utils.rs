use crate::files;
use crate::files::list::{ListFilesConfig, ListQuery};
use crate::hub::Hub;
use google_drive3::api::File;
use std::fmt;
use std::str::FromStr;
use regex;

pub async fn resolve_path(hub: &Hub, path: &str) -> Result<File, PathResolutionError> {
    let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return Err(PathResolutionError::InvalidPath);
    }

    let mut current_id = "root".to_string();
    let mut current_file = files::info::get_file(hub, &current_id)
        .await
        .map_err(|e| PathResolutionError::ApiError(e.to_string()))?;

    for part in parts.iter() {
        let query = format!(
            "'{}' in parents and name = '{}' and trashed = false",
            current_id, part
        );
        
        let config = ListFilesConfig {
            query: ListQuery::from_str(&query).map_err(|e| PathResolutionError::ApiError(e.to_string()))?,
            order_by: Default::default(),
            max_files: 1,
        };

        let files = files::list::list_files(hub, &config)
            .await
            .map_err(|e| PathResolutionError::ApiError(e.to_string()))?;

        if let Some(file) = files.into_iter().next() {
            current_id = file.id.clone().unwrap_or_default();
            current_file = file;
        } else {
            return Err(PathResolutionError::NotFound(part.to_string()));
        }
    }

    Ok(current_file)
}

/// Resolves a path, creating directories as needed
pub async fn resolve_or_create_path(hub: &Hub, path: &str) -> Result<File, PathResolutionError> {
    // If path is empty or just "/", return root folder
    let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        let root = files::info::get_file(hub, "root")
            .await
            .map_err(|e| PathResolutionError::ApiError(e.to_string()))?;
        return Ok(root);
    }
    
    let mut current_id = "root".to_string();
    let mut current_file = files::info::get_file(hub, &current_id)
        .await
        .map_err(|e| PathResolutionError::ApiError(e.to_string()))?;
    
    for part in parts.iter() {
        // Try to find the folder
        let query = format!(
            "'{}' in parents and name = '{}' and trashed = false",
            current_id, part
        );
        
        let config = ListFilesConfig {
            query: ListQuery::from_str(&query).map_err(|e| PathResolutionError::ApiError(e.to_string()))?,
            order_by: Default::default(),
            max_files: 1,
        };

        let files = files::list::list_files(hub, &config)
            .await
            .map_err(|e| PathResolutionError::ApiError(e.to_string()))?;

        if let Some(file) = files.into_iter().next() {
            current_id = file.id.clone().unwrap_or_default();
            current_file = file;
        } else {
            // Folder not found, create it
            let mkdir_config = files::mkdir::Config {
                id: None,
                name: part.to_string(),
                parents: Some(vec![current_id]),
                print_only_id: false,
            };
            
            let new_folder = files::mkdir::create_directory(hub, &mkdir_config, Default::default())
                .await
                .map_err(|e| PathResolutionError::CreateDirectoryError(e.to_string()))?;
            
            current_id = new_folder.id.clone().ok_or(PathResolutionError::MissingId)?;
            current_file = new_folder;
        }
    }
    
    Ok(current_file)
}

/// Resolves a path that may contain wildcards and returns a list of matching files
pub async fn resolve_wildcard_path(hub: &Hub, path: &str) -> Result<Vec<File>, PathResolutionError> {
    // Split the path into directory parts and the wildcard part
    let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return Err(PathResolutionError::InvalidPath);
    }
    
    // Check if the last part contains a wildcard
    let (dir_parts, wildcard_part) = if parts.last().unwrap().contains('*') {
        (&parts[0..parts.len()-1], parts.last().unwrap())
    } else {
        // If no wildcard, just resolve as a regular path
        let file = resolve_path(hub, path).await?;
        return Ok(vec![file]);
    };
    
    // Navigate to the directory containing the wildcard
    let mut current_id = "root".to_string();
    
    for part in dir_parts {
        let query = format!(
            "'{}' in parents and name = '{}' and trashed = false",
            current_id, part
        );
        
        let config = ListFilesConfig {
            query: ListQuery::from_str(&query).map_err(|e| PathResolutionError::ApiError(e.to_string()))?,
            order_by: Default::default(),
            max_files: 1,
        };

        let files = files::list::list_files(hub, &config)
            .await
            .map_err(|e| PathResolutionError::ApiError(e.to_string()))?;

        if let Some(file) = files.into_iter().next() {
            current_id = file.id.clone().unwrap_or_default();
        } else {
            return Err(PathResolutionError::NotFound((*part).to_string()));
        }
    }
    
    // Convert wildcard to regex pattern
    let wildcard_regex = wildcard_to_regex(wildcard_part);
    let regex = regex::Regex::new(&wildcard_regex)
        .map_err(|e| PathResolutionError::InvalidWildcard(e.to_string()))?;
    
    // List all files in the directory and filter by the wildcard pattern
    let query = format!("'{}' in parents and trashed = false", current_id);
    let config = ListFilesConfig {
        query: ListQuery::from_str(&query).map_err(|e| PathResolutionError::ApiError(e.to_string()))?,
        order_by: Default::default(),
        max_files: 1000, // Set a reasonable limit
    };
    
    let files = files::list::list_files(hub, &config)
        .await
        .map_err(|e| PathResolutionError::ApiError(e.to_string()))?;
    
    // Filter files by the wildcard pattern
    let matching_files = files.into_iter()
        .filter(|file| {
            if let Some(name) = &file.name {
                regex.is_match(name)
            } else {
                false
            }
        })
        .collect::<Vec<_>>();
    
    if matching_files.is_empty() {
        return Err(PathResolutionError::NoMatchesFound(wildcard_part.to_string()));
    }
    
    Ok(matching_files)
}

/// Convert a wildcard pattern to a regex pattern
fn wildcard_to_regex(pattern: &str) -> String {
    let mut regex = String::from("^");
    
    for c in pattern.chars() {
        match c {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '\\' | '^' | '$' | '|' => {
                regex.push('\\');
                regex.push(c);
            },
            _ => regex.push(c),
        }
    }
    
    regex.push('$');
    regex
}

#[derive(Debug)]
pub enum PathResolutionError {
    InvalidPath,
    NotFound(String),
    ApiError(String),
    InvalidWildcard(String),
    NoMatchesFound(String),
    CreateDirectoryError(String),
    MissingId,
}

impl std::error::Error for PathResolutionError {}

impl fmt::Display for PathResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath => write!(f, "Invalid path provided"),
            Self::NotFound(part) => write!(f, "Path component not found: {}", part),
            Self::ApiError(e) => write!(f, "API error: {}", e),
            Self::InvalidWildcard(e) => write!(f, "Invalid wildcard pattern: {}", e),
            Self::NoMatchesFound(pattern) => write!(f, "No files matching pattern: {}", pattern),
            Self::CreateDirectoryError(e) => write!(f, "Failed to create directory: {}", e),
            Self::MissingId => write!(f, "Created directory is missing ID"),
        }
    }
}

