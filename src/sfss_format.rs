use core::panic;
use std::{fs::File, io::Cursor};

use byteorder::{ByteOrder, LE};
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::io::Result as IoResult;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

const MAGIC_BYTES: [u8; 6] = [ 53, 46, 53, 53, 253, 254];

// FILE STRUCTURE:
// 6 bytes: MAGIC DATA [35 2E 35 35 FD FE]
// 2 bytes: FILENAME LEN
// X bytes: FILENAME
// 4 bytes: FILETYPE [MAJOR_TYPE IDENTIFIER IDENTIFIER IDENTIFIER]
// 8 bytes: PASSWORD
// 1 bytes: FLAGS

fn bools_to_u8(bools: [bool; 8]) -> u8 {
    // true true false...
    // 1100_0000
    let mut res: u8 = 0;
    for (i, b) in bools.iter().enumerate() {
        res |= (*b as u8) << (7 - i)
    }
    res
}

fn u8_to_bools(byte: u8) -> [bool; 8] {
    let mut res = [false; 8];
    for (i, b) in res.iter_mut().enumerate() {
        *b = byte & 1 << (7 - i) != 0;
    }
    res
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    Text,
    Binary(BinaryType),
    // The string is for specifying the language, to be used for syntax highlighting
    Code(String),
}
impl Default for FileType {
    fn default() -> Self {
        FileType::Text
    }
}

impl FileType {
    fn as_bytes(&self) -> [u8; 4] {
        match self {
            Self::Text => [0, 0, 0, 0],
            Self::Binary(b) => match b {
                BinaryType::Previewable => [1, 0, 0, 0],
                BinaryType::NonPreviewable => [1, 0, 0, 1],
            },
            Self::Code(c) => {
                if c.len() > 3 {
                    return [2, 0, 0, 0];
                }
                let mut res = [2, 0, 0, 0];
                for (i, b) in c.as_bytes().iter().enumerate() {
                    // This wont fail, as we gaurentee that the length of the string is less than 3.
                    res[i + 4 - c.len()] = *b;
                }
                res
            }
        }
    }

    fn from_bytes(b: [u8; 4]) -> Self {
        match b[0] {
            2 => Self::Code(String::from_utf8(b[1..].to_vec()).unwrap_or("".to_string())),
            1 => match b[3] {
                0 => Self::Binary(BinaryType::Previewable),
                1 | _ => Self::Binary(BinaryType::NonPreviewable),
            },
            0 | _ => Self::Text,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryType {
    Previewable,
    NonPreviewable,
}

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
    fn from_iter<I: Iterator<Item = bool>>(iter: &mut I) -> Self {
        Self {
            public: iter.next().unwrap_or(false),
            protected: iter.next().unwrap_or(false),
            no_preview: iter.next().unwrap_or(false),
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct SfssFile {
    pub filename: String,
    pub hash: String,
    pub filetype: FileType,
    pub flags: FileFlags,
    pub password: Option<String>,
    pub file: std::path::PathBuf,
    compressed: bool,
    buf: Vec<u8>,
}

impl std::fmt::Debug for SfssFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n\tFilename: {:?}\n\tHash: {:?}\n\tType: {:?}\n\tFlags: {:?}\n\tPassword: {:?}\n\tPath: {:?}\n\tCompressed {:?}\n}}", self.filename, self.hash, self.filetype, self.flags, self.password, self.file, self.compressed)
    }
}

impl std::default::Default for SfssFile {
    fn default() -> Self {
        Self {
            filename: String::default(),
            hash: String::default(),
            filetype: FileType::Text, // TODO: Change to check magic bytes of input
            file: std::path::PathBuf::from(std::env::var("SFSS_LOCATION").unwrap()),
            flags: FileFlags::default(),
            password: None,
            compressed: false,
            buf: Vec::new(),
        }
    }
}

use rocket::http::ContentType;
impl SfssFile {
    fn content_type(&self) -> ContentType {
        match self.filetype {
            FileType::Text => ContentType::Plain,
            FileType::Code(_) => ContentType::Plain,
            FileType::Binary(BinaryType::Previewable) => {
                ContentType::from_extension(self.filename.rsplit('.').nth(0).unwrap())
                    .unwrap_or(ContentType::Binary)
            }
            FileType::Binary(BinaryType::NonPreviewable) => ContentType::Binary,
        }
    }

    fn force_write(&mut self) -> IoResult<()> {
        let mut fd = if self.file.is_file() {
            let mut fd = std::fs::OpenOptions::new().write(true).open(&self.file)?;
            fd.seek(SeekFrom::Start(0)).unwrap();
            fd.set_len(0).unwrap();
            fd
        } else {
            std::fs::File::create(&self.file)?
        };
        fd.write_all(&self.header_as_bytes())?;
        fd.write_all(&self.buf)?;
        Ok(())
    }

