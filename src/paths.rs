use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub const MANIFEST_FILENAME: &'static str = "Package.toml";
pub const LOCK_FILENAME: &'static str = "Package.lock";

pub fn pkg<P>(ws: &P, pkg: &P) -> PathBuf
where
    P: AsRef<Path> + ?Sized,
{
    pkgs(ws).join(pkg)
}

pub fn pkgs<P>(ws: &P) -> PathBuf
where
    P: AsRef<Path> + ?Sized,
{
    ensure_exists(ws.as_ref().join("pkg"))
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
    ensure_exists(dirs::home_dir().unwrap().join(".grill"))
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
