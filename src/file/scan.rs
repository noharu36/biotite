use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn is_markdown_file(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .map_or(false, |ext| ext == "md" || ext == "markdown")
}

pub fn scan_dir(dir: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| {
            e.ok().and_then(|entry| {
                let path = entry.path();
                if is_markdown_file(path) {
                    Some(path.to_path_buf())
                } else {
                    None
                }
            })
        })
        .collect()
}
