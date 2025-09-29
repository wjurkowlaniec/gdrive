pub mod about;
pub mod account;
pub mod app_config;
pub mod common;
pub mod drives;
pub mod files;
pub mod hub;
pub mod permissions;
pub mod version;

use clap::{Parser, Subcommand};
use common::delegate::ChunkSize;
use common::permission;
use crate::common::drive_file;
use crate::files::list::ListSortOrder;
use crate::common::hub_helper;
use crate::files::list::ListQuery;
use crate::files::info::info;
use crate::files::list::list;
use crate::files::download::download;
use crate::files::upload::upload;
use crate::files::upload_with_check::upload_with_overwrite_check;
use crate::files::update::update;
use crate::files::delete::delete;
use crate::files::mkdir::mkdir;
use crate::files::rename::rename;
use crate::files::mv::mv;
use crate::files::copy::copy;
use crate::files::import::import;
use crate::files::export::export;
use mime::Mime;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None, disable_version_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print information about gdrive
    About,

    /// Commands for managing accounts
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },

    /// Commands for managing drives
    Drives {
        #[command(subcommand)]
        command: DriveCommand,
    },

    /// Commands for managing files
    #[command(alias = "f")]
    Files {
        #[command(subcommand)]
        command: FileCommand,
    },

    /// Commands for managing file permissions
    Permissions {
        #[command(subcommand)]
        command: PermissionCommand,
    },

    /// Upload file or directory to Google Drive
    Push {
        /// Local file or directory path
        file_path: PathBuf,

        /// Remote destination path (e.g., "/path/to/destination")
        remote_path: String,

        /// MIME type (default: auto-detect)
        #[arg(short, long)]
        mime: Option<String>,

        /// Upload directories recursively
        #[arg(short, long)]
        recursive: bool,

        /// Overwrite existing files without prompting
        #[arg(long, short = 'y')]
        overwrite: bool,
    },

    /// Print version information
    Version,
}

#[derive(Subcommand)]
enum AccountCommand {
    /// Add an account
    Add,

    /// List all accounts
    List,

    /// Print current account
    Current,

    /// Switch to a different account
    Switch {
        /// Account name
        account_name: String,
    },

    /// Remove an account
    Remove {
        /// Account name
        account_name: String,
    },

    /// Export account, this will create a zip file of the account which can be imported
    Export {
        /// Account name
        account_name: String,
    },

    /// Import account that was created with the export command
    Import {
        /// Path to archive
        file_path: PathBuf,
    },
}

#[derive(Subcommand)]
enum DriveCommand {
    /// List drives
    List {
        /// Don't print header
        #[arg(long)]
        skip_header: bool,

        /// Field separator
        #[arg(long, default_value_t = String::from("\t"))]
        field_separator: String,
    },
}

#[derive(Subcommand)]
enum FileCommand {
    /// Print file info
    Info {
        /// File id or path
        file_id: String,
    },

    /// List files
    List {
        /// Query string for filtering files
        #[arg(short, long)]
        query: Option<String>,

        /// Max files to list
        #[arg(long, default_value_t = 30)]
        max: usize,

        /// Order by. See https://developers.google.com/drive/api/v3/reference/files/list
        #[arg(long, default_value_t = ListSortOrder::default())]
        order_by: ListSortOrder,

        /// List files in a specific folder
        #[arg(long, value_name = "DIRECTORY_ID")]
        parent: Option<String>,

        /// List files on a shared drive
        #[arg(long, value_name = "DRIVE_ID")]
        drive: Option<String>,

        /// Don't print header
        #[arg(long)]
        skip_header: bool,

        /// Show full file name without truncating
        #[arg(long)]
        full_name: bool,

        /// Field separator
        #[arg(long, default_value_t = String::from("\t"))]
        field_separator: String,
    },

