use std::io::Result;
use std::path::Path;

pub mod walker;

pub fn stat_all(path: &Path) -> Result<()> {
    eprintln!("Stat all files in {}", path.display());
    walker::walk_sum(path)
}

