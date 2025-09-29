#[cfg(test)]
mod tests {
        use crate::{Cli, Command, FileCommand};
    use clap::Parser;

    #[test]
    fn upload_non_recursive_default() {
        // gdrive files upload . /tmp/gdrive_test/2 -> recursive should be false by default
        let cli = Cli::try_parse_from([
            "gdrive",
            "files",
            "upload",
            ".",
            "/tmp/gdrive_test/2",
        ])
        .expect("parse failed");

        match cli.command {
            Command::Files { command } => match command {
                FileCommand::Upload {
                    file_path: _,
                    remote_path: _,
                    mime: _,
                    recursive,
                    chunk_size: _,
                    print_chunk_errors: _,
                    print_chunk_info: _,
                    print_only_id: _,
                } => {
                    assert!(!recursive, "default should be non-recursive");
                }
                _ => panic!("unexpected subcommand"),
            },
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn upload_recursive_flag() {
        // gdrive files upload -r . /tmp/gdrive_test/2 -> recursive should be true
        let cli = Cli::try_parse_from([
            "gdrive",
            "files",
            "upload",
            "-r",
            ".",
            "/tmp/gdrive_test/2",
        ])
        .expect("parse failed");

        match cli.command {
            Command::Files { command } => match command {
                FileCommand::Upload {
                    file_path: _,
                    remote_path: _,
                    mime: _,
                    recursive,
                    chunk_size: _,
                    print_chunk_errors: _,
                    print_chunk_info: _,
                    print_only_id: _,
                } => {
                    assert!(recursive, "-r should enable recursive upload");
                }
                _ => panic!("unexpected subcommand"),
            },
            _ => panic!("unexpected command"),
        }
    }
    
    #[test]
    fn push_command_non_recursive_default() {
        // gdrive push . /tmp/gdrive_test/2 -> recursive should be false by default
        let cli = Cli::try_parse_from([
            "gdrive",
            "push",
            ".",
            "/tmp/gdrive_test/2",
        ])
        .expect("parse failed");

        match cli.command {
            Command::Push {
                file_path: _,
                remote_path: _,
                mime: _,
                recursive,
                overwrite: _,
            } => {
                assert!(!recursive, "push should be non-recursive by default");
            }
            _ => panic!("unexpected command"),
        }
    }
    
    #[test]
    fn push_command_recursive_flag() {
        // gdrive push -r . /tmp/gdrive_test/2 -> recursive should be true
        let cli = Cli::try_parse_from([
            "gdrive",
            "push",
            "-r",
            ".",
            "/tmp/gdrive_test/2",
        ])
        .expect("parse failed");

        match cli.command {
            Command::Push {
                file_path: _,
                remote_path: _,
                mime: _,
                recursive,
                overwrite: _,
            } => {
                assert!(recursive, "push -r should enable recursive upload");
            }
            _ => panic!("unexpected command"),
        }
    }
}
