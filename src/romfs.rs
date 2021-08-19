use byteorder::LittleEndian;
use byteorder::ByteOrder;
use byteorder::ReadBytesExt;
use std::io::SeekFrom;
use std::io::Seek;
use std::io::Read;
use super::read::Reader;

#[derive(Debug)]
pub struct RomFS<'a> {
    file: Reader<'a>,
    header: Header,
    lvl3_header: Level3Header,
}

impl<'a> RomFS<'a> {
    pub fn new(mut file: Reader<'a>) -> Result<RomFS, std::io::Error> {
        let header = Header::read(&mut file)?;

        let lvl3_header_offset = align(header.master_hash_size as u64 + align(header.header_length as u64, 0x10), header.level1_block_size);
        file.seek(SeekFrom::Start(lvl3_header_offset))?;

        let mut lvl3_header = Level3Header::read(&mut file)?;
        lvl3_header.header_offset = lvl3_header_offset;

        Ok(RomFS { file, header, lvl3_header })
    }

    pub fn entries(&self) -> NodeIterator {
        NodeIterator {
            context: NodeIteratorContext {
                file: self.file.clone(),
                directory_base_offset: self.lvl3_header.header_offset + self.lvl3_header.directory_metadata_table.offset as u64,
                file_base_offset: self.lvl3_header.header_offset + self.lvl3_header.file_metadata_table.offset as u64,
            },
            next_directory: Some(0),
            next_file: None,
        }
    }
}

impl<'a> super::read::VirtualFile<'a> for RomFS<'a> {
    fn reader(&self) -> Reader<'a> {
        self.file.at_zero()
    }
}

#[derive(Debug, Default)]
struct Header {
    magic: [u8; 4],
    bom: u32,
    master_hash_size: u32,
    level1_logical_offset: u64,
    level1_hashdata_size: u64,
    level1_block_size: u64,
    reserved0: u32,
    level2_logical_offset: u64,
    level2_hashdata_size: u64,
    level2_block_size: u64,
    reserved1: u32,
    level3_logical_offset: u64,
    level3_hashdata_size: u64,
    level3_block_size: u64,
    reserved2: u32,
    header_length: u32,
    reserved3: u32,
}

impl Header {
    pub fn read(input: &mut Reader) -> Result<Self, std::io::Error> {
        let mut header = Self::default();

        input.read_exact(&mut header.magic)?;
        assert!(header.magic == *b"IVFC");

        header.bom = input.read_u32::<LittleEndian>()?;
        assert!(header.bom == 0x10000);
        header.master_hash_size = input.read_u32::<LittleEndian>()?;

        header.level1_logical_offset = input.read_u64::<LittleEndian>()?;
        header.level1_hashdata_size = input.read_u64::<LittleEndian>()?;
        header.level1_block_size = 2u64.pow(input.read_u32::<LittleEndian>()?);
        header.reserved0 = input.read_u32::<LittleEndian>()?;

        header.level2_logical_offset = input.read_u64::<LittleEndian>()?;
        header.level2_hashdata_size = input.read_u64::<LittleEndian>()?;
        header.level2_block_size = 2u64.pow(input.read_u32::<LittleEndian>()?);
        header.reserved1 = input.read_u32::<LittleEndian>()?;

        header.level3_logical_offset = input.read_u64::<LittleEndian>()?;
        header.level3_hashdata_size = input.read_u64::<LittleEndian>()?;
        header.level3_block_size = 2u64.pow(input.read_u32::<LittleEndian>()?);
        header.reserved2 = input.read_u32::<LittleEndian>()?;
        header.header_length = input.read_u32::<LittleEndian>()?;
        header.reserved3 = input.read_u32::<LittleEndian>()?;

        Ok(header)
    }
}


#[derive(Debug)]
pub enum Node<'a> {
    File(FileMetadata<'a>),
    Directory(DirectoryMetadata<'a>),
}

#[derive(Clone, Debug)]
pub struct NodeIteratorContext<'a> {
    file: Reader<'a>,
    directory_base_offset: u64,
    file_base_offset: u64,
}

pub struct NodeIterator<'a> {
    context: NodeIteratorContext<'a>,
    next_directory: Option<u32>,
    next_file: Option<u32>,
}

impl<'a> NodeIterator<'a> {
    fn directory_offset(&self) -> Option<u64> {
        self.next_directory.map(|offset| self.context.directory_base_offset + offset as u64)
    }

    fn file_offset(&self) -> Option<u64> {
        self.next_file.map(|offset| self.context.file_base_offset + offset as u64)
    }

    pub fn next(&mut self) -> Result<Option<Node>, std::io::Error> {
        match self.directory_offset() {
            Some(offset) => {
                match Self::read_directory(&self.context, offset) {
                    Ok(dm) => {
                        self.next_directory = dm.header.next_directory;
                        Ok(Some(Node::Directory(dm)))
                    },
                    Err(e) => Err(e),
                }
            },
            None => match self.file_offset() {
                Some(offset) => {
                    match Self::read_file(&self.context, offset) {
                        Ok(fm) => {
                            self.next_file = fm.header.next_file;
                            Ok(Some(Node::File(fm)))
                        },
                        Err(e) => Err(e),
                    }
                },
                None => Ok(None),
            }
        }
    }

