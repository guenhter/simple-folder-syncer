#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::{
    fs::{self, File},
    io::Write,
    os::windows::{fs::MetadataExt, process::CommandExt},
    path::{self, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use windows_sys::Win32::{
    Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN, System::Threading::CREATE_NO_WINDOW,
};

const DEFAULT_CONFIG_FILE_NAME: &str = "folder_sync_config.yaml";

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Configuration {
    source: String,
    target: String,
    create_last_sync_result_file: bool,
    exclude_root_source_hidden_entries: bool,
    exclude_paths: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let home_dir = find_config_path().unwrap();
    run_folder_sync(&home_dir)?;

    Ok(())
}

fn run_folder_sync(config_path: &Path) -> anyhow::Result<()> {
    let config = read_config(config_path)?;

    let output = run_folder_sync_with_config(config.clone())?;

    if config.create_last_sync_result_file {
        let target = path::absolute(PathBuf::from(config.target))?;
        write_folder_sync_result(&target, &output)?;
    }

    Ok(())
}

fn run_folder_sync_with_config(config: Configuration) -> anyhow::Result<String> {
    let source = path::absolute(config.source)?;
    let target = path::absolute(config.target)?;

    let additional_excludes: Vec<PathBuf> = config
        .exclude_paths
        .into_iter()
        .map(path::absolute)
        .collect::<Result<Vec<PathBuf>, _>>()?;

    let all_exclude_paths = collect_exclude_paths(
        &source,
        config.exclude_root_source_hidden_entries,
        additional_excludes,
    )?;
    let mut exclude_args = build_robocopy_exclude_arguments(&all_exclude_paths)?;

    let mut args = vec![
        source.display().to_string(),
        target.display().to_string(),
        "/mir".to_string(),
        "/z".to_string(),
        "/r:1".to_string(),
        "/w:1".to_string(),
        "/sl".to_string(),
        r"/unilog:C:\temp\simple_folder_sync_robocopy.log".to_string(),
    ];
    args.append(&mut exclude_args);

    let output = std::process::Command::new("robocopy")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;

    remove_excluded_files_and_folders_in_target(&source, &target, all_exclude_paths)?;

    let full_output = format!(
        "=== STDOUT ===\n{}\n=== STDERR ===\n{}",
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?
    );

    Ok(full_output)
}

fn find_config_path() -> Option<PathBuf> {
    match home::home_dir() {
        Some(path) => Some(path.join(DEFAULT_CONFIG_FILE_NAME)),
        None => None,
    }
}

fn read_config(config_path: &Path) -> anyhow::Result<Configuration> {
    let config_file = fs::File::open(config_path)?;
    let config = serde_yaml::from_reader(config_file)?;
    Ok(config)
}

fn write_folder_sync_result(folder_sync_target: &Path, content: &str) -> anyhow::Result<()> {
    let mut file = File::create(folder_sync_target.join("last-sync-result.txt"))?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

fn collect_exclude_paths(
    source: &Path,
    exclude_root_source_hidden_entries: bool,
    exclude_paths: Vec<PathBuf>,
) -> anyhow::Result<Vec<PathBuf>> {
    let hidden_entries: Vec<PathBuf> = if exclude_root_source_hidden_entries {
        list_dir_entries(source)?
            .into_iter()
            .filter(|e| is_hidden(&e.path()).unwrap_or(false))
            .map(|e| e.path())
            .collect()
    } else {
        vec![]
    };

    let mut all_entries = hidden_entries;
    let mut exclude_paths = exclude_paths;
    all_entries.append(&mut exclude_paths);

    Ok(all_entries)
}

fn build_robocopy_exclude_arguments(exclude_paths: &Vec<PathBuf>) -> anyhow::Result<Vec<String>> {
    let mut args = vec![];

    let exclude_file_paths: Vec<String> = exclude_paths
        .iter()
        .filter(|e| e.is_file())
        .map(|e| e.display().to_string())
        .collect();
    let exclude_folder_paths: Vec<String> = exclude_paths
        .iter()
        .filter(|e| e.is_dir())
        .map(|e| e.display().to_string())
        .collect();

    if !exclude_file_paths.is_empty() {
        args.push("/XF".to_string());
        args.append(&mut exclude_file_paths.clone());
    }

    if !exclude_folder_paths.is_empty() {
        args.push("/XD".to_string());
        args.append(&mut exclude_folder_paths.clone());
    }

    Ok(args)
}

fn list_dir_entries(dir: &Path) -> anyhow::Result<Vec<fs::DirEntry>> {
    let paths = fs::read_dir(dir)?
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();
    Ok(paths)
}

fn is_hidden(dir_entry: &Path) -> std::io::Result<bool> {
    let metadata = fs::metadata(dir_entry)?;
    let attributes = metadata.file_attributes();

    Ok(attributes & FILE_ATTRIBUTE_HIDDEN > 0)
}

fn remove_excluded_files_and_folders_in_target(
    source: &Path,
    target: &Path,
    paths_to_delete: Vec<PathBuf>,
) -> anyhow::Result<()> {
    let source = path::absolute(source)?;
    let target = path::absolute(target)?;

    let paths_to_delete: Vec<PathBuf> = replace_root_path(&source, &target, &paths_to_delete)?
        .into_iter()
        .map(|e| path::absolute(e))
        .collect::<Result<Vec<PathBuf>, _>>()?;

    // This check is actually unnecessary because logically it cannot happen due to `replace_root_path`
    // but I anyway make it as a final safety net, in case something changes in the future, so this is
    // the last gate which must be passed.
    if paths_to_delete.iter().any(|e| !e.starts_with(&target)) {
        return Err(anyhow::Error::msg(
            "Some paths to delete are not prefixed with the target path",
        ));
    }

    let folders_to_delete = paths_to_delete.iter().filter(|e| e.is_dir());
    let files_to_delete = paths_to_delete.iter().filter(|e| e.is_file());

    // Perform the actual deletion
    for entry in folders_to_delete {
        println!(" -- Removing {}", entry.display());

        match remove_dir_all_alternative(entry) {
            Ok(_) => {}
            Err(e) => {
                println!("Error deleting folder {}", entry.display());
                return Err(e.into());
            }
        }
    }
    for entry in files_to_delete {
        fs::remove_file(entry)?;
    }

    Ok(())
}

fn replace_root_path(
    current_root: &Path,
    new_root: &Path,
    paths: &Vec<PathBuf>,
) -> anyhow::Result<Vec<PathBuf>> {
    paths
        .iter()
        .map(|path| {
            let relative_path = path.strip_prefix(current_root)?;
            Ok(new_root.join(relative_path))
        })
        .collect()
}

// https://github.com/rust-lang/rust/issues/126576
// https://github.com/winfsp/winfsp/issues/561
fn remove_dir_all_alternative(path: &Path) -> anyhow::Result<()> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();

        if path.is_dir() {
            remove_dir_all_alternative(&path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    fs::remove_dir(path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::fs;

    use anyhow::Ok;
    use assertor::{assert_that, BooleanAssertion, EqualityAssertion, VecAssertion};
    use tempfile::tempdir;
    use walkdir::WalkDir;
    use windows_sys::Win32::Storage::FileSystem::SetFileAttributesA;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_run_without_special_parameters() {
        let temp_dir = tempdir().expect("Failed to create a temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        let source_dir_path: PathBuf = temp_dir.path().join("source");
        let target_dir_path: PathBuf = temp_dir.path().join("target");

        store_config(
            &config_path,
            &Configuration {
                source: source_dir_path.display().to_string(),
                target: target_dir_path.display().to_string(),
                create_last_sync_result_file: false,
                exclude_root_source_hidden_entries: false,
                exclude_paths: vec![],
            },
        )
        .unwrap();
        prepare_test_folder(&source_dir_path).unwrap();

        // Actual call under test
        {
            run_folder_sync(&config_path).unwrap();
        }

        // Assertions
        {
            let target_dir_hierarchy = list_files_and_folders(&target_dir_path).unwrap();
            let expected_dir_hierarchy = vec![
                "file1.txt".to_string(),
                "hidden-file1.txt".to_string(),
                "some-folder".to_string(),
                "some-folder\\file2.txt".to_string(),
                "some-folder\\hidden-file2.txt".to_string(),
                "some-hidden-folder".to_string(),
                "some-hidden-folder\\file3.txt".to_string(),
            ];
            assert_that!(target_dir_hierarchy).contains_exactly(expected_dir_hierarchy);
        }
    }

    #[test]
    fn test_run_with_hidden_files_in_root_excluded() {
        let temp_dir = tempdir().expect("Failed to create a temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        let source_dir_path: PathBuf = temp_dir.path().join("source");
        let target_dir_path: PathBuf = temp_dir.path().join("target");

        store_config(
            &config_path,
            &Configuration {
                source: source_dir_path.display().to_string(),
                target: target_dir_path.display().to_string(),
                create_last_sync_result_file: false,
                exclude_root_source_hidden_entries: true,
                exclude_paths: vec![],
            },
        )
        .unwrap();
        prepare_test_folder(&source_dir_path).unwrap();

        // Actual call under test
        {
            run_folder_sync(&config_path).unwrap();
        }

        // Assertions
        {
            let target_dir_hierarchy = list_files_and_folders(&target_dir_path).unwrap();
            let expected_dir_hierarchy = vec![
                "file1.txt".to_string(),
                "some-folder".to_string(),
                "some-folder\\file2.txt".to_string(),
                "some-folder\\hidden-file2.txt".to_string(),
            ];
            assert_that!(target_dir_hierarchy).contains_exactly(expected_dir_hierarchy);
        }
    }

    #[test]
    fn test_create_exclude_paths() {
        let temp_dir = tempdir().expect("Failed to create a temp dir");
        let source_dir_path: PathBuf = temp_dir.path().join("source");

        prepare_test_folder(&source_dir_path).unwrap();

        let exclude_paths = collect_exclude_paths(
            &source_dir_path,
            true,
            vec![source_dir_path.join("foobar.txt")],
        )
        .unwrap();

        assert_that!(exclude_paths).contains_exactly(vec![
            source_dir_path.join("hidden-file1.txt"),
            source_dir_path.join("some-hidden-folder"),
            source_dir_path.join("foobar.txt"),
        ]);
    }

    #[test]
    fn test_create_excluce_paths_nothing_excluded() {
        let exclude_paths = collect_exclude_paths(Path::new("/tmp/foo"), false, vec![]).unwrap();

        assert_that!(exclude_paths).is_empty();
    }

    #[test]
    fn test_folder_sync_result_file_written_to_target() {
        let temp_dir = tempdir().expect("Failed to create a temp dir");
        let config_path = temp_dir.path().join("config.yaml");
        let source_dir_path = temp_dir.path().join("source");
        let target_dir_path = temp_dir.path().join("target");

        store_config(
            &config_path,
            &Configuration {
                source: source_dir_path.display().to_string(),
                target: target_dir_path.display().to_string(),
                create_last_sync_result_file: true,
                exclude_root_source_hidden_entries: false,
                exclude_paths: vec![],
            },
        )
        .unwrap();
        prepare_test_folder(&source_dir_path).unwrap();

        // Actual call under test
        {
            run_folder_sync(&config_path).unwrap();
        }

        // Assertions
        {
            let folder_sync_result_file = target_dir_path.join("last-sync-result.txt");
            assert_that!(folder_sync_result_file.exists()).is_true();
        }
    }

    #[test]
    fn test_excluded_files_or_folders_get_deleted_on_target() {
        let temp_dir = tempdir().expect("Failed to create a temp dir");
        let source_dir_path = temp_dir.path().join("source");
        let target_dir_path = temp_dir.path().join("target");

        let config = Configuration {
            source: source_dir_path.display().to_string(),
            target: target_dir_path.display().to_string(),
            create_last_sync_result_file: false,
            exclude_root_source_hidden_entries: false,
            exclude_paths: vec![],
        };
        prepare_test_folder(&source_dir_path).unwrap();

        // Actual call under test
        {
            run_folder_sync_with_config(config.clone()).unwrap();

            // Run again, but this time exclude hidden files
            let config = Configuration {
                exclude_root_source_hidden_entries: true,
                ..config
            };
            run_folder_sync_with_config(config).unwrap();
        }

        // Assertions
        {
            let target_dir_hierarchy = list_files_and_folders(&target_dir_path).unwrap();
            let source_dir_hierarchy = list_files_and_folders(&source_dir_path).unwrap();
            let expected_target_dir_hierarchy = vec![
                "file1.txt".to_string(),
                "some-folder".to_string(),
                "some-folder\\file2.txt".to_string(),
                "some-folder\\hidden-file2.txt".to_string(),
            ];
            let expected_source_dir_hierarchy = vec![
                "file1.txt".to_string(),
                "hidden-file1.txt".to_string(),
                "some-folder".to_string(),
                "some-folder\\file2.txt".to_string(),
                "some-folder\\hidden-file2.txt".to_string(),
                "some-hidden-folder".to_string(),
                "some-hidden-folder\\file3.txt".to_string(),
            ];
            assert_that!(target_dir_hierarchy).contains_exactly(expected_target_dir_hierarchy);
            assert_that!(source_dir_hierarchy).contains_exactly(expected_source_dir_hierarchy);
        }
    }

    #[test]
    fn test_read_config() {
        let temp_dir = tempdir().expect("Failed to create a temp dir");
        let config_path = temp_dir.path().join("config.yaml");

        let config = Configuration {
            source: r"C:\temp\source\".to_string(),
            target: r"C:\temp\target".to_string(),
            create_last_sync_result_file: true,
            exclude_root_source_hidden_entries: true,
            exclude_paths: vec![r"C:/temp/source/some-folder/".to_string()],
        };

        store_config(&config_path, &config).unwrap();

        let read_config = read_config(&config_path).unwrap();

        assert_that!(read_config).is_equal_to(config);
    }

    #[test]
    fn test_remove_all_files_and_folders_in_target() {
        let temp_dir = tempdir().expect("Failed to create a temp dir");
        let temp_dir = temp_dir.path();

        fs::File::create(temp_dir.join("f1.txt")).unwrap();
        fs::File::create(temp_dir.join("f2.txt")).unwrap();
        fs::create_dir_all(temp_dir.join("stay")).unwrap();
        fs::File::create(temp_dir.join("stay").join("f3.txt")).unwrap();

        fs::create_dir_all(temp_dir.join("tbr").join("foo").join("bar")).unwrap();
        fs::File::create(temp_dir.join("tbr").join("f4.txt")).unwrap();
        fs::File::create(temp_dir.join("tbr").join("foo").join("f5.txt")).unwrap();
        fs::File::create(temp_dir.join("tbr").join("foo").join("bar").join("f6.txt")).unwrap();

        let paths_to_remove = vec![temp_dir.join("f2.txt"), temp_dir.join("tbr")];

        // Function under test
        {
            remove_excluded_files_and_folders_in_target(&temp_dir, &temp_dir, paths_to_remove)
                .unwrap();
        }

        // Assertions
        {
            let actual_dir_hierarchy = list_files_and_folders(&temp_dir).unwrap();
            let expected_dir_hierarchy = vec![
                "f1.txt".to_string(),
                "stay".to_string(),
                "stay\\f3.txt".to_string(),
            ];
            assert_that!(actual_dir_hierarchy).contains_exactly(expected_dir_hierarchy);
        }
    }

    #[test]
    fn test_replace_root_path() {
        let current_root = Path::new("/tmp/foo");
        let paths_to_replace = vec![
            PathBuf::from("/tmp/foo/file1.txt"),
            PathBuf::from("/tmp/foo/subdir/file2.txt"),
        ];
        let new_root = Path::new("/foo/bar");

        let updated_paths = replace_root_path(current_root, new_root, &paths_to_replace).unwrap();

        assert_that!(updated_paths.len()).is_equal_to(2);
        assert_that!(updated_paths[0]).is_equal_to(PathBuf::from("/foo/bar/file1.txt"));
        assert_that!(updated_paths[1]).is_equal_to(PathBuf::from("/foo/bar/subdir/file2.txt"));
    }

    fn store_config(config_path: &PathBuf, test_config: &Configuration) -> anyhow::Result<()> {
        let config_file = fs::File::create(config_path)?;
        serde_yaml::to_writer(config_file, test_config)?;
        Ok(())
    }

    fn list_files_and_folders(dir: &Path) -> anyhow::Result<Vec<String>> {
        println!("== List {}", dir.display());

        WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .filter(|e| *e != dir)
            .map(|path| {
                let relative_path = path.strip_prefix(dir)?;
                Ok(relative_path.display().to_string())
            })
            .collect()
    }

    fn prepare_test_folder(temp_dir: &Path) -> anyhow::Result<()> {
        // |
        // - some-folder
        //     - file2.txt
        //     - hidden-file2.txt
        // - some-hidden-folder
        //     - file3.txt
        // - file1.txt
        // - hidden-file1.txt

        fs::create_dir_all(temp_dir)?;

        let vis_folder = temp_dir.join("some-folder");
        let hidden_folder = temp_dir.join("some-hidden-folder");

        create_folder(&vis_folder, false).unwrap();
        create_folder(&hidden_folder, true).unwrap();

        create_file(&temp_dir.join("file1.txt"), false).unwrap();
        create_file(&temp_dir.join("hidden-file1.txt"), true).unwrap();

        create_file(&vis_folder.join("file2.txt"), false).unwrap();
        create_file(&vis_folder.join("hidden-file2.txt"), true).unwrap();

        create_file(&hidden_folder.join("file3.txt"), true).unwrap();

        Ok(())
    }

    fn create_folder(path: &Path, hidden: bool) -> anyhow::Result<()> {
        fs::create_dir_all(path)?;

        if hidden {
            add_file_attributes(path, FILE_ATTRIBUTE_HIDDEN)?;
        }

        Ok(())
    }

    fn create_file(path: &Path, hidden: bool) -> anyhow::Result<()> {
        fs::File::create(path)?;

        if hidden {
            add_file_attributes(path, FILE_ATTRIBUTE_HIDDEN)?;
        }

        Ok(())
    }

    fn add_file_attributes(path: &Path, new_attributes: u32) -> anyhow::Result<()> {
        let metadata = fs::metadata(path)?;
        let existing_attributes = metadata.file_attributes();

        let c_str = CString::new(path.display().to_string())?;

        unsafe {
            SetFileAttributesA(
                c_str.as_ptr() as *const u8,
                existing_attributes | new_attributes,
            );
        }

        Ok(())
    }
}
