use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::io::Error;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use super::read::Reader;
use super::read::VirtualFile;

#[derive(Debug)]
pub struct GARC<'a> {
    file: Reader<'a>,
    header: Header,
}

impl<'a> GARC<'a> {
    pub fn new(mut file: Reader<'a>) -> Result<GARC, std::io::Error> {
        let header = Header::read(&mut file)?;

        Ok(GARC { file, header })
    }

    pub fn file_count(&self) -> u16 {
        self.header.otaf_file_count
    }

    pub fn entries<'b>(&'b self) -> FileIterator<'a, 'b> {
        FileIterator {
            context: FileIteratorContext {
                file: self.file.clone(),
                file_count: self.header.otaf_file_count,
                base_offset: self.header.btaf_offset,
                fat_offsets: &self.header.otaf_entries,
                data_offset: self.header.data_offset as u64,
            },
            index: 0,
        }
    }

    pub fn file_at(&self, i: usize, j: usize) -> Result<Option<SubfileEntry<'a>>, std::io::Error> {
        let mut index = 0;
        let mut it = self.entries();
        while let Some(entry) = it.next()? {
            if index == i {
                let mut subindex = 0;
                let mut jt = entry.entries();
                while let Some(subentry) = jt.next()? {
                    if subindex == j {
                        return Ok(Some(subentry));
                    }
                    subindex += 1;
                }
            }

            index += 1;
        }

        Ok(None)
    }
}

impl<'a> VirtualFile<'a> for GARC<'a> {
    fn reader(&self) -> Reader<'a> {
        self.file.at_zero()
    }
}

#[derive(Debug, Default)]
pub struct Header {
    magic: u32,
    header_length: u32,
    endianess: u16,

    version: u16,
    file_size: u32,

    data_offset: u32,
    file_length: u32,
    content_largest_padded: u32,
    content_largest_unpadded: u32,
    content_pad_to_nearest: u32,

    otaf_magic: u32,
    otaf_section_size: u32,
    otaf_file_count: u16,
    otaf_padding: u16,

    otaf_entries: Vec<u32>,

    btaf_magic: u32,
    btaf_section_size: u32,
    btaf_file_count: u32,
    btaf_offset: u64,

    bmif_magic: u32,
    bmif_section_size: u32,
    bmif_data_size: u32,
}

impl Header {
    pub fn read(input: &mut Reader) -> Result<Self, std::io::Error> {
        let mut buf = vec![];
        let mut header = Header::default();

        header.magic         = input.read_u32::<LittleEndian>()?;
        header.header_length = input.read_u32::<LittleEndian>()?;
        header.endianess     = input.read_u16::<LittleEndian>()?;

        header.version   = input.read_u16::<LittleEndian>()?;
        assert!(header.version == 0x400 || header.version == 0x600);

        header.file_size = input.read_u32::<LittleEndian>()?;

        header.data_offset = input.read_u32::<LittleEndian>()?;
        header.file_length = input.read_u32::<LittleEndian>()?;

        if header.version == 0x400 {
            header.content_largest_padded = input.read_u32::<LittleEndian>()?;
            header.content_largest_unpadded = header.content_largest_padded;
            header.content_pad_to_nearest = 4;
        } else {
            header.content_largest_padded = input.read_u32::<LittleEndian>()?;
            header.content_largest_unpadded = input.read_u32::<LittleEndian>()?;
            header.content_pad_to_nearest = input.read_u32::<LittleEndian>()?;
        }

        header.otaf_magic = input.read_u32::<LittleEndian>()?;
        assert!(header.otaf_magic == 1178686543);
        header.otaf_section_size = input.read_u32::<LittleEndian>()?;
        header.otaf_file_count = input.read_u16::<LittleEndian>()?;
        header.otaf_padding = input.read_u16::<LittleEndian>()?;

        assert!(header.otaf_section_size - 12 == header.otaf_file_count as u32 * 4);

        for _ in 0..header.otaf_file_count {
            let v = input.read_u32::<LittleEndian>()?;
            header.otaf_entries.push(v);
        }

        header.btaf_magic = input.read_u32::<LittleEndian>()?;
        assert!(header.btaf_magic == 1178686530);
        header.btaf_section_size = input.read_u32::<LittleEndian>()?;
        header.btaf_file_count = input.read_u32::<LittleEndian>()?;
        header.btaf_offset = input.stream_position()?;

        assert!(header.btaf_section_size - 12 == header.btaf_file_count as u32 * 16);
        buf.resize(header.btaf_section_size as usize - 12, 0);
        input.read_exact(&mut buf)?;


        header.bmif_magic = input.read_u32::<LittleEndian>()?;
        header.bmif_section_size = input.read_u32::<LittleEndian>()?;
        header.bmif_data_size = input.read_u32::<LittleEndian>()?;

        buf.resize(header.bmif_section_size as usize - 12, 0);
        input.read_exact(&mut buf)?;

        assert!(header.data_offset as u64 == input.stream_position().unwrap());
        assert!(header.data_offset + header.bmif_data_size == header.file_length);

        Ok(header)
    }
}

