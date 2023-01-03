use std::path::Path;

use pjsh_core::{
    utils::{path_to_string, resolve_path, word_var},
    Context,
};

use crate::Replacement;

/// Completes a path matching a filter.
pub fn complete_paths<F>(prefix: &str, context: &Context, filter: F) -> Vec<Replacement>
where
    F: Fn(&Path) -> bool,
{
    let original_prefix = prefix;
    let mut prefix = prefix.to_string();
    let mut home = None;
    if original_prefix.starts_with("~/") {
        home = word_var(context, "HOME");
        if let Some(home) = home {
            prefix = original_prefix.replacen("~/", &format!("{home}/"), 1)
        }
    }

    if let Some((dir, file_prefix)) = prefix.rsplit_once('/') {
        let Ok(files) = std::fs::read_dir(resolve_path(context, dir)) else {
            return Vec::default();
        };

        return files
            .into_iter()
            .filter_map(|file| file.ok().map(|f| f.path()))
            .filter(|path| filter(path))
            .filter_map(|path| {
                let file_name = filtered_file_name(path, file_prefix)?;
                let mut content = format!("{dir}/{}", file_name);

                if original_prefix.starts_with("~/") {
                    let home = home.expect("home should be defined");
                    content = content.replacen(home, "~", 1);
                }

                Some(Replacement::new(content, file_name))
            })
            .collect();
    }

    let Some(Ok(files)) = word_var(context, "PWD").map(std::fs::read_dir) else {
        return Vec::default();
    };

    files
        .into_iter()
        .filter_map(|file| file.ok().map(|f| f.path()))
        .filter(|path| filter(path))
        .filter_map(|path| filtered_file_name(path, &prefix))
        .map(Replacement::from)
        .collect()
}

/// Returns a filtered file name.
fn filtered_file_name<P: AsRef<Path>>(path: P, name_prefix: &str) -> Option<String> {
    let path = path.as_ref();
    let path_str = path_to_string(path);
    let (_, file_str) = path_str.rsplit_once('/')?;

    if !file_str.starts_with(name_prefix) {
        return None;
    }

    let mut file_name = file_str.to_owned();

    // Distinguish directories from regular files by adding a trailing slash.
    // This character will also be completed, resulting in faster navigation.
    if path.is_dir() {
        file_name += "/";
    }

    Some(file_name)
}
