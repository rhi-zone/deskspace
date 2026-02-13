use async_trait::async_trait;

use crate::projection::{Projection, ProjectionOutput, Resource, Result};
use crate::workspace::Workspace;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg"];

pub struct ImagePreview;

#[async_trait]
impl Projection for ImagePreview {
    fn id(&self) -> &str {
        "image.preview"
    }

    fn name(&self) -> &str {
        "Image Preview"
    }

    fn confidence(&self, resource: &Resource) -> f32 {
        if resource.is_dir {
            return 0.0;
        }
        match &resource.extension {
            Some(ext) if IMAGE_EXTENSIONS.contains(&ext.as_str()) => 1.0,
            _ => 0.0,
        }
    }

    async fn project(
        &self,
        resource: &Resource,
        _workspace: &Workspace,
    ) -> Result<ProjectionOutput> {
        let mime_type = mime_guess::from_path(&resource.path)
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());
        let url = format!("/api/files/raw/{}", resource.path);
        Ok(ProjectionOutput::Image { mime_type, url })
    }
}
