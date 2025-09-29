use crate::common::delegate::BackoffConfig;
use crate::common::delegate::ChunkSize;
use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::file_info;
use crate::common::file_info::FileInfo;
use crate::common::file_tree;
use crate::common::file_tree::FileTree;
use crate::common::hub_helper;
use crate::common::id_gen::IdGen;
use crate::files;
use crate::files::info::DisplayConfig;
use crate::files::path_utils;
use crate::hub::Hub;
use human_bytes::human_bytes;
use mime::Mime;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::io::{self, empty};
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

pub struct Config {
    pub file_path: PathBuf,
    pub mime_type: Option<Mime>,
    pub parents: Option<Vec<String>>,
    pub chunk_size: ChunkSize,
    pub print_chunk_errors: bool,
    pub print_chunk_info: bool,
    pub upload_directories: bool,
    pub print_only_id: bool,
}

impl Config {
    pub fn with_remote_path(mut self, remote_path: Option<String>) -> Self {
        if let Some(remote_path) = remote_path {
            self.parents = Some(vec![remote_path]);
        }
        self
    }
}

pub async fn upload(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let delegate_config = UploadDelegateConfig {
        chunk_size: config.chunk_size.clone(),
        backoff_config: BackoffConfig {
            max_retries: 100000,
            min_sleep: Duration::from_secs(1),
            max_sleep: Duration::from_secs(60),
        },
        print_chunk_errors: config.print_chunk_errors,
        print_chunk_info: config.print_chunk_info,
    };

    err_if_directory(&config.file_path, &config)?;

    if config.file_path.is_dir() {
        upload_directory(&hub, &config, delegate_config).await?;
    } else {
        upload_regular(&hub, &config, delegate_config).await?;
    }

    Ok(())
}

async fn upload_regular(
    hub: &Hub,
    config: &Config,
    delegate_config: UploadDelegateConfig,
) -> Result<(), Error> {
    let file = fs::File::open(&config.file_path)
        .map_err(|err| Error::OpenFile(config.file_path.clone(), err))?;

    let file_info = FileInfo::from_file(
        &file,
        &file_info::Config {
            file_path: config.file_path.clone(),
            mime_type: config.mime_type.clone(),
            parents: config.parents.clone(),
        },
    )
    .map_err(Error::FileInfo)?;

    let reader = std::io::BufReader::new(file);

    if !config.print_only_id {
        println!("Uploading {}", config.file_path.display());
    }

    let file = upload_file(hub, reader, None, file_info, delegate_config)
        .await
        .map_err(Error::Upload)?;

    if config.print_only_id {
        print!("{}", file.id.unwrap_or_default())
    } else {
        println!("File successfully uploaded");
        let fields = files::info::prepare_fields(&file, &DisplayConfig::default());
        files::info::print_fields(&fields);
    }

    Ok(())
}

pub async fn upload_directory(
    hub: &Hub,
    config: &Config,
    delegate_config: UploadDelegateConfig,
) -> Result<(), Error> {
    let mut ids = IdGen::new(hub, &delegate_config);
    let path = &config.file_path;
    let tree = FileTree::from_path(path, &mut ids)
        .await
        .map_err(Error::CreateFileTree)?;

    let tree_info = tree.info();

    if !config.print_only_id {
        println!(
            "Found {} files in {} directories with a total size of {}",
            tree_info.file_count,
            tree_info.folder_count,
            human_bytes(tree_info.total_file_size as f64)
        );
    }

    let mut folder_ids: std::collections::HashMap<PathBuf, String> = std::collections::HashMap::new();

    for folder in tree.folders() {
        let folder_path = folder.relative_path();
        let folder_name = folder_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let parent_id = if folder.parent.is_none() {
            // This is the root folder, use the config's parents if available
            match &config.parents {
                Some(parents) if !parents.is_empty() => &parents[0],
                _ => {
                    return Err(Error::Other(format!(
                        "No parent specified for root directory {}",
                        folder_path.display()
                    )))
                }
            }
        } else {
            // This is a subfolder, get its parent from the folder_ids map
            let parent = folder.parent.as_ref().unwrap();
            match folder_ids.get(&parent.relative_path()) {
                Some(id) => id,
                None => {
                    return Err(Error::Other(format!(
                        "Failed to find parent for {}",
                        folder_path.display()
                    )))
                }
            }
        };

        // For directories, we don't need to read the file content
        // Just create the folder metadata
        let folder_info = FileInfo {
            name: folder_path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            // Use the correct MIME type for Google Drive folders
            mime_type: "application/vnd.google-apps.folder".parse().unwrap(),
            parents: Some(vec![parent_id.to_string()]),
            size: 0,
        };
        
        // Create an empty reader for the directory
        let reader = std::io::empty();

        let file = upload_file(hub, reader, None, folder_info, delegate_config.clone())
            .await
            .map_err(Error::Upload)?;

        if let Some(id) = &file.id {
            folder_ids.insert(folder_path, id.clone());
        } else {
            return Err(Error::DriveFolderMissingId);
        }
    }

    // The first loop already created all directories, now upload files
    for file in tree.root.files() {
        let file_path = file.relative_path();
        let parent_path = file_path.parent().unwrap_or_else(|| Path::new(""));
        
        let parent_id = folder_ids.get(parent_path).or_else(|| {
            if parent_path == Path::new("") {
                config.parents.as_ref().and_then(|p| p.first())
            } else {
                None
            }
        });
        
        let parent_id = match parent_id {
            Some(id) => id,
            None => {
                return Err(Error::Other(format!(
                    "Failed to find parent for {}",
                    file_path.display()
                )))
            }
        };

        if !config.print_only_id {
            println!(
                "Uploading file '{}' to parent id: {}",
                file_path.display(),
                parent_id
            );
        }

        let file_handle = fs::File::open(&file_path)
            .map_err(|err| Error::OpenFile(file_path.clone(), err))?;
            
        let file_info = FileInfo::from_file(
            &file_handle,
            &file_info::Config {
                file_path: file_path.clone(),
                mime_type: config.mime_type.clone(),
                parents: Some(vec![parent_id.to_string()]),
            },
        )
        .map_err(Error::FileInfo)?;

        // Reopen the file for reading
        let file = fs::File::open(&file_path)
            .map_err(|err| Error::OpenFile(file_path.clone(), err))?;

        let reader = std::io::BufReader::new(file);

        let _file = upload_file(hub, reader, None, file_info, delegate_config.clone())
            .await
            .map_err(Error::Upload)?;
    }

    if !config.print_only_id {
        println!(
            "Uploaded {} files in {} directories with a total size of {}",
            tree_info.file_count,
            tree_info.folder_count,
            human_bytes(tree_info.total_file_size as f64)
        );
    }

    // This section was removed as it contained references to non-existent variables

    Ok(())
}