    fn read_directory<'b>(context: &NodeIteratorContext<'b>, offset: u64) -> Result<DirectoryMetadata<'b>, std::io::Error> {
        let mut file = context.file.clone();
        file.seek(std::io::SeekFrom::Start(offset))?;

        let header = DirectoryMetadataHeader::read(&mut file)?;

        Ok(DirectoryMetadata { context: context.clone(), header })
    }

    pub fn read_file<'b>(context: &NodeIteratorContext<'b>, offset: u64) -> Result<FileMetadata<'b>, std::io::Error> {
        let mut file = context.file.clone();
        file.seek(std::io::SeekFrom::Start(offset))?;

        let header = FileMetadataHeader::read(&mut file)?;

        Ok(FileMetadata { context: context.clone(), header })
    }
}


#[derive(Default, Debug)]
pub struct Level3HeaderSection {
    offset: u32,
    length: u32,
}

#[derive(Default, Debug)]
pub struct Level3Header {
    pub header_offset: u64,
    header_length: u32,
    directory_hash_table: Level3HeaderSection,
    directory_metadata_table: Level3HeaderSection,
    file_hash_table: Level3HeaderSection,
    file_metadata_table: Level3HeaderSection,
    pub file_data_offset: u32,
}

impl Level3Header {
    pub fn read(input: &mut Reader) -> Result<Self, std::io::Error> {
        let mut lvl3 = Self::default();

        lvl3.header_length = input.read_u32::<LittleEndian>()?;
        lvl3.directory_hash_table.offset = input.read_u32::<LittleEndian>()?;
        lvl3.directory_hash_table.length = input.read_u32::<LittleEndian>()?;
        lvl3.directory_metadata_table.offset = input.read_u32::<LittleEndian>()?;
        lvl3.directory_metadata_table.length = input.read_u32::<LittleEndian>()?;
        lvl3.file_hash_table.offset = input.read_u32::<LittleEndian>()?;
        lvl3.file_hash_table.length = input.read_u32::<LittleEndian>()?;
        lvl3.file_metadata_table.offset = input.read_u32::<LittleEndian>()?;
        lvl3.file_metadata_table.length = input.read_u32::<LittleEndian>()?;
        lvl3.file_data_offset = input.read_u32::<LittleEndian>()?;

        Ok(lvl3)
    }
}

fn align(offset: u64, block_size: u64) -> u64 {
    if offset % block_size == 0 {
        offset
    } else {
        offset + (block_size - (offset % block_size))
    }
}

#[derive(Debug)]
pub struct DirectoryMetadata<'a> {
    context: NodeIteratorContext<'a>,
    header: DirectoryMetadataHeader,
}

impl<'a> DirectoryMetadata<'a> {
    pub fn basename(&self) -> &String {
        &self.header.basename
    }

    pub fn entries(&self) -> NodeIterator {
        NodeIterator {
            context: self.context.clone(),
            next_directory: self.header.first_subdirectory,
            next_file: self.header.first_file,
        }
    }
}


#[derive(Debug, Default)]
pub struct DirectoryMetadataHeader {
    parent: u32,
    next_directory: Option<u32>,
    first_subdirectory: Option<u32>,
    first_file: Option<u32>,
    hash_next_directory: Option<u32>,
    name_length: u32,
    pub basename: String,
}

impl DirectoryMetadataHeader {
    fn read(input: &mut Reader) -> Result<Self, std::io::Error> {
        let mut header = DirectoryMetadataHeader::default();

        header.parent = input.read_u32::<LittleEndian>()?;
        header.next_directory = read_option_u32(input)?;
        header.first_subdirectory = read_option_u32(input)?;
        header.first_file = read_option_u32(input)?;
        header.hash_next_directory = read_option_u32(input)?;
        header.name_length = input.read_u32::<LittleEndian>()?;

        let mut buffer = vec![];
        buffer.resize(header.name_length as usize, 0);
        input.read_exact(&mut buffer)?;

        header.basename = read_le16_string(&buffer)?;

        Ok(header)
    }
}

pub fn read_option_u32(input: &mut dyn std::io::Read) -> Result<Option<u32>, std::io::Error> {
    let offset = input.read_u32::<LittleEndian>()?;

    if offset == 0xFFFFFFFF {
        Ok(None)
    } else {
        Ok(Some(offset))
    }
}

pub fn read_le16_string(input: &[u8]) -> Result<String, std::io::Error> {
    let mut s = String::new();

    for i in (0..input.len()).step_by(2) {
        s.push(char::from_u32(LittleEndian::read_u16(&input[i..i+2]) as u32).unwrap());
    }

    Ok(s)
}

#[derive(Debug)]
pub struct FileMetadata<'a> {
    context: NodeIteratorContext<'a>,
    header: FileMetadataHeader,
}

impl<'a> FileMetadata<'a> {
    pub fn basename(&self) -> &String {
        &self.header.basename
    }
}


#[derive(Debug, Default)]
pub struct FileMetadataHeader {
    parent: u32,
    next_file: Option<u32>,
    pub file_data_offset: u64,
    pub file_data_length: u64,
    hash_next_file: Option<u32>,
    name_length: u32,
    pub basename: String,
}

impl FileMetadataHeader {
    pub fn read(input: &mut Reader) -> Result<Self, std::io::Error> {
        let mut header = FileMetadataHeader::default();

        header.parent = input.read_u32::<LittleEndian>()?;
        header.next_file = read_option_u32(input)?;
        header.file_data_offset = input.read_u64::<LittleEndian>()?;
        header.file_data_length = input.read_u64::<LittleEndian>()?;
        header.hash_next_file = read_option_u32(input)?;
        header.name_length = input.read_u32::<LittleEndian>()?;

        let mut buffer = vec![];
        buffer.resize(header.name_length as usize, 0);
        input.read_exact(&mut buffer)?;

        header.basename = read_le16_string(&buffer)?;

        Ok(header)
    }
}
