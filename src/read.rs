use std::io::SeekFrom;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Seek;

#[derive(Debug, Clone)]
pub struct Reader<'a> {
    file: &'a std::sync::RwLock<std::fs::File>,
    offset: u64,
    length: u64,
    position: u64,
}

impl<'a> Reader<'a> {
    pub fn new(file: &'a std::sync::RwLock<std::fs::File>, offset: u64, length: u64) -> Reader {
        Reader {
            file,
            offset,
            length,
            position: 0,
        }
    }

    pub fn limit(&self, offset: u64, length: u64) -> Result<Reader, Error> {
        if offset + length <= self.length {
            Ok(Reader {
                file: self.file,
                offset: self.offset + offset,
                length: length,
                position: 0,
            })
        } else {
            Err(Error::new(ErrorKind::Other, "Reader limit out of bounds"))
        }
    }

    pub fn at_zero(&self) -> Reader<'a> {
        Reader {
            file: self.file.clone(),
            offset: self.offset,
            length: self.length,
            position: 0,
        }
    }
}

impl<'a> std::io::Read for Reader<'a> {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let mut f = self.file.write().map_err(|_e| Error::new(ErrorKind::Other, "Write lock failed"))?;

        f.seek(SeekFrom::Start(self.offset + self.position))?;

        let maxread = buffer.len().min((self.length - self.position) as usize);

        let bytes_read = f.read(&mut buffer[0..maxread])?;
        self.position += bytes_read as u64;

        Ok(bytes_read)
    }
}

impl<'a> std::io::Seek for Reader<'a> {
    fn seek(&mut self, destination: SeekFrom) -> Result<u64, Error> {
        match destination {
            SeekFrom::End(offset) => {
                if offset > 0 {
                    Err(Error::new(ErrorKind::Other, "Can't seek past end of file"))
                } else {
                    let offset = (-offset) as u64;
                    if offset < self.length {
                        self.position = self.length - offset;
                        Ok(self.position)
                    } else {
                        Err(Error::new(ErrorKind::Other, "Can't seek before start of file"))
                    }
                }
            },
            SeekFrom::Start(offset) => {
                if offset < self.length {
                    self.position = offset;
                    Ok(self.position)
                } else {
                    Err(Error::new(ErrorKind::Other, "Can't seek past end of file"))
                }
            },
            SeekFrom::Current(offset) => {
                if offset < 0 {
                    let offset = (-offset) as u64;
                    if self.position < offset {
                        Err(Error::new(ErrorKind::Other, "Can't seek before start of file"))
                    } else {
                        self.position -= offset;
                        Ok(self.position)
                    }
                } else {
                    let offset = offset as u64;
                    if self.position + offset < self.length {
                        self.position += offset;
                        Ok(self.position)
                    } else {
                        Err(Error::new(ErrorKind::Other, "Can't seek past end of file"))
                    }
                }
            }
        }
    }
}

pub trait VirtualFile<'a> {
    fn reader(&'a self) -> Reader<'a>;
}

pub struct FileHolder {
    file: std::sync::RwLock<std::fs::File>,
    length: u64,
}

impl FileHolder {
    pub fn new(mut file: std::fs::File) -> Result<Self, std::io::Error> {
        let length = file.seek(std::io::SeekFrom::End(0))?;

        Ok(FileHolder { file: file.into(), length })
    }

    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        Self::new(std::fs::File::open(filename)?)
    }
}

impl<'a> VirtualFile<'a> for FileHolder {
    fn reader(&'a self) -> Reader<'a> {
        Reader::new(&self.file, 0, self.length)
    }
}
