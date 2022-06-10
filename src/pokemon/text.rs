use byteorder::ReadBytesExt;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Seek;
use super::super::read::Reader;

#[derive(Debug)]
pub struct Texts<'a> {
    file: Reader<'a>,
    header: Header,
}

impl<'a> Texts<'a> {
    pub fn new(mut file: Reader<'a>) -> Result<Self, std::io::Error> {
        let header = Header::read(&mut file)?;

        Ok(Self { file, header })
    }

    pub fn entries<'b>(&'b self) -> TextIterator<'a, 'b> {
        TextIterator {
            file: self.file.clone(),
            header: &self.header,
            index: 0,
        }
    }
}

pub struct TextIterator<'a, 'b> {
    file: Reader<'a>,
    header: &'b Header,
    index: u16,
}

impl<'a, 'b> Iterator for TextIterator<'a, 'b> {
    type Item = Result<String, std::io::Error>;
    fn next(&mut self) -> Option<Result<String, std::io::Error>> {
        self.try_next().transpose()
    }
}

impl<'a, 'b> TextIterator<'a, 'b> {
    fn try_next(&mut self) -> Result<Option<String>, std::io::Error> {
        if self.index < self.header.line_count {
            self.file.seek(SeekFrom::Start(20 + self.index as u64 * 8))?;
            let entry = LineEntry::read(&mut self.file)?;

            self.file.seek(SeekFrom::Start(16 + entry.offset as u64))?;
            let poop = entry.read_line(&mut self.file, self.index)?;

            self.index += 1;

            Ok(Some(poop))
        } else {
            Ok(None)
        }
    }
}


// Unsure where the section header begins.
#[derive(Default, Debug)]
pub struct Header {
    section_count: u16,
    pub line_count: u16,
    total_length: u32,
    initial_key: u32,
    section_data_offset: u32,
    section_length: u32,
}

impl Header {
    pub fn read(input: &mut Reader) -> Result<Self, std::io::Error> {
        let mut header = Self::default();

        header.section_count = input.read_u16::<LittleEndian>()?;
        header.line_count = input.read_u16::<LittleEndian>()?;
        header.total_length = input.read_u32::<LittleEndian>()?;
        header.initial_key = input.read_u32::<LittleEndian>()?;
        header.section_data_offset = input.read_u32::<LittleEndian>()?;
        header.section_length = input.read_u32::<LittleEndian>()?;

        Ok(header)
    }
}

#[derive(Debug, Default)]
struct LineEntry {
    offset: u32,
    length: u16,
    unknown0: u16,
}

impl LineEntry {
    pub fn read(input: &mut Reader) -> Result<Self, std::io::Error> {
        let mut entry = Self::default();

        entry.offset = input.read_u32::<LittleEndian>()?;
        entry.length = input.read_u16::<LittleEndian>()?;
        entry.unknown0 = input.read_u16::<LittleEndian>()?;

        Ok(entry)
    }

    pub fn read_line(&self, input: &mut Reader, n: u16) -> Result<String, std::io::Error> {
        let mut buf = vec![];
        buf.resize(2 * self.length as usize, 0);
        input.read_exact(&mut buf)?;

        decode(&buf, n)
    }
}

fn key(index: u16) -> u16 {
    ((index as u32 * 0x2983 + 0x7C89) & 0xFFFF) as u16
}

fn decode(bytes: &[u8], index: u16) -> Result<String, std::io::Error> {
    let mut text = String::new();

    let mut key = key(index);
    for i in (0..bytes.len()).step_by(2) {
        let character = LittleEndian::read_u16(&bytes[i..i+2]) ^ key;

        text.push(char::from_u32(character as u32).unwrap());

        key = key.rotate_left(3);
    }

    Ok(text.trim_end_matches('\x00').to_string())
}
