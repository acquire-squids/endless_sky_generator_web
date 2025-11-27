use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use flate2::{Compression, write::DeflateEncoder};
use rawzip::{self, CompressionMethod, ZipArchiveWriter};

type ZipBytes<'a> = io::Cursor<&'a mut Vec<u8>>;

pub struct Zip<'a> {
    writer: ZipArchiveWriter<ZipBytes<'a>>,
}

impl<'a> Zip<'a> {
    pub fn new(output: &'a mut Vec<u8>) -> Self {
        Zip {
            writer: ZipArchiveWriter::new(io::Cursor::new(output)),
        }
    }

    pub fn write_file<P: Into<PathBuf>>(
        &mut self,
        path: P,
        bytes: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let path = P::into(path).display().to_string();

        let (mut entry, config) = self
            .writer
            .new_file(path.as_str())
            .compression_method(CompressionMethod::Deflate)
            .start()?;

        let encoder = DeflateEncoder::new(&mut entry, Compression::default());

        let mut writer = config.wrap(encoder);

        writer.write_all(bytes)?;

        let (_, descriptor) = writer.finish()?;

        let _compressed_len = entry.finish(descriptor)?;

        Ok(())
    }

    pub fn write_dir<P: Into<PathBuf>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let path = P::into(path).display().to_string();
        self.writer.new_dir(path.as_str()).create()?;
        Ok(())
    }

    pub fn finish(self) -> Result<ZipBytes<'a>, Box<dyn Error>> {
        Ok(self.writer.finish()?)
    }
}
