use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;

use crate::projection::{Projection, Resource};

pub struct ProjectionRegistry {
    projections: HashMap<String, Arc<dyn Projection>>,
}

impl ProjectionRegistry {
    pub fn new() -> Self {
        Self {
            projections: HashMap::new(),
        }
    }

    pub fn register(&mut self, projection: Arc<dyn Projection>) {
        self.projections
            .insert(projection.id().to_owned(), projection);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn Projection>> {
        self.projections.get(id)
    }

    /// Return the projection with the highest confidence for the given resource.
    pub fn best_for(&self, resource: &Resource) -> Option<&Arc<dyn Projection>> {
        self.projections
            .values()
            .filter(|p| p.confidence(resource) > 0.0)
            .max_by(|a, b| {
                a.confidence(resource)
                    .partial_cmp(&b.confidence(resource))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Return all projections that match the resource, sorted by confidence descending.
    pub fn available_for(&self, resource: &Resource) -> Vec<ProjectionInfo> {
        let mut matches: Vec<_> = self
            .projections
            .values()
            .filter(|p| p.confidence(resource) > 0.0)
            .map(|p| ProjectionInfo {
                id: p.id().to_owned(),
                name: p.name().to_owned(),
                confidence: p.confidence(resource),
            })
            .collect();
        matches.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches
    }
}

impl Default for ProjectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectionInfo {
    pub id: String,
    pub name: String,
    pub confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projection::{ProjectionError, ProjectionOutput, Resource};
    use crate::workspace::Workspace;
    use async_trait::async_trait;

    struct DummyProjection {
        proj_id: &'static str,
        conf: f32,
    }

    #[async_trait]
    impl Projection for DummyProjection {
        fn id(&self) -> &str {
            self.proj_id
        }
        fn name(&self) -> &str {
            "Dummy"
        }
        fn confidence(&self, _resource: &Resource) -> f32 {
            self.conf
        }
        async fn project(
            &self,
            _resource: &Resource,
            _workspace: &Workspace,
        ) -> crate::projection::Result<ProjectionOutput> {
            Err(ProjectionError::Unsupported)
        }
    }

    #[test]
    fn best_for_returns_highest_confidence() {
        let mut reg = ProjectionRegistry::new();
        reg.register(Arc::new(DummyProjection {
            proj_id: "low",
            conf: 0.3,
        }));
        reg.register(Arc::new(DummyProjection {
            proj_id: "high",
            conf: 0.9,
        }));
        let resource = Resource::new("test.txt".into(), false);
        let best = reg.best_for(&resource).unwrap();
        assert_eq!(best.id(), "high");
    }

    #[test]
    fn available_for_sorted_descending() {
        let mut reg = ProjectionRegistry::new();
        reg.register(Arc::new(DummyProjection {
            proj_id: "low",
            conf: 0.3,
        }));
        reg.register(Arc::new(DummyProjection {
            proj_id: "high",
            conf: 0.9,
        }));
        let resource = Resource::new("test.txt".into(), false);
        let available = reg.available_for(&resource);
        assert_eq!(available[0].id, "high");
        assert_eq!(available[1].id, "low");
    }
}
