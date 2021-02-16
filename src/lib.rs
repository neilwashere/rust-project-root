//! # project root
//!
//! Helper to find the absolute root directory path of a project as it stands relative
//! to the location of the nearest Cargo.lock file.

use std::{env, io};
use std::fs::read_dir;
use std::path::PathBuf;

/// Get the project root (relative to closest Cargo.lock file)
/// ```rust
/// let project_root = match project_root::get_project_root() {
///     Ok(p) => p.to_str().expect("Could not retrieve project path").to_string(),
///     Err(e) => panic!(e),
/// };
/// println!("Current project root is {}", project_root);
/// ```
pub fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();
    let mut path_component = path_ancestors.next();

    loop {
        let have_project_root = match path_component {
            None => panic!("no directories left to check :/"),
            Some(p) => {
                // do any entries in this directory look like Cargo.toml?
                read_dir(p)?
                    .into_iter()
                    .any(|p| {
                        p.expect("Unable to get directory entry")
                            .file_name()
                            .to_str()
                            .unwrap()
                            == "Cargo.lock"
                    })
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
    use std::fs::read_to_string;

    #[test]
    fn it_should_find_our_project_root() {
        let crate_name = "name = \"project-root\"";

        let project_root = get_project_root().expect("There is no project root");

        let toml_path = project_root.to_str().unwrap().to_owned() + "/Cargo.toml";
        let toml_string = read_to_string(toml_path).unwrap();

        assert!(toml_string.contains(crate_name));
    }
}
