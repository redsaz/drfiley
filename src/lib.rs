use std::io::Result;
use std::path::{Path, PathBuf};
use std::vec::Vec;

pub mod jobs;
pub mod walker;

pub fn stat_all(path: &Path) -> Result<()> {
    eprintln!("Stat all files in {}", path.display());
    let mut sum_dirs = 0;
    let mut sum_files = 0;
    let mut sum_bytes: u64 = 0;
    let mut files: Vec<PathBuf> = Vec::new();
    for i in walker::walk(path)? {
        let i = i?;
        let path = i.path();
        if path.is_dir() {
            sum_dirs += 1;
        } else {
            sum_files += 1;
            let md = path.metadata().expect("Get metadata");
            let file_size = md.len();
            sum_bytes = sum_bytes + file_size;
            files.push(path)
        }
    }
    eprintln!("Total dirs: {sum_dirs}");
    eprintln!("Total files: {sum_files}");
    eprintln!("Total bytes: {sum_bytes}");
    eprintln!("Files: {:?}", files);
    Ok(())
}
