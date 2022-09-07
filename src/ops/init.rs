use std::{fs, path::Path};

use anyhow::{bail, Result};

pub fn init(path: &Path, name: &str) -> Result<()> {
    let manifest_path = path.join(crate::paths::MANIFEST_FILENAME);

    if manifest_path.exists() {
        bail!("Already initialized")
    }

    fs::write(
        manifest_path,
        format!(
            "\
[Package]
Name = \"{}\"
Version = \"0.1.0\"
Description = \"\"
",
            name
        ),
    )?;
    Ok(())
}
