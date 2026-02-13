use async_trait::async_trait;
use serde::Serialize;

use crate::workspace::{Workspace, WorkspaceError};

/// A resource that a projection operates on.
#[derive(Debug, Clone)]
pub struct Resource {
    /// Path relative to the workspace root.
    pub path: String,
    /// Whether this resource is a directory.
    pub is_dir: bool,
    /// File extension (lowercase, without dot), if any.
    pub extension: Option<String>,
}

impl Resource {
    pub fn new(path: String, is_dir: bool) -> Self {
        let extension = if is_dir {
            None
        } else {
            std::path::Path::new(&path)
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
        };
        Self {
            path,
            is_dir,
            extension,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("unsupported resource")]
    Unsupported,
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ProjectionError>;

#[async_trait]
pub trait Projection: Send + Sync {
    /// Unique identifier for this projection (e.g. "dir.list").
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// How well this projection handles the given resource. 0.0 = not at all, 1.0 = perfect.
    fn confidence(&self, resource: &Resource) -> f32;

    /// Produce the projection output for the given resource.
    async fn project(&self, resource: &Resource, workspace: &Workspace)
        -> Result<ProjectionOutput>;
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ProjectionOutput {
    DirectoryList {
        entries: Vec<DirectoryEntry>,
    },
    Text {
        content: String,
        language: Option<String>,
        line_count: usize,
    },
    Markdown {
        raw: String,
        toc: Vec<TocEntry>,
    },
    Image {
        mime_type: String,
        url: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub extension: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TocEntry {
    pub level: u8,
    pub text: String,
    pub slug: String,
}
