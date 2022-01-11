use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

use dirs::home_dir;

use super::Complete;

const PATH_SEPARATOR: char = '/';

pub struct FileCompleter;
impl Complete for FileCompleter {
    fn complete(&self, input: &str) -> Vec<String> {
        let (dir_name, file_name) = match input.rfind(PATH_SEPARATOR) {
            Some(idx) => input.split_at(idx + PATH_SEPARATOR.len_utf8()),
            None => ("", input),
        };

        let path = base_path(dir_name);
        files_in_path(path, dir_name, file_name)
    }
}

fn base_path(directory: impl AsRef<Path>) -> PathBuf {
    let dir_path = directory.as_ref();
    if dir_path.starts_with("~") {
        // The path is relative to the user's home directory.
        if let Some(home) = home_dir() {
            match dir_path.strip_prefix("~") {
                Ok(rel_path) => home.join(rel_path),
                _ => home,
            }
        } else {
            // The user does not have a known home directory. The path is not likely to work,
            // but the autocomplete function should not throw any errors.
            dir_path.to_path_buf()
        }
    } else if dir_path.is_relative() {
        // The path is relative to the current directory.
        if let Ok(working_directory) = current_dir() {
            working_directory.join(dir_path)
        } else {
            // The current directory cannot be accessed. The path is not likely to work, but the
            // autocomplete function should not throw any errors.
            dir_path.to_path_buf()
        }
    } else {
        // The path is absolute.
        dir_path.to_path_buf()
    }
}

fn files_in_path(path: PathBuf, dir_name: &str, file_name: &str) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }

    let mut files: Vec<String> = Vec::new();

    if let Ok(read_dir) = path.read_dir() {
        for entry in read_dir.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(file_name) {
                    if let Ok(metadata) = fs::metadata(entry.path()) {
                        let mut path = String::from(dir_name) + name;

                        if metadata.is_dir() {
                            path.push(PATH_SEPARATOR);
                        }

                        files.push(path);
                    }
                }
            }
        }
    }

    files
}
