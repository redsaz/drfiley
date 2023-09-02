use crate::walker;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Job to collect files and directories of a given directory.
pub struct ScanJob {
    path: PathBuf,
}

#[derive(Debug)]
pub struct ScanItem {
    pub path: PathBuf,
    pub size_bytes: u64,
}

#[derive(Debug)]
enum PathNode {
    Dir {
        size_bytes: u64,
        items: BTreeMap<PathBuf, PathNode>,
    },
    File {
        size_bytes: u64,
    },
}

impl ScanJob {
    pub fn new(path: &Path) -> ScanJob {
        ScanJob {
            path: path.to_path_buf(),
        }
    }

    pub fn run(&self) -> Vec<ScanItem> {
        let files = Self::scan(self);
        let tree = Self::treeify(&files);

        eprintln!("Tree: {:?}", tree);
        files
    }

    fn scan(&self) -> Vec<ScanItem> {
        // let mut size_map: BTreeMap<u64, Vec<PathBuf>> = BTreeMap::new();
        eprintln!("Stat all files in {}", self.path.display());
        let mut sum_dirs = 0;
        let mut sum_files = 0;
        let mut files: Vec<ScanItem> = Vec::new();
        for i in walker::walk(self.path.as_path()).expect("Couldn't walk path.") {
            let i = i.expect("TODO");
            let path = i.path();
            if path.is_dir() {
                sum_dirs += 1;
            } else {
                sum_files += 1;
                let md = path.metadata().expect("Failed to fetch metadata.");
                let file_size = md.len();
                files.push(ScanItem {
                    path: path,
                    size_bytes: file_size,
                });
            }
        }
        eprintln!("Total dirs: {sum_dirs}");
        eprintln!("Total files: {sum_files}");
        files
    }

    fn treeify(files: &Vec<ScanItem>) -> PathNode {
        // Start the tree with an empty dir
        let mut tree = PathNode::Dir {
            size_bytes: 0,
            items: BTreeMap::new(),
        };

        // for each file,
        //     for each dir up to the filename,
        //         ensure the dir has an entry in the tree
        //     place the filename in the filelist of the last dir
        for file in files {
            let mut parent = &mut tree;

            // Ensure all parent path parts of the file are initialized in the tree, and advance the parent
            for dir_part in file
                .path
                .parent()
                .expect("Path ended in a root or prefix, or was empty.")
                .components()
            {
                let dir_part_name = PathBuf::from(dir_part.as_os_str());
                match parent {
                    PathNode::Dir { size_bytes,  items } => {*size_bytes = *size_bytes + file.size_bytes; parent = items
                        .entry(dir_part_name.to_owned())
                        .or_insert_with(|| PathNode::Dir{items: BTreeMap::new(), size_bytes: 0})},
                    PathNode::File {size_bytes: _} => panic!("Cannot treeify {file:?} because its parent directory is somehow a file instead."),
                };
            }
            // Insert the file into the tree
            let file_name =
                PathBuf::from(file.path.file_name().expect("Should have filename, not .."));
            match parent {
                PathNode::Dir { size_bytes: _, items} => items.entry(file_name.to_owned()).or_insert_with(|| PathNode::File{size_bytes: file.size_bytes}),
                PathNode::File {size_bytes: _} => panic!("Cannot treeify {file:?} because its parent directory is somehow a file instead."),
            };
        }

        tree
    }
}
