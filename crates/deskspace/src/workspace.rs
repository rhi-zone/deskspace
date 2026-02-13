use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("path escapes workspace root: {0}")]
    PathTraversal(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, WorkspaceError>;

#[derive(Clone)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn new(root: impl AsRef<Path>) -> std::io::Result<Self> {
        let root = root.as_ref().canonicalize()?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve a relative path to an absolute path within the workspace.
    /// Returns an error if the resolved path escapes the workspace root.
    pub fn resolve(&self, relative: impl AsRef<Path>) -> Result<PathBuf> {
        let relative = relative.as_ref();

        // Join with root — if relative is absolute, strip the leading /
        let joined = if relative.is_absolute() {
            self.root
                .join(relative.strip_prefix("/").unwrap_or(relative))
        } else {
            self.root.join(relative)
        };

        // Canonicalize if the path exists, otherwise canonicalize the parent
        let resolved = if joined.exists() {
            joined.canonicalize()?
        } else {
            let parent = joined
                .parent()
                .ok_or_else(|| WorkspaceError::PathTraversal(relative.display().to_string()))?;
            let file_name = joined
                .file_name()
                .ok_or_else(|| WorkspaceError::PathTraversal(relative.display().to_string()))?;
            parent.canonicalize()?.join(file_name)
        };

        if !resolved.starts_with(&self.root) {
            return Err(WorkspaceError::PathTraversal(
                relative.display().to_string(),
            ));
        }

        Ok(resolved)
    }

    pub async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let resolved = self.resolve(path)?;
        Ok(tokio::fs::read(resolved).await?)
    }

    pub async fn read_to_string(&self, path: &Path) -> Result<String> {
        let resolved = self.resolve(path)?;
        Ok(tokio::fs::read_to_string(resolved).await?)
    }

    pub async fn write(&self, path: &Path, contents: &[u8]) -> Result<()> {
        let resolved = self.resolve(path)?;
        if let Some(parent) = resolved.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        Ok(tokio::fs::write(resolved, contents).await?)
    }

    pub async fn metadata(&self, path: &Path) -> Result<std::fs::Metadata> {
        let resolved = self.resolve(path)?;
        Ok(tokio::fs::metadata(resolved).await?)
    }

    pub async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>> {
        let resolved = self.resolve(path)?;
        let mut rd = tokio::fs::read_dir(&resolved).await?;
        let mut entries = Vec::new();
        while let Some(entry) = rd.next_entry().await? {
            let meta = entry.metadata().await?;
            entries.push(DirEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir: meta.is_dir(),
                size: meta.len(),
            });
        }
        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(entries)
    }
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolve_normal_path() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("hello.txt"), "hi").unwrap();
        let ws = Workspace::new(dir.path()).unwrap();
        let resolved = ws.resolve("hello.txt").unwrap();
        assert!(resolved.starts_with(ws.root()));
        assert!(resolved.ends_with("hello.txt"));
    }

    #[test]
    fn reject_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path()).unwrap();
        let result = ws.resolve("../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn resolve_absolute_path_stripped() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/file.txt"), "data").unwrap();
        let ws = Workspace::new(dir.path()).unwrap();
        let resolved = ws.resolve("/sub/file.txt").unwrap();
        assert!(resolved.starts_with(ws.root()));
    }

    #[tokio::test]
    async fn read_dir_sorts_dirs_first() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("b.txt"), "").unwrap();
        fs::create_dir(dir.path().join("a_dir")).unwrap();
        fs::write(dir.path().join("a.txt"), "").unwrap();
        let ws = Workspace::new(dir.path()).unwrap();
        let entries = ws.read_dir(Path::new("")).await.unwrap();
        assert!(entries[0].is_dir);
        assert_eq!(entries[0].name, "a_dir");
    }
}
