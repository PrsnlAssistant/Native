//! Media types

/// Selected media from the file picker
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedMedia {
    pub data: String,      // Base64 encoded
    pub mimetype: String,
    pub filename: String,
}
