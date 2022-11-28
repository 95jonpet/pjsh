use std::path::Path;

use pjsh_core::{
    utils::{path_to_string, resolve_path},
    Context,
};

/// Completes a path.
pub fn complete_paths<F>(prefix: &str, context: &Context, filter: F) -> Vec<String>
where
    F: Fn(&Path) -> bool,
{
    if let Some((dir, file_prefix)) = prefix.rsplit_once('/') {
        let Ok(files) = std::fs::read_dir(resolve_path(context, dir)) else {
            return Vec::default();
        };

        return files
            .into_iter()
            .filter_map(|file| file.ok().map(|f| f.path()))
            .filter(|path| filter(path))
            .filter_map(|path| Some(format!("{dir}/{}", filtered_file_name(path, file_prefix)?)))
            .collect();
    }

    let Ok(Ok(files)) = std::env::current_dir().map(std::fs::read_dir) else {
        return Vec::default();
    };

    files
        .into_iter()
        .filter_map(|file| file.ok().map(|f| f.path()))
        .filter(|path| filter(path))
        .filter_map(|path| filtered_file_name(path, prefix))
        .collect()
}

/// Returns a filtered file name.
fn filtered_file_name<P: AsRef<Path>>(path: P, name_prefix: &str) -> Option<String> {
    let path_str = path_to_string(path);
    let (_, file_str) = path_str.rsplit_once('/')?;

    if !file_str.starts_with(name_prefix) {
        return None;
    }

    Some(file_str.to_owned())
}
