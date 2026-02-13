use async_trait::async_trait;

use crate::projection::{Projection, ProjectionOutput, Resource, Result};
use crate::workspace::Workspace;

const TEXT_EXTENSIONS: &[&str] = &[
    "txt",
    "rs",
    "toml",
    "json",
    "yaml",
    "yml",
    "xml",
    "html",
    "css",
    "js",
    "ts",
    "tsx",
    "jsx",
    "py",
    "rb",
    "go",
    "c",
    "cpp",
    "h",
    "hpp",
    "java",
    "kt",
    "swift",
    "sh",
    "bash",
    "zsh",
    "fish",
    "ps1",
    "bat",
    "cmd",
    "sql",
    "graphql",
    "proto",
    "lua",
    "vim",
    "el",
    "clj",
    "ex",
    "exs",
    "erl",
    "hs",
    "ml",
    "mli",
    "r",
    "jl",
    "nim",
    "zig",
    "v",
    "d",
    "ada",
    "pas",
    "f90",
    "lisp",
    "scm",
    "rkt",
    "conf",
    "ini",
    "cfg",
    "env",
    "lock",
    "csv",
    "tsv",
    "log",
    "diff",
    "patch",
    "nix",
    "tf",
    "dockerfile",
    "makefile",
];

pub struct TextRaw;

impl TextRaw {
    fn detect_language(ext: &str) -> Option<String> {
        match ext {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" => Some("javascript"),
            "ts" => Some("typescript"),
            "tsx" => Some("tsx"),
            "jsx" => Some("jsx"),
            "rb" => Some("ruby"),
            "go" => Some("go"),
            "c" | "h" => Some("c"),
            "cpp" | "hpp" => Some("cpp"),
            "java" => Some("java"),
            "kt" => Some("kotlin"),
            "swift" => Some("swift"),
            "sh" | "bash" | "zsh" | "fish" => Some("bash"),
            "sql" => Some("sql"),
            "html" => Some("html"),
            "css" => Some("css"),
            "json" => Some("json"),
            "yaml" | "yml" => Some("yaml"),
            "toml" => Some("toml"),
            "xml" => Some("xml"),
            "lua" => Some("lua"),
            "nix" => Some("nix"),
            "hs" => Some("haskell"),
            "ex" | "exs" => Some("elixir"),
            "erl" => Some("erlang"),
            "r" => Some("r"),
            "jl" => Some("julia"),
            "diff" | "patch" => Some("diff"),
            _ => None,
        }
        .map(String::from)
    }
}

#[async_trait]
impl Projection for TextRaw {
    fn id(&self) -> &str {
        "text.raw"
    }

    fn name(&self) -> &str {
        "Plain Text"
    }

    fn confidence(&self, resource: &Resource) -> f32 {
        if resource.is_dir {
            return 0.0;
        }
        match &resource.extension {
            Some(ext) if TEXT_EXTENSIONS.contains(&ext.as_str()) => 0.8,
            Some(_) => 0.0,
            None => 0.3, // extensionless files are often text
        }
    }

    async fn project(
        &self,
        resource: &Resource,
        workspace: &Workspace,
    ) -> Result<ProjectionOutput> {
        let content = workspace
            .read_to_string(std::path::Path::new(&resource.path))
            .await?;
        let line_count = content.lines().count();
        let language = resource
            .extension
            .as_deref()
            .and_then(Self::detect_language);
        Ok(ProjectionOutput::Text {
            content,
            language,
            line_count,
        })
    }
}
