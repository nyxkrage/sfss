use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    Text,
    Binary(BinaryType),
    // The string is for specifying the language, to be used for syntax highlighting
    Code(u32),
}
impl Default for FileType {
    fn default() -> Self {
        FileType::Text
    }
}

impl FileType {
    pub fn as_bytes(&self) -> [u8; 4] {
        match self {
            Self::Text => [0, 0, 0, 0],
            Self::Binary(b) => match b {
                BinaryType::Previewable => [1, 0, 0, 0],
                BinaryType::NonPreviewable => [1, 0, 0, 1],
            },
            Self::Code(id) => {
                let mut res = [2, 0, 0, 0];
                LittleEndian::write_u24(&mut res[1..], *id);
                res
            }
        }
    }

    pub fn from_bytes(b: [u8; 4]) -> Self {
        match b[0] {
            2 => Self::Code(LittleEndian::read_u24(&b[1..])),
            1 => match b[3] {
                0 => Self::Binary(BinaryType::Previewable),
                1 | _ => Self::Binary(BinaryType::NonPreviewable),
            },
            0 | _ => Self::Text,
        }
    }

    // TODO: make concrete error enum to use in results
    pub fn to_hljs(&self) -> Option<&'static str> {
        if let FileType::Code(i) = self {
            highlightjs_rs::from_id(*i as usize)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryType {
    Previewable,
    NonPreviewable,
}
