use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// Minimal CLI surface tests. These ensure the new UX shape exists.
// Behavior/integration with Google Drive is covered by ignored tests below.

#[ignore]
#[test]
fn help_lists_new_top_level_commands() {
    let mut cmd = Command::cargo_bin("gdrive").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tree").or(predicate::str::contains("Tree")))
        .stdout(predicate::str::contains("pull").or(predicate::str::contains("Download")))
        .stdout(predicate::str::contains("push").or(predicate::str::contains("Upload")))
        .stdout(predicate::str::contains("mkdir").or(predicate::str::contains("Create directory")))
        .stdout(predicate::str::contains("rm").or(predicate::str::contains("Delete")));
}

#[ignore]
#[test]
fn tree_help_shows_date_and_size() {
    let mut cmd = Command::cargo_bin("gdrive").unwrap();
    cmd.args(["tree", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("date"))
        .stdout(predicate::str::contains("size"));
}

#[ignore]
#[test]
fn pull_help_shows_flags() {
    let mut cmd = Command::cargo_bin("gdrive").unwrap();
    cmd.args(["pull", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("-r")) // recursive
        .stdout(predicate::str::contains("-f")) // files only in dir
        .stdout(predicate::str::contains("--overwrite").or(predicate::str::contains("overwrite")));
}

#[ignore]
#[test]
fn push_help_shows_flags() {
    let mut cmd = Command::cargo_bin("gdrive").unwrap();
    cmd.args(["push", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("-r")) // recursive
        .stdout(predicate::str::contains("--mime").or(predicate::str::contains("MIME")))
        .stdout(predicate::str::contains("glob").or(predicate::str::contains("pattern")));
}

#[ignore]
#[test]
fn mkdir_help_mentions_remote_path() {
    let mut cmd = Command::cargo_bin("gdrive").unwrap();
    cmd.args(["mkdir", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("remote path").or(predicate::str::contains("/tmp/gdrive_test")));
}

#[ignore]
#[test]
fn pull_non_recursive_downloads_only_files_in_dir() {
    // Example: gdrive pull /tmp/gdrive_test ~/tmp/gdrive-tests -> downloads f0.zip only
    // Behavior spec only.
}

#[ignore]
#[test]
fn pull_recursive_downloads_subdirs() {
    // Example: gdrive pull -r /tmp/gdrive_test ~/tmp/gdrive-tests
}

#[ignore]
#[test]
fn pull_dash_f_downloads_all_files_in_dir_without_subdirs() {
    // Example: gdrive pull -f /tmp/gdrive_test/
}

#[ignore]
#[test]
fn push_non_recursive_default() {
    // Example: gdrive push . /tmp/gdrive_test/2 -> no subdirs
}

#[ignore]
#[test]
fn push_recursive_with_r() {
    // Example: gdrive push -r . /tmp/gdrive_test/2
}

#[ignore]
#[test]
fn push_glob_patterns() {
    // Example: gdrive push /some/dir/*.jpg /tmp/gdrive_test/2
}

#[ignore]
#[test]
fn mkdir_creates_nested_path() {
    // Example: gdrive mkdir /tmp/gdrive_test/3
}

#[ignore]
#[test]
fn rm_removes_file_or_directory_recursively() {
    // Example: gdrive rm -r /some/dir
}

// Overwrite behavior specs (ignored): default is no overwrite; with --overwrite skip prompt
#[ignore]
#[test]
fn pull_prompts_and_lists_files_when_overwrite_not_set() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let existing = temp.child("f0.zip");
    existing.write_str("dummy").unwrap();

    let mut cmd = Command::cargo_bin("gdrive").unwrap();
    cmd.args(["pull", "/tmp/gdrive_test", temp.path().to_string_lossy().as_ref()]);

    cmd.assert()
        .stdout(predicate::str::contains("Will overwrite:"))
        .stdout(predicate::str::contains("f0.zip"))
        .stdout(predicate::str::contains("Overwrite 1 file"))
        .stdout(predicate::str::contains("[y/N]"));
}

#[ignore]
#[test]
fn pull_with_overwrite_flag_skips_prompt_and_overwrites() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let existing = temp.child("f0.zip");
    existing.write_str("dummy").unwrap();

    let mut cmd = Command::cargo_bin("gdrive").unwrap();
    cmd.args(["pull", "--overwrite", "/tmp/gdrive_test", temp.path().to_string_lossy().as_ref()]);

    cmd.assert()
        .stderr(predicate::str::contains("Error").not())
        .stdout(predicate::str::contains("Overwriting:").or(predicate::str::contains("Downloading")));
}

#[ignore]
#[test]
fn push_prompts_and_lists_files_when_overwrite_not_set() {
    let mut cmd = Command::cargo_bin("gdrive").unwrap();
    cmd.args(["push", "f0.zip", "/tmp/gdrive_test/2"]);
    cmd.assert()
        .stdout(predicate::str::contains("Will overwrite:"))
        .stdout(predicate::str::contains("f0.zip"))
        .stdout(predicate::str::contains("Overwrite 1 file"))
        .stdout(predicate::str::contains("[y/N]"));
}

#[ignore]
#[test]
fn push_with_overwrite_flag_skips_prompt_and_overwrites() {
    let mut cmd = Command::cargo_bin("gdrive").unwrap();
    cmd.args(["push", "--overwrite", "f0.zip", "/tmp/gdrive_test/2"]);
    cmd.assert()
        .stderr(predicate::str::contains("Error").not())
        .stdout(predicate::str::contains("Overwriting:").or(predicate::str::contains("Uploading")));
}