    /// Download file
    Download {
        /// File id or path (e.g., "/path/to/file" or "file_id")
        file_id: String,

        /// Output path (directory or file path)
        #[arg(value_name = "OUTPUT_PATH")]
        output_path: Option<PathBuf>,

        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,

        /// Follow shortcuts
        #[arg(long)]
        follow_shortcuts: bool,

        /// Download directories
        #[arg(long)]
        recursive: bool,

        /// Destination path (default: current directory)
        #[arg(short, long)]
        destination: Option<PathBuf>,

        /// Output to stdout
        #[arg(long)]
        stdout: bool,
    },

    /// Upload file or directory
    Upload {
        /// Local file or directory path
        file_path: PathBuf,

        /// Remote destination path (e.g., "/path/to/destination")
        remote_path: Option<String>,

        /// MIME type (default: auto-detect)
        #[arg(short, long)]
        mime: Option<String>,

        /// Upload directories recursively
        #[arg(long)]
        recursive: bool,

        /// Chunk size in bytes
        #[arg(long)]
        chunk_size: Option<u64>,

        /// Print chunk errors
        #[arg(long)]
        print_chunk_errors: bool,

        /// Print chunk info
        #[arg(long)]
        print_chunk_info: bool,

        /// Print only the file ID
        #[arg(long)]
        print_only_id: bool,
    },

    /// Update file. This will create a new version of the file. The older versions will typically be kept for 30 days.
    Update {
        /// File id of the file you want ot update
        file_id: String,

        /// Path of file to upload
        file_path: Option<PathBuf>,

        /// Force mime type [default: auto-detect]
        #[arg(long, value_name = "MIME_TYPE")]
        mime: Option<Mime>,

        /// Set chunk size in MB, must be a power of two.
        #[arg(long, value_name = "1|2|4|8|16|32|64|128|256|512|1024|4096|8192", default_value_t = ChunkSize::default())]
        chunk_size: ChunkSize,

        /// Print errors occuring during chunk upload
        #[arg(long, value_name = "", default_value_t = false)]
        print_chunk_errors: bool,

        /// Print details about each chunk
        #[arg(long, value_name = "", default_value_t = false)]
        print_chunk_info: bool,
    },

    /// Delete file
    Delete {
        /// File id
        file_id: String,

        /// Delete directory and all it's content
        #[arg(long)]
        recursive: bool,
    },

    /// Create directory
    Mkdir {
        /// Name
        name: String,

        /// Create in an existing directory
        #[arg(long, value_name = "DIRECTORY_ID")]
        parent: Option<Vec<String>>,

        /// Print only id of folder
        #[arg(long, default_value_t = false)]
        print_only_id: bool,
    },

    /// Rename file/directory
    Rename {
        /// Id of file or directory
        file_id: String,

        /// New name
        name: String,
    },

    /// Move file/directory
    Move {
        /// Id of file or directory to move
        file_id: String,

        /// Id of folder to move to
        folder_id: String,
    },

    /// Copy file
    Copy {
        /// Id of file or directory to move
        file_id: String,

        /// Id of folder to copy to
        folder_id: String,
    },

    /// Import file as a google document/spreadsheet/presentation.
    /// Example of file types that can be imported: doc, docx, odt, pdf, html, xls, xlsx, csv, ods, ppt, pptx, odp
    Import {
        /// Path to file
        file_path: PathBuf,

        /// Upload to an existing directory
        #[arg(long, value_name = "DIRECTORY_ID")]
        parent: Option<Vec<String>>,

        /// Print only id of file
        #[arg(long, default_value_t = false)]
        print_only_id: bool,
    },

    /// Export google document to file
    Export {
        /// File id
        file_id: String,

        /// File path to export to. The file extension will determine the export format
        file_path: PathBuf,

        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
    },
}

