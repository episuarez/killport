//! Project detection: walk up from the process cwd (fallback: exe dir) looking for
//! a project-root marker. Name comes from package.json when present, else dir name.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: String,
}

const MARKERS: &[&str] = &[
    "package.json",
    "pyproject.toml",
    "composer.json",
    "Cargo.toml",
    "go.mod",
    ".git",
];

const MAX_DEPTH: usize = 8;

pub fn detect(cwd: Option<&str>, exe: Option<&str>) -> Option<Project> {
    let start = cwd
        .map(PathBuf::from)
        .or_else(|| exe.map(|e| PathBuf::from(e).parent().map(Path::to_path_buf))?)?;

    let mut dir = start.as_path();
    for _ in 0..MAX_DEPTH {
        for marker in MARKERS {
            let candidate = dir.join(marker);
            if candidate.exists() {
                let name = if *marker == "package.json" {
                    package_name(&candidate).unwrap_or_else(|| dir_name(dir))
                } else {
                    dir_name(dir)
                };
                return Some(Project {
                    name,
                    path: dir.to_string_lossy().into_owned(),
                });
            }
        }
        dir = dir.parent()?;
    }
    None
}

fn dir_name(dir: &Path) -> String {
    dir.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| dir.to_string_lossy().into_owned())
}

fn package_name(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    json.get("name")?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::detect;

    #[test]
    fn finds_marker_walking_up() {
        // The crate dir itself has Cargo.toml -> must resolve to a project.
        let here = env!("CARGO_MANIFEST_DIR");
        let nested = format!("{here}/src");
        let p = detect(Some(&nested), None).expect("project");
        assert_eq!(p.name, "killport-core");
    }
}
