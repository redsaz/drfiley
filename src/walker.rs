use std::fs;
use std::io::Result;
use std::collections::VecDeque;

use std::path::{Path, PathBuf};

/// Iterator over the entries in a directory and sub-directories.
///
/// This iterator is returned from the [`walk`] function of this module and
/// will yield instances of <code>[io::Result]<[DirEntry]></code>. The order
/// of entries returned by the iterator is currently undefined.
///
/// # Errors
///
/// This [`io::Result`] will be an [`Err`] if some IO error occurs during
/// iteration.
#[derive(Debug)]
pub struct Walker {
    base: PathBuf,
}

impl IntoIterator for Walker {
    type Item = Result<fs::DirEntry>;
    type IntoIter = WalkerIntoIterator;

    fn into_iter(self) -> WalkerIntoIterator {
        let path = self.base.as_path();
        WalkerIntoIterator {
            current_iter: fs::read_dir(path),
            remaining_dirs: VecDeque::new(),
        }
    }
}

pub struct WalkerIntoIterator {
    current_iter: Result<fs::ReadDir>,
    remaining_dirs: VecDeque<PathBuf>,
}

impl Iterator for WalkerIntoIterator {
    type Item = Result<fs::DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        /*
        1. Work through the current iterator.
        2. Once it is complete, replace it with the first iterator from the
           remaining iterators list, if any.
        3. Any directory encountered will add an iterator to the back of the
           list.
        4. Whether or not the item from the current iterator is a file or dir,
           it will be returned.
        5. Once the list of remaining iterators is complete, the walker is
           done.
        */
        match &mut self.current_iter {
            Ok(iter) => {
                let mut opt_item = iter.next();

                // If there are no more entries in the dir:
                // then iterate over the next dir in the list.
                // BUT, if that dir is empty, go to the next dir, and so on,
                // until there's either a dir with an entry or there are no
                // more dirs.
                while let None = opt_item {
                    if self.remaining_dirs.is_empty() {
                        break;
                    }
                    let opt_path = self.remaining_dirs.pop_front();
                    opt_item = match opt_path {
                        Some(path) => {
                            self.current_iter = fs::read_dir(path);
                            match &mut self.current_iter {
                                Ok(iter2) => iter2.next(),
                                Err(error) => Some(Err(std::io::Error::new(error.kind(), "Bah"))),
                            }
                        }
                        None => None,
                    };
                }
                match &opt_item {
                    Some(item) => {
                        if let Ok(w) = item {
                            let path = w.path();
                            if path.is_dir() {
                                // TODO: Consider VecDeque
                                self.remaining_dirs.push_back(path);
                            }
                        }
                    }
                    None => {
                        let opt_path = self.remaining_dirs.pop_front();
                        opt_item = match opt_path {
                            Some(path) => {
                                self.current_iter = fs::read_dir(path);
                                self.next()
                            }
                            None => None,
                        };
                    }
                }
                opt_item
            }
            Err(error) => Some(Err(std::io::Error::new(error.kind(), "Bah"))),
        }
    }
}

pub fn walk_sum(path: &Path) -> Result<()> {
    let mut sum_dirs = 0;
    let mut sum_files = 0;
    for i in walk(path)? {
        let i = i?;
        let path = i.path();
        if path.is_dir() {
            sum_dirs += 1;
            // eprintln!("Found a dir: {}", path.display());
            // walk(path.as_path())?;
        } else {
            sum_files += 1;
            // eprintln!("Found a file maybe: {}", path.display());
        }
    }
    eprintln!("Total dirs: {sum_dirs}");
    eprintln!("Total files: {sum_files}");
    Ok(())
}

fn walk(path: &Path) -> Result<Walker> {
    Ok(Walker { base: path.to_path_buf()})
}
