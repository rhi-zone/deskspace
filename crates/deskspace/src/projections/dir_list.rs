use async_trait::async_trait;

use crate::projection::{DirectoryEntry, Projection, ProjectionOutput, Resource, Result};
use crate::workspace::Workspace;

pub struct DirList;

#[async_trait]
impl Projection for DirList {
    fn id(&self) -> &str {
        "dir.list"
    }

    fn name(&self) -> &str {
        "Directory Listing"
    }

    fn confidence(&self, resource: &Resource) -> f32 {
        if resource.is_dir {
            1.0
        } else {
            0.0
        }
    }

    async fn project(
        &self,
        resource: &Resource,
        workspace: &Workspace,
    ) -> Result<ProjectionOutput> {
        let entries = workspace
            .read_dir(std::path::Path::new(&resource.path))
            .await?;
        let entries = entries
            .into_iter()
            .map(|e| DirectoryEntry {
                extension: if e.is_dir {
                    None
                } else {
                    std::path::Path::new(&e.name)
                        .extension()
                        .map(|ext| ext.to_string_lossy().to_lowercase())
                },
                name: e.name,
                is_dir: e.is_dir,
                size: e.size,
            })
            .collect();
        Ok(ProjectionOutput::DirectoryList { entries })
    }
}
