use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Link {
    /// The location of the link
    pub source: PathBuf,
    /// The file to be linked to
    pub target: PathBuf,
}