#[derive(Subcommand)]
enum PermissionCommand {
    /// Grant permission to file
    Share {
        /// File id
        file_id: String,

        /// The role granted by this permission. Allowed values are: owner, organizer, fileOrganizer, writer, commenter, reader
        #[arg(long, default_value_t = permission::Role::default())]
        role: permission::Role,

        /// The type of the grantee. Valid values are: user, group, domain, anyone
        #[arg(long, default_value_t = permission::Type::default())]
        type_: permission::Type,

        /// Email address. Required for user and group type
        #[arg(long)]
        email: Option<String>,

        /// Domain. Required for domain type
        #[arg(long)]
        domain: Option<String>,

        /// Whether the permission allows the file to be discovered through search. This is only applicable for permissions of type domain or anyone
        #[arg(long)]
        discoverable: bool,
    },

    /// List permissions for a file
    List {
        /// File id
        file_id: String,

        /// Don't print header
        #[arg(long)]
        skip_header: bool,

        /// Field separator
        #[arg(long, default_value_t = String::from("\t"))]
        field_separator: String,
    },

    /// Revoke permissions for a file. If no other options are specified, the 'anyone' permission will be revoked
    Revoke {
        /// File id
        file_id: String,

        /// Revoke all permissions (except owner)
        #[arg(long)]
        all: bool,

        /// Revoke specific permission
        #[arg(long, value_name = "PERMISSION_ID")]
        id: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::About => {
            // fmt
            about::about()
        }

        Command::Account { command } => {
            // fmt
            match command {
                AccountCommand::Add => {
                    // fmt
                    account::add().await.unwrap_or_else(handle_error)
                }

                AccountCommand::List => {
                    // fmt
                    account::list().unwrap_or_else(handle_error)
                }

                AccountCommand::Current => {
                    // fmt
                    account::current().unwrap_or_else(handle_error)
                }

                AccountCommand::Switch { account_name } => {
                    // fmt
                    account::switch(account::switch::Config { account_name })
                        .unwrap_or_else(handle_error)
                }

                AccountCommand::Remove { account_name } => {
                    // fmt
                    account::remove(account::remove::Config { account_name })
                        .unwrap_or_else(handle_error)
                }

                AccountCommand::Export { account_name } => {
                    // fmt
                    account::export(account::export::Config { account_name })
                        .unwrap_or_else(handle_error)
                }

                AccountCommand::Import { file_path } => {
                    // fmt
                    account::import(account::import::Config {
                        archive_path: file_path,
                    })
                    .unwrap_or_else(handle_error)
                }
            }
        }

        Command::Drives { command } => {
            // fmt
            match command {
                DriveCommand::List {
                    skip_header,
                    field_separator,
                } => drives::list(drives::list::Config {
                    skip_header,
                    field_separator,
                })
                .await
                .unwrap_or_else(handle_error),
            }
        }

        Command::Files { command } => {
            match command {
                FileCommand::Info { file_id } => {
                    // fmt
                    info(files::info::Config { 
                        file_id,
                        size_in_bytes: false, // Display size in human-readable format
                    })
                        .await
                        .unwrap_or_else(handle_error)
                }

                FileCommand::List {
                    query,
                    max,
                    order_by,
                    parent,
                    drive,
                    skip_header,
                    full_name,
                    field_separator,
                } => {
                    let parent_query =
                        parent.map(|folder_id| ListQuery::FilesInFolder { folder_id });

                    let drive_query = drive.map(|drive_id| ListQuery::FilesOnDrive { drive_id });

                    let q = parent_query.or(drive_query).unwrap_or(ListQuery::default());

                    if let Some(query) = query {
                        list(files::list::Config {
                            query: ListQuery::Custom(query),
                            order_by,
                            max_files: max,
                            skip_header,
                            truncate_name: !full_name,
                            field_separator,
                        })
                        .await
                        .unwrap_or_else(handle_error)
                    } else {
                        list(files::list::Config {
                            query: q,
                            order_by,
                            max_files: max,
                            skip_header,
                            truncate_name: !full_name,
                            field_separator,
                        })
                        .await
                        .unwrap_or_else(handle_error)
                    }
                }

                FileCommand::Download {
                    file_id,
                    output_path,
                    overwrite,
                    follow_shortcuts,
                    recursive,
                    destination,
                    stdout,
                } => {
                    // For debugging
                    println!("Downloading file: {}", file_id);
                    let existing_file_action = if overwrite {
                        files::download::ExistingFileAction::Overwrite
                    } else {
                        files::download::ExistingFileAction::Abort
                    };

                    // Determine the destination based on parameters
                    // Priority: stdout > destination > current directory
                    let dst = if stdout {
                        files::download::Destination::Stdout
                    } else if let Some(path) = destination {
                        files::download::Destination::Path(path)
                    } else {
                        files::download::Destination::CurrentDir
                    };
                    
                    // If output_path is specified, use it as the destination
                    let dst = if let Some(path) = output_path {
                        println!("Output path specified: {}", path.display());
                        // Create parent directories if they don't exist
                        if let Some(parent) = path.parent() {
                            if !parent.exists() {
                                println!("Creating parent directory: {}", parent.display());
                                match std::fs::create_dir_all(parent) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        eprintln!("Error creating directory '{}': {}", parent.display(), e);
                                        std::process::exit(1);
                                    }
                                }
                            }
                        }
                        files::download::Destination::Path(path)
                    } else {
                        dst
                    };

                    // Check if file_id is a path (starts with '/')
                    let (file_id, path) = if file_id.starts_with('/') {
                        (String::new(), Some(file_id))
                    } else {
                        (file_id, None)
                    };
                    
                    // Ensure destination directory exists
                    if let files::download::Destination::Path(ref path) = dst {
                        // If path is a directory, ensure it exists
                        if path.is_dir() || path.extension().is_none() {
                            if !path.exists() {
                                println!("Creating directory: {}", path.display());
                                match std::fs::create_dir_all(path) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        eprintln!("Error creating directory '{}': {}", path.display(), e);
                                    }
                                }
                            }
                        } else {
                            // If path is a file, ensure its parent directory exists
                            if let Some(parent) = path.parent() {
                                if !parent.exists() {
                                    println!("Creating parent directory: {}", parent.display());
                                    match std::fs::create_dir_all(parent) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            eprintln!("Error creating directory '{}': {}", parent.display(), e);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    download(files::download::Config {
                        file_id,
                        path,
                        existing_file_action,
                        follow_shortcuts,
                        download_directories: recursive,
                        destination: dst,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Upload {
                    file_path,
                    remote_path,
                    mime,
                    recursive,
                    chunk_size,
                    print_chunk_errors,
                    print_chunk_info,
                    print_only_id,
                } => {
                    // Convert MIME string to Mime type if provided
                    let mime_type = mime.and_then(|m| m.parse::<Mime>().ok());
                    
                    // Create config with common parameters
                    let config = files::upload::Config {
                        file_path,
                        mime_type,
                        parents: None, // Will be set by with_remote_path
                        chunk_size: chunk_size.map(ChunkSize::from_mb).unwrap_or_default(),
                        print_chunk_errors,
                        print_chunk_info,
                        upload_directories: recursive,
                        print_only_id,
                    };
                    
                    // If remote_path is provided, resolve it to a folder ID
                    // If the path doesn't exist, create the necessary directories
                    let config = if let Some(remote_path) = remote_path {
                        let hub = hub_helper::get_hub().await.unwrap_or_else(|e| {
                            eprintln!("Error getting hub: {}", e);
                            std::process::exit(1);
                        });
                        
                        // Check if the remote path has a file extension (likely a filename)
                        let remote_path_clone = remote_path.clone();
                        let (dir_path, filename) = if remote_path_clone.contains('.') && !remote_path_clone.ends_with('/') {
                            // Extract the directory path and filename
                            let path = PathBuf::from(&remote_path_clone);
                            if let Some(parent) = path.parent() {
                                let parent_str = parent.to_string_lossy().to_string();
                                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                (parent_str, Some(filename))
                            } else {
                                (remote_path_clone, None)
                            }
                        } else {
                            (remote_path_clone, None)
                        };
                        
                        match files::path_utils::resolve_or_create_path(&hub, &dir_path).await {
                            Ok(file) => {
                                if !drive_file::is_directory(&file) {
                                    eprintln!("Error: '{}' exists but is not a directory", remote_path);
                                    std::process::exit(1);
                                }
                                
                                // If a filename was specified in the remote path, use it
                                if let Some(new_filename) = filename {
                                    println!("Uploading to: {} as {}", file.name.unwrap_or_default(), new_filename);
                                    
                                    // We'll use the new filename specified in the remote path
                                    
                                    // Create a temporary file with the new name
                                    let temp_dir = std::env::temp_dir();
                                    let temp_file_path = temp_dir.join(&new_filename);
                                    
                                    // Copy the file to the temporary location with the new name
                                    match std::fs::copy(&config.file_path, &temp_file_path) {
                                        Ok(_) => {
                                            files::upload::Config {
                                                file_path: temp_file_path,
                                                parents: Some(vec![file.id.unwrap_or_default()]),
                                                ..config
                                            }
                                        },
                                        Err(e) => {
                                            eprintln!("Error creating temporary file: {}", e);
                                            std::process::exit(1);
                                        }
                                    }
                                } else {
                                    println!("Uploading to: {}", file.name.unwrap_or_default());
                                    files::upload::Config {
                                        parents: Some(vec![file.id.unwrap_or_default()]),
                                        ..config
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("Error resolving remote path: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        config
                    };

                    upload(config)
                        .await
                        .unwrap_or_else(handle_error)
                }

                FileCommand::Update {
                    file_id,
                    file_path,
                    mime,
                    chunk_size,
                    print_chunk_errors,
                    print_chunk_info,
                } => {
                    // fmt
                    update(files::update::Config {
                        file_id,
                        file_path,
                        mime_type: mime,
                        chunk_size,
                        print_chunk_errors,
                        print_chunk_info,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Delete { file_id, recursive } => {
                    // fmt
                    delete(files::delete::Config {
                        file_id,
                        delete_directories: recursive,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Mkdir {
                    name,
                    parent,
                    print_only_id,
                } => {
                    // fmt
                    mkdir(files::mkdir::Config {
                        id: None,
                        name,
                        parents: parent,
                        print_only_id,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Rename { file_id, name } => {
                    // fmt
                    rename(files::rename::Config { file_id, name })
                        .await
                        .unwrap_or_else(handle_error)
                }

                FileCommand::Move { file_id, folder_id } => {
                    // fmt
                    mv(files::mv::Config {
                        file_id,
                        to_folder_id: folder_id,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Copy { file_id, folder_id } => {
                    // fmt
                    copy(files::copy::Config {
                        file_id,
                        to_folder_id: folder_id,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Import {
                    file_path,
                    parent,
                    print_only_id,
                } => {
                    // fmt
                    import(files::import::Config {
                        file_path,
                        parents: parent,
                        print_only_id,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Export {
                    file_id,
                    file_path,
                    overwrite,
                } => {
                    let existing_file_action = if overwrite {
                        files::export::ExistingFileAction::Overwrite
                    } else {
                        files::export::ExistingFileAction::Abort
                    };

                    export(files::export::Config {
                        file_id,
                        file_path,
                        existing_file_action,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }
            }
        }

        Command::Permissions { command } => {
            match command {
                PermissionCommand::Share {
                    file_id,
                    role,
                    type_,
                    discoverable,
                    email,
                    domain,
                } => {
                    // fmt
                    permissions::share(permissions::share::Config {
                        file_id,
                        role,
                        type_,
                        discoverable,
                        email,
                        domain,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                PermissionCommand::List {
                    file_id,
                    skip_header,
                    field_separator,
                } => {
                    // fmt
                    permissions::list(permissions::list::Config {
                        file_id,
                        skip_header,
                        field_separator,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                PermissionCommand::Revoke { file_id, all, id } => {
                    let action = if all {
                        permissions::revoke::RevokeAction::AllExceptOwner
                    } else if id.is_some() {
                        permissions::revoke::RevokeAction::Id(id.unwrap_or_default())
                    } else {
                        permissions::revoke::RevokeAction::Anyone
                    };

                    permissions::revoke(permissions::revoke::Config { file_id, action })
                        .await
                        .unwrap_or_else(handle_error)
                }
            }
        }

        Command::Push {
            file_path,
            remote_path,
            mime,
            recursive,
            overwrite,
        } => {
            // Get hub for path resolution
            let hub = hub_helper::get_hub().await.unwrap_or_else(|e| {
                eprintln!("Error getting hub: {}", e);
                std::process::exit(1);
            });

            let rp = std::path::PathBuf::from(&remote_path);

            // Determine if a filename was specified (remote_path does not end with '/')
            let (dir_path, desired_name): (String, Option<String>) = if remote_path.ends_with('/') {
                (remote_path.clone(), None)
            } else if let Some(parent) = rp.parent() {
                let parent_str = if parent.as_os_str().is_empty() { "/".to_string() } else { parent.to_string_lossy().to_string() };
                let fname = rp.file_name().map(|s| s.to_string_lossy().to_string());
                (parent_str, fname)
            } else {
                // No parent component, treat as name in root
                ("/".to_string(), Some(remote_path.clone()))
            };

            println!("Resolving remote directory: {} (name: {:?})", dir_path, desired_name);

            // Resolve or create the destination directory
            let remote_dir = files::path_utils::resolve_or_create_path(&hub, &dir_path)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("Error resolving remote path: {}", e);
                    std::process::exit(1);
                });

            if !crate::common::drive_file::is_directory(&remote_dir) {
                eprintln!("Error: '{}' exists but is not a directory", dir_path);
                std::process::exit(1);
            }

            let folder_id = remote_dir.id.unwrap_or_else(|| {
                eprintln!("Error: Folder ID missing");
                std::process::exit(1);
            });

            // If a destination filename is provided and we're uploading a single file,
            // copy to a temp file with the desired name to control the uploaded name.
            let adjusted_file_path = if desired_name.is_some() && !file_path.is_dir() {
                let name = desired_name.as_ref().unwrap();
                let temp_file_path = std::env::temp_dir().join(name);
                match std::fs::copy(&file_path, &temp_file_path) {
                    Ok(_) => temp_file_path,
                    Err(e) => {
                        eprintln!("Error creating temporary file '{}': {}", temp_file_path.display(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                file_path.clone()
            };

            let config = files::upload::Config {
                file_path: adjusted_file_path,
                mime_type: mime.and_then(|m| m.parse::<Mime>().ok()),
                parents: Some(vec![folder_id.clone()]),
                chunk_size: ChunkSize::default(),
                print_chunk_errors: false,
                print_chunk_info: false,
                upload_directories: recursive,
                print_only_id: false,
            };

            println!(
                "Upload config: file_path={}, parent_id={}, recursive={}",
                config.file_path.display(),
                folder_id,
                recursive
            );

            if !overwrite {
                upload_with_overwrite_check(&hub, config)
                    .await
                    .unwrap_or_else(handle_error)
            } else {
                upload(config)
                    .await
                    .unwrap_or_else(handle_error)
            }
        }

        Command::Version => {
            // fmt
            version::version()
        }
    }
}

fn handle_error(err: impl Error) {
    eprintln!("Error: {}", err);
    std::process::exit(1);
}

#[cfg(test)]
mod tests;
