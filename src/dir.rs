use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn pkg<P>(pkg: P) -> PathBuf
where
    P: AsRef<Path>,
{
    pkgs().join(pkg)
}

pub fn pkgs() -> PathBuf {
    ensure_exists(home().join("pkg"))
}

pub fn tmp() -> PathBuf {
    ensure_exists(home().join("tmp"))
}

fn ensure_exists(path: PathBuf) -> PathBuf {
    fs::create_dir_all(&path).unwrap();
    path
}

pub fn home() -> PathBuf {
    dirs::home_dir().unwrap().join(".grill")
}
