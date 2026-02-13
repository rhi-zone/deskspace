use async_trait::async_trait;

use crate::projection::{Projection, ProjectionOutput, Resource, Result, TocEntry};
use crate::workspace::Workspace;

pub struct TextMarkdown;

impl TextMarkdown {
    fn extract_toc(raw: &str) -> Vec<TocEntry> {
        let mut toc = Vec::new();
        for line in raw.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix('#') {
                let mut level: u8 = 1;
                let mut text_start = rest;
                while let Some(after) = text_start.strip_prefix('#') {
                    level += 1;
                    text_start = after;
                }
                if level > 6 {
                    continue;
                }
                let text = text_start.trim().to_string();
                if text.is_empty() {
                    continue;
                }
                let slug = text
                    .to_lowercase()
                    .chars()
                    .map(|c| {
                        if c.is_alphanumeric() || c == '-' || c == '_' {
                            c
                        } else if c == ' ' {
                            '-'
                        } else {
                            ' ' // will be filtered
                        }
                    })
                    .filter(|c| *c != ' ')
                    .collect();
                toc.push(TocEntry { level, text, slug });
            }
        }
        toc
    }
}

#[async_trait]
impl Projection for TextMarkdown {
    fn id(&self) -> &str {
        "text.markdown"
    }

    fn name(&self) -> &str {
        "Markdown"
    }

    fn confidence(&self, resource: &Resource) -> f32 {
        if resource.is_dir {
            return 0.0;
        }
        match resource.extension.as_deref() {
            Some("md" | "markdown") => 1.0,
            _ => 0.0,
        }
    }

    async fn project(
        &self,
        resource: &Resource,
        workspace: &Workspace,
    ) -> Result<ProjectionOutput> {
        let raw = workspace
            .read_to_string(std::path::Path::new(&resource.path))
            .await?;
        let toc = Self::extract_toc(&raw);
        Ok(ProjectionOutput::Markdown { raw, toc })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_toc_basic() {
        let md = "# Hello\n\nSome text\n\n## World\n\n### Deep\n";
        let toc = TextMarkdown::extract_toc(md);
        assert_eq!(toc.len(), 3);
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[0].text, "Hello");
        assert_eq!(toc[0].slug, "hello");
        assert_eq!(toc[1].level, 2);
        assert_eq!(toc[1].text, "World");
        assert_eq!(toc[2].level, 3);
    }

    #[test]
    fn extract_toc_slug_special_chars() {
        let md = "## Hello, World! (v2.0)\n";
        let toc = TextMarkdown::extract_toc(md);
        assert_eq!(toc[0].slug, "hello-world-v20");
    }

    #[test]
    fn extract_toc_skips_empty_headings() {
        let md = "# \n## Real heading\n";
        let toc = TextMarkdown::extract_toc(md);
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].text, "Real heading");
    }
}