    #[inline]
    pub fn verify_magic(bytes: [u8; 6]) -> bool {
        bytes == MAGIC_BYTES
    }

    pub fn set_password(&mut self) -> bool {
        if self.password == None {
            self.password = passwords::PasswordGenerator::new()
                .uppercase_letters(true)
                .generate_one().ok();
                true
        } else {
            false
        }
    }

    pub fn open(&mut self) -> IoResult<()> {
        let fd = File::open(&mut self.file)?;
        let mut reader = BufReader::new(fd);
        self.header_from_bytes(&mut reader)?;
        self.compressed = true;
        reader.read_to_end(&mut self.buf)?;
        Ok(())
    }

    pub fn new(hashcode: String, only_header: bool) -> IoResult<Self> {
        let mut path = std::path::PathBuf::from(std::env::var("SFSS_LOCATION").unwrap());
        let mut res = Self::default();
        path.push(&hashcode);
        res.file = path;
        let fd = File::open(&res.file)?;
        let mut reader = BufReader::new(fd);
        res.hash = hashcode;
        res.header_from_bytes(&mut reader)?;
        res.compressed = true;
        if false == only_header {
            reader.read_to_end(&mut res.buf)?;
        }
        Ok(res)
    }

    pub fn create(filename: String, public: bool, protected: bool, no_preview: bool) -> Self {
        SfssFile {
            filename: filename,
            hash: String::default(),
            filetype: FileType::Text, // TODO: Change to check magic bytes of input
            file: std::path::PathBuf::from(std::env::var("SFSS_LOCATION").unwrap()),
            flags: FileFlags {
                public,
                protected,
                no_preview,
            },
            password: if protected {
                passwords::PasswordGenerator::new()
                    .uppercase_letters(true)
                    .generate_one()
                    .ok()
            } else {
                None
            },
            compressed: false,
            buf: Vec::new(),
        }
    }

    fn header_as_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(21 + &self.filename.len());
        buf.write_all(&MAGIC_BYTES).unwrap();
        buf.write_all(&(self.filename.len() as u16).to_le_bytes())
            .unwrap();
        buf.write_all(&self.filename.as_bytes()).unwrap();
        buf.write_all(&self.filetype.as_bytes()).unwrap();
        if let Some(password) = &self.password {
            buf.write_all(&password.as_bytes()).unwrap();
        } else {
            buf.write_all(b"\x00\x00\x00\x00\x00\x00\x00\x00").unwrap()
        }
        let flags = [
            self.flags.public,
            self.flags.protected,
            self.flags.no_preview,
            false,
            false,
            false,
            false,
            false,
        ];
        buf.write(&mut [bools_to_u8(flags)]).unwrap();
        buf
    }

    fn header_from_bytes<R: Read>(&mut self, reader: &mut BufReader<R>) -> IoResult<()> {
        let mut magic: [u8; 6] = [0; 6];
        reader.read_exact(&mut magic)?;
        if false == SfssFile::verify_magic(magic) {
            return Err(IoError::from(IoErrorKind::InvalidInput));
        };

        let mut filename_len: [u8; 2] = [0; 2];
        reader.read_exact(&mut filename_len)?;

        let filename_len: u16 = LE::read_u16(&filename_len);
        reader
            .take(filename_len as u64)
            .read_to_string(&mut self.filename)?;

        let mut filetype: [u8; 4] = [0; 4];
        reader.read_exact(&mut filetype)?;
        self.filetype = FileType::from_bytes(filetype);

        let mut password: [u8; 8] = [0; 8];
        reader.read_exact(&mut password)?;
        if password == [0; 8] {
            self.password = None;
        } else {
            self.password = String::from_utf8(Vec::from(password)).ok()
        }

        let mut flag_bytes: [u8; 1] = [0];
        reader.read(&mut flag_bytes)?;
        self.flags = FileFlags::from_iter(&mut u8_to_bools(flag_bytes[0]).iter().copied());

        Ok(())
    }

    fn compress(&mut self) -> IoResult<u64> {
        if true == self.compressed {
            return Err(IoError::from(IoErrorKind::InvalidData));
        }
        let mut buf: &[u8] = &Vec::new();

        let mut encoder =
            flate2::write::ZlibEncoder::new(&mut self.buf, flate2::Compression::fast());

        std::io::copy(&mut buf, &mut encoder)?;
        let size = encoder.total_out();
        encoder.finish().unwrap();

        self.compressed = true;
        Ok(size)
    }

