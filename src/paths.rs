use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub const PACKAGE_FILE: &'static str = "Package.toml";
pub const LOCK_FILE: &'static str = "Package.lock";

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

pub fn index() -> PathBuf {
    home().join("index.toml")
}

pub fn token() -> PathBuf {
    home().join("token")
}

pub fn home() -> PathBuf {
    dirs::home_dir().unwrap().join(".grill")
}

pub fn beeflibs() -> PathBuf {
    beef().join("BeefLibs")
}

pub fn beeflib<P>(pkg: P) -> PathBuf
where
    P: AsRef<Path>,
{
    beeflibs().join(pkg)
}

pub fn themes() -> PathBuf {
    beef().join("bin").join("themes")
}

pub fn beef() -> PathBuf {
    PathBuf::from(env::var("BeefPath").expect("No Beef installation found"))
}

fn ensure_exists(path: PathBuf) -> PathBuf {
    fs::create_dir_all(&path).unwrap();
    path
}
