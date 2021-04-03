use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileFlags {
    pub public: bool,
    pub protected: bool,
    pub no_preview: bool,
}

impl Default for FileFlags {
    fn default() -> Self {
        Self {
            public: true,
            protected: false,
            no_preview: false,
        }
    }
}

impl FileFlags {
    pub fn from_iter<I: Iterator<Item = bool>>(iter: &mut I) -> Self {
        Self {
            public: iter.next().unwrap_or(false),
            protected: iter.next().unwrap_or(false),
            no_preview: iter.next().unwrap_or(false),
        }
    }
}
