use druid::im::Vector;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub(crate) enum FileNode {
    Directory {
        path: PathBuf,
        children: Vector<FileNode>,
    },
    File {
        path: PathBuf,
        size: u64,
    },
}

impl FileNode {
    pub(crate) fn size(&self) -> u64 {
        match self {
            FileNode::Directory { .. } => 0,
            FileNode::File { size, .. } => *size,
        }
    }

    pub(crate) fn path(&self) -> &PathBuf {
        match self {
            FileNode::Directory { path, .. } => path,
            FileNode::File { path, .. } => path,
        }
    }

    pub(crate) fn as_vector(self) -> Vector<FileNode> {
        self.into_iter().collect()
    }
}

pub(crate) struct FileNodeIterator {
    pub(crate) stack: Vec<FileNode>,
}

impl FileNodeIterator {
    pub(crate) fn new(root: FileNode) -> Self {
        FileNodeIterator { stack: vec![root] }
    }
}

impl Iterator for FileNodeIterator {
    type Item = FileNode;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            match node {
                FileNode::Directory { children, .. } => {
                    for child in children.into_iter().rev() {
                        self.stack.push(child);
                    }
                    let child = self.stack.pop();
                    assert!(
                        matches!(child, Some(FileNode::File { .. }) | None),
                        "FileNodeIterator shouldn't return directories"
                    );
                    return child;
                }
                FileNode::File { path, size } => {
                    return Some(FileNode::File { path, size });
                }
            }
        }
        None
    }
}

impl IntoIterator for FileNode {
    type Item = FileNode;
    type IntoIter = FileNodeIterator;

    fn into_iter(self) -> Self::IntoIter {
        FileNodeIterator::new(self)
    }
}

pub(crate) fn traverse_files_parallel(path: &PathBuf) -> Option<FileNode> {
    tracing::debug!("Starting traverse with path `{}`", path.display());
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

    let Ok(metadata) = std::fs::metadata(path) else {
        return None;
    };

    if metadata.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            let paths: Vec<_> = entries
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .collect();

            let children: Vector<FileNode> = paths
                .par_iter()
                .filter_map(|path| {
                    if path.is_file() {
                        let metadata = std::fs::metadata(path).ok();
                        let size = metadata.map(|md| md.len()).unwrap_or(0);

                        Some(FileNode::File {
                            path: path.clone(),
                            size,
                        })
                    } else {
                        traverse_files_parallel(path)
                    }
                })
                .collect::<Vec<_>>()
                .into();

            tracing::debug!(
                "Found directory `{}` with `{}` children",
                path.display(),
                children.len()
            );
            Some(FileNode::Directory {
                path: path.clone(),
                children,
            })
        } else {
            tracing::debug!("Failed traverse with path `{}`", path.display());
            None
        }
    } else if metadata.is_file() {
        tracing::debug!("Found file `{}`", path.display());
        Some(FileNode::File {
            path: path.clone(),
            size: metadata.len(),
        })
    } else {
        tracing::warn!("Failed traverse with path `{}`", path.display());
        None
    }
}

#[cfg(test)]
mod tests {
    use druid::im::vector;

    use crate::file_system::FileNode;

    #[test]
    fn iterator() {
        let root = FileNode::Directory {
            path: "/".into(),
            children: vector![
                FileNode::File {
                    path: "/1".into(),
                    size: 1
                },
                FileNode::Directory {
                    path: "/2".into(),
                    children: vector![FileNode::File {
                        path: "/2/3".into(),
                        size: 3,
                    }]
                }
            ],
        };
        let all_children = root.into_iter().collect::<Vec<_>>();
        assert_eq!(
            all_children,
            vec![
                FileNode::File {
                    path: "/1".into(),
                    size: 1
                },
                FileNode::File {
                    path: "/2/3".into(),
                    size: 3
                }
            ]
        )
    }
}
