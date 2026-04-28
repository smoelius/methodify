use methodify::methodify;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[methodify]
fn is_executable<P: AsRef<Path>>(path: &P) -> bool {
    let path = path.as_ref();

    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[methodify]
fn push_if_executable(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_executable() {
        paths.push(path);
    }
}

#[methodify]
fn executable_count(paths: &[PathBuf]) -> usize {
    paths.iter().filter(|path| path.is_executable()).count()
}

#[methodify]
fn has_executable<P: AsRef<Path>>(paths: &[P]) -> bool {
    paths.iter().any(|path| path.as_ref().is_executable())
}

#[test]
fn turns_first_argument_into_receiver() {
    let executable = std::env::current_exe().unwrap();
    assert!(executable.is_executable());

    let mut executable_paths = Vec::new();
    executable_paths.push_if_executable(executable.clone());
    assert_eq!(executable_paths, vec![executable.clone()]);

    assert_eq!(executable_paths.as_slice().executable_count(), 1);

    assert!(executable_paths.as_slice().has_executable());
}
