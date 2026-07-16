use std::path::PathBuf;


pub struct NfsHandler {
    export_root: PathBuf,
}

impl NfsHandler {
    pub fn new(export_root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let export_root = export_root.into();
        if !export_root.is_dir() {
            anyhow::bail!("export root is not a directory: {}", export_root.display());
        }
        Ok(Self {
            export_root: export_root.canonicalize()?,
        })
    }
}
