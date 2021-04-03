#[cfg(test)]
mod tests {
    #[test]
    fn write_and_read_full() {
        use std::io::Write;

        let content = "File write and read test".as_bytes().to_vec();
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

    #[test]
    fn file_compress_decompress() {
        use std::io::Write;

        let content = b"File Compression and decompression test";
        let filename = "".to_string();
        let tmp_dir = tempdir::TempDir::new("sfss").unwrap();
        std::env::set_var("SFSS_LOCATION", tmp_dir.path());
        dbg!(std::env::var("SFSS_LOCATION").unwrap());
        let mut input = super::SfssFile::create(filename.clone(), true, true, false);

        input.write_all(content).unwrap();
        assert_eq!(input.buf, content);
        input.flush().unwrap();

        input.decompress().unwrap();
        assert_eq!(input.buf, content)
    }

    #[test]
    fn compress_and_decompress() {
        use flate2::read::ZlibDecoder;
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Read;
        use std::io::Write;

        let content = "This is some plain text".as_bytes().to_vec();
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(&content).unwrap();
        let compressed = e.finish().unwrap();

        let mut z = ZlibDecoder::new(&compressed[..]);
        let mut b = Vec::new();
        z.read_to_end(&mut b).unwrap();

        assert_eq!(content, b);
    }
}
