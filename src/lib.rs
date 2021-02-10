//! # project root
//!
//! Helper to find the absolute root directory path of a project as it stands relative
//! to the location of the nearest Cargo.lock file.

use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

/// Get the project root (relative to closest Cargo.lock file)
/// ```rust
/// let project_path = match project_root::get_project_root() {
///     Ok(p) => p.to_str().expect("Could not retrieve project path").to_string(),
///     Err(e) => panic!(e),
/// };
/// ```
pub fn get_project_root() -> Result<PathBuf, anyhow::Error> {
    let path = env::current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();
    let mut path_component = path_ancestors.next();

    loop {
        let have_project_root = match path_component {
            None => panic!("no directories left to check :/"),
            Some(p) => {
                // Get all paths (files or directories) at this location
                let paths = read_dir(p).unwrap();
                // return true if one of these paths is Cargo.lock. This means we are at the root!
                paths
                    .into_iter()
                    .any(|p| p.unwrap().file_name().to_str().unwrap() == "Cargo.lock")
            }
        };

        if have_project_root {
            break;
        }

        path_component = path_ancestors.next();
    }

    let project_path = path_component
        .unwrap()
        .to_str()
        .expect("Could not locate project root");

    Ok(PathBuf::from(project_path))
}

#[cfg(test)]
mod tests {
    use crate::get_project_root;
    use serde::Deserialize;
    use std::fs::read_to_string;

    #[derive(Deserialize)]
    struct Config {
        package: Package,
    }

    #[derive(Deserialize)]
    struct Package {
        name: String,
    }

    #[test]
    fn it_should_find_our_project_root() {
        let crate_name = "project-root";

        let project_root = get_project_root().expect("There is no project root");

        let toml_path = project_root.to_str().unwrap().to_owned() + "/Cargo.toml";
        let toml_string = read_to_string(toml_path).unwrap();
        let toml: Config = toml::from_str(toml_string.as_str()).unwrap();

        assert_eq!(toml.package.name, crate_name)
    }
}