#[derive(Clone, Debug)]
pub struct FileIteratorContext<'a, 'b> {
    file: Reader<'a>,
    file_count: u16,
    base_offset: u64,
    data_offset: u64,
    fat_offsets: &'b [u32],
}

#[derive(Debug)]
pub struct FileIterator<'a, 'b> {
    context: FileIteratorContext<'a, 'b>,
    index: u16,
}

impl<'a, 'b> FileIterator<'a, 'b> {
    pub fn file_offset(&self) -> u64 {
        self.context.base_offset + self.context.fat_offsets[self.index as usize] as u64
    }

    pub fn next(&mut self) -> Result<Option<FileEntry<'a, 'b>>, std::io::Error> {
        if self.index < self.context.file_count {
            self.context.file.seek(SeekFrom::Start(self.file_offset()))?;
            self.index += 1;

            let header = FileEntryHeader::read(&mut self.context.file)?;

            Ok(Some(FileEntry {
                context: self.context.clone(),
                index: self.index - 1,
                header,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct FileEntry<'a, 'b> {
    context: FileIteratorContext<'a, 'b>,
    index: u16,
    header: FileEntryHeader,
}

impl<'a, 'b> FileEntry<'a, 'b> {
    pub fn entries(&self) -> SubfileIterator<'a, 'b> {
        SubfileIterator {
            context: self.context.clone(),
            index: 0,
            vector: self.header.vector,
            offset: self.context.base_offset + self.context.fat_offsets[self.index as usize] as u64 + 4,
        }
    }

    pub fn index(&self) -> u16 {
        self.index
    }
}

#[derive(Debug)]
pub struct SubfileIterator<'a, 'b> {
    context: FileIteratorContext<'a, 'b>,
    vector: u32,
    index: u8,
    offset: u64,
}

impl<'a, 'b> SubfileIterator<'a, 'b> {
    pub fn next(&mut self) -> Result<Option<SubfileEntry<'a>>, std::io::Error> {
        if self.vector >> self.index == 0 {
            Ok(None)
        } else {
            while (self.vector & (1u32 << self.index)) == 0 {
                self.index += 1;
            }

             self.context.file.seek(SeekFrom::Start(self.offset))?;
             let header = SubfileEntryHeader::read(&mut self.context.file)?;

             self.index += 1;
             self.offset = self.context.file.stream_position()?;

             Ok(Some(SubfileEntry {
                 context: SubfileEntryContext {
                     file: self.context.file.clone(),
                     data_offset: self.context.data_offset,
                 },
                 header,
                 index: self.index - 1,
             }))
        }
    }
}

#[derive(Default, Debug)]
pub struct FileEntryHeader {
    vector: u32,
}

impl FileEntryHeader {
    fn read(input: &mut Reader) -> Result<Self, Error> {
        let mut entry = Self::default();

        entry.vector = input.read_u32::<LittleEndian>()?;

        Ok(entry)
    }
}

#[derive(Debug)]
struct SubfileEntryContext<'a> {
    file: Reader<'a>,
    data_offset: u64,
}

#[derive(Debug)]
pub struct SubfileEntry<'a> {
    context: SubfileEntryContext<'a>,
    header: SubfileEntryHeader,
    index: u8,
}

impl<'a> SubfileEntry<'a> {
    pub fn index(&self) -> u8 {
        self.index
    }
}

impl<'a> VirtualFile<'a> for SubfileEntry<'a> {
    fn reader(&self) -> Reader<'a> {
        self.context.file.limit(
            self.context.data_offset + self.header.start as u64,
            self.header.length as u64,
        ).unwrap()
    }
}

#[derive(Default, Debug)]
pub struct SubfileEntryHeader {
    start: u32,
    end: u32,
    length: u32,
}

impl SubfileEntryHeader {
    pub fn read(input: &mut Reader) -> Result<Self, Error> {
        let mut subentry = Self::default();

        subentry.start = input.read_u32::<LittleEndian>()?;
        subentry.end = input.read_u32::<LittleEndian>()?;
        subentry.length = input.read_u32::<LittleEndian>()?;

        // assert start+end = length?

        Ok(subentry)
    }
}