    pub fn decompress(&mut self) -> IoResult<u64> {
        if false == self.compressed {
            return Err(IoError::from(IoErrorKind::InvalidData));
        }
        let mut buf: Vec<u8> = Vec::new();

        let mut decoder = flate2::read::ZlibDecoder::new(&self.buf[..]);
        std::io::copy(&mut decoder, &mut buf)?;
        let size = decoder.total_out();
        drop(decoder);
        std::io::copy(&mut &buf[..], &mut self.buf)?;

        self.compressed = false;
        Ok(size)
    }

    fn hash(&self) -> String {
        base_62::encode(&xxhrs::XXH3_64::hash(&self.buf).to_le_bytes())
    }
}

impl Write for SfssFile {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.compress()?;
        self.hash = self.hash();
        self.file.push(&self.hash);
        let mut fd = if self.file.is_file() {
            let mut fd = std::fs::File::open(&self.file)?;
            let mut tmp_buf: [u8; 6] = [0; 6];
            if let Err(e) = fd.read_exact(&mut tmp_buf) {
                if e.kind() != IoErrorKind::UnexpectedEof {
                    return Err(e);
                }
            }
            if tmp_buf == MAGIC_BYTES {
                return Err(IoError::from(IoErrorKind::AlreadyExists))
            }
            fd.seek(SeekFrom::Start(0)).unwrap();
            fd.set_len(0).unwrap();
            fd
        } else {
            std::fs::File::create(&self.file)?
        };
        fd.write_all(&self.header_as_bytes())?;
        fd.write_all(&self.buf)?;
        Ok(())
    }
}

impl Read for SfssFile {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        if self.compressed == true {
            return Err(IoError::from(IoErrorKind::InvalidData));
        }
        let mut buffer: &[u8] = &self.buf;
        buffer.read(buf)
    }
}

use rocket::response::Result as responseResult;
use rocket::{http::Status, response::Responder, Request, Response};

impl<'r> Responder<'r, 'static> for SfssFile {
    fn respond_to(self, _: &'r Request<'_>) -> responseResult<'static> {
        Response::build()
            .header(self.content_type())
            .status(Status::Ok)
            .sized_body(self.buf.len(), Cursor::new(self.buf))
            .ok()
    }
}

use multipart::server::Multipart;
use rocket::data::{FromData, Outcome};
use rocket::Data;

#[rocket::async_trait]
impl FromData for SfssFile {
    type Error = IoError;

    async fn from_data(request: &Request<'_>, data: Data) -> Outcome<Self, Self::Error> {
        // All of these errors should be reported
        let ct = request
            .headers()
            .get_one("Content-Type")
            .expect("no content-type");
        let idx = ct.find("boundary=").expect("no boundary");
        let boundary = &ct[(idx + 9)..];

        let mut d = Vec::new();
        data.open(128usize * rocket::data::ByteUnit::MiB)
            .stream_to(&mut d)
            .await
            .expect("Unable to read");

        let mut mp = Multipart::with_body(Cursor::new(d), boundary);
        let mut sfss_file = SfssFile::create("".into(), false, false, false);

        // Custom implementation parts
        mp.foreach_entry(|mut entry| match &*entry.headers.name {
            "public" => {
                sfss_file.flags.public = true;
            }
            "protected" => {
                sfss_file.flags.protected = true;
                sfss_file.set_password();
            }
            "no_preview" => {
                sfss_file.flags.no_preview = true;
            }
            "file" => {
                sfss_file.filename = entry.headers.filename.unwrap();
                std::io::copy(&mut entry.data, &mut sfss_file).unwrap();
            }
            _ => (),
        }).expect("Unable to iterate");

        if let Err(err) = sfss_file.flush() {
            if err.kind() == IoErrorKind::AlreadyExists {
                let existing = SfssFile::new(sfss_file.hash.clone(), true).unwrap();
                sfss_file.flags.public |= existing.flags.public;
                if false == (existing.flags.protected && sfss_file.flags.protected) {
                    sfss_file.flags.protected = false;
                    sfss_file.password = None;
                } else {
                    sfss_file.password = existing.password;
                };
                sfss_file.flags.no_preview &= existing.flags.no_preview;
                sfss_file.hash = existing.hash;

                sfss_file.force_write().unwrap();
            } else {
                panic!();
            }
        }

        // End custom
        Outcome::Success(sfss_file)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn write_and_read_full() {
        use std::io::Write;

        let content = "This is some plain text".as_bytes().to_vec();
        let filename = "test.txt".to_string();
        let tmp_dir = tempdir::TempDir::new("sfss").unwrap();
        std::env::set_var("SFSS_LOCATION", tmp_dir.path());
        dbg!(std::env::var("SFSS_LOCATION").unwrap());
        let mut input = super::SfssFile::create(filename.clone(), true, true, false);

        input.write_all(&content).unwrap();
        assert_eq!(input.buf, content);
        input.flush().unwrap();

        let output = super::SfssFile::new(input.hash.clone(), false).unwrap();

        assert_eq!(input, output);
    }
}