pub async fn upload_file<RS>(
    hub: &Hub,
    src_file: RS,
    file_id: Option<String>,
    file_info: FileInfo,
    delegate_config: UploadDelegateConfig,
) -> Result<google_drive3::api::File, google_drive3::Error>
where
    RS: google_drive3::client::ReadSeek,
{
    let dst_file = google_drive3::api::File {
        id: file_id,
        name: Some(file_info.name),
        mime_type: Some(file_info.mime_type.to_string()),
        parents: file_info.parents,
        ..google_drive3::api::File::default()
    };

    let chunk_size_bytes = delegate_config.chunk_size.in_bytes();
    let mut delegate = UploadDelegate::new(delegate_config);

    let req = hub
        .files()
        .create(dst_file)
        .param("fields", "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink")
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true);

    let (_, file) = if file_info.size > chunk_size_bytes {
        req.upload_resumable(src_file, file_info.mime_type).await?
    } else {
        req.upload(src_file, file_info.mime_type).await?
    };

    Ok(file)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    FileHelper(String),
    ResolvePath(path_utils::PathResolutionError),
    FileInfo(file_info::Error),
    OpenFile(PathBuf, io::Error),
    Upload(google_drive3::Error),
    IsDirectory(PathBuf),
    DriveFolderMissingId,
    CreateFileTree(file_tree::Error),
    Mkdir(google_drive3::Error),
    Other(String),
}

// Implement From for google_drive3::Error to allow using ? operator
impl From<google_drive3::Error> for Error {
    fn from(err: google_drive3::Error) -> Self {
        Error::Upload(err)
    }
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::FileHelper(err) => write!(f, "{}", err),
            Error::ResolvePath(err) => write!(f, "{}", err),
            Error::FileInfo(err) => write!(f, "{}", err),
            Error::OpenFile(path, err) => {
                write!(f, "Failed to open file '{}': {}", path.display(), err)
            }
            Error::Upload(err) => write!(f, "Failed to upload file: {}", err),
            Error::IsDirectory(path) => write!(
                f,
                "'{}' is a directory. Use --recursive to upload directories.",
                path.display()
            ),
            Error::DriveFolderMissingId => write!(f, "Folder created on drive does not have an id"),
            Error::CreateFileTree(err) => write!(f, "Failed to create file tree: {}", err),
            Error::Mkdir(err) => write!(f, "Failed to create directory: {}", err),
            Error::Other(err) => write!(f, "{}", err),
        }
    }
}

impl Error {
    fn description(&self) -> &str {
        match self {
            Error::Hub(_) => "Failed to get hub",
            Error::FileHelper(_) => "File helper error",
            Error::ResolvePath(_) => "Failed to resolve path",
            Error::FileInfo(_) => "Failed to get file info",
            Error::Mkdir(_) => "Failed to create directory",
            Error::OpenFile(_, _) => "Failed to open file",
            Error::Upload(_) => "Failed to upload file",
            Error::IsDirectory(_) => "Is a directory",
            Error::DriveFolderMissingId => "Drive folder missing id",
            Error::CreateFileTree(_) => "Failed to create file tree",
            Error::Other(_) => "Other error",
        }
    }
}

fn err_if_directory(path: &PathBuf, config: &Config) -> Result<(), Error> {
    if path.is_dir() && !config.upload_directories {
        return Err(Error::Other(format!(
            "'{}' is a directory. Use --recursive to upload directories.",
            path.display()
        )));
    }
    Ok(())
}
