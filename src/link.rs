use std::path::PathBuf;

/// Structure containing the source/target information for the link.
#[derive(Debug, Clone, Default)]
pub struct Link {
    /// The location of the file that the link points to
    pub source: PathBuf,
    /// The name of the link on the file system
    pub target: PathBuf,
}
