//! # project root
//!
//! Helper to find the absolute root directory path of a project as it stands relative
//! to the location of the nearest Cargo.toml file.

use std::ffi::OsString;
use std::fs::read_dir;
use std::path::PathBuf;
use std::{env, io};
use std::io::ErrorKind;

/// Get the project root (relative to closest Cargo.toml file)
/// ```rust
/// match project_root::get_project_root() {
///     Ok(p) => println!("Current project root is {:?}", p),
///     Err(e) => println!("Error obtaining project root {:?}", e)
/// };
/// ```
pub fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();

    while let Some(p) = path_ancestors.next() {
        let has_cargo =
            read_dir(p)?
                .into_iter()
                .any(|p| p.unwrap().file_name() == OsString::from("Cargo.toml"));
        if has_cargo {
            return Ok(PathBuf::from(p))
        }
    }
    Err(io::Error::new(ErrorKind::NotFound, "Ran out of places to find Cargo.toml"))

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
