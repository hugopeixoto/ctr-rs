use super::romfs::RomFS;
use super::read::Reader;
use super::read::VirtualFile;
use std::io::Read;
use byteorder::ReadBytesExt;
use byteorder::LittleEndian;

#[derive(Debug)]
pub struct NCCH<'a> {
    file: Reader<'a>,
    header: Header,
}

impl<'a> NCCH<'a> {
    pub fn new(mut file: Reader<'a>) -> Result<NCCH, std::io::Error> {
        let header = Header::read(&mut file)?;

        Ok(NCCH { file, header })
    }

    pub fn romfs(&self) -> Result<RomFS, std::io::Error> {
        RomFS::new(self.file.limit(self.header.romfs_offset, self.header.romfs_size)?)
    }

    pub fn id(&self) -> u64 {
        self.header.partition_id
    }
}

impl<'a> VirtualFile<'a> for NCCH<'a> {
    fn reader(&self) -> Reader<'a> {
        self.file.clone()
    }
}

#[derive(Default, Debug)]
struct Header {
    signature: Vec<u8>, // should be [u8; 0x100] but that doesn't Default :x
    magic: [u8; 4],
    size: u64,
    partition_id: u64,
    maker_code: [u8; 2],
    version: [u8; 2],
    content_lock_check: [u8; 4],
    program_id: u64,
    reserved0: [u8; 0x10],
    logo_region_sha256: [u8; 0x20],
    product_code: [u8; 0x10],
    exheader_sha256: [u8; 0x20],
    exheader_size: u64,
    reserved1: [u8; 4],
    flags: [u8; 8],
    plain_region_offset: u64,
    plain_region_size: u64,
    logo_region_offset: u64,
    logo_region_size: u64,
    exefs_offset: u64,
    exefs_size: u64,
    exefs_hash_size: u64,
    reserved2: [u8; 4],
    romfs_offset: u64,
    romfs_size: u64,
    romfs_hash_size: u64,
    reserved3: [u8; 4],
    exefs_superblock_sha256: [u8; 0x20],
    romfs_superblock_sha256: [u8; 0x20],
}

impl Header {
    fn read(input: &mut Reader) -> Result<Header, std::io::Error> {
        let mut header = Header::default();
        header.signature.resize(0x100, 0);

        input.read_exact(&mut header.signature)?;
        input.read_exact(&mut header.magic)?;

        assert!(header.magic == *b"NCCH");

        header.size = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        header.partition_id = input.read_u64::<LittleEndian>()?;
        input.read_exact(&mut header.maker_code)?;
        input.read_exact(&mut header.version)?;
        input.read_exact(&mut header.content_lock_check)?;
        header.program_id = input.read_u64::<LittleEndian>()?;
        input.read_exact(&mut header.reserved0)?;
        input.read_exact(&mut header.logo_region_sha256)?;
        input.read_exact(&mut header.product_code)?;
        input.read_exact(&mut header.exheader_sha256)?;
        header.exheader_size = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        input.read_exact(&mut header.reserved1)?;
        input.read_exact(&mut header.flags)?;

        header.plain_region_offset = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        header.plain_region_size = input.read_u32::<LittleEndian>()? as u64 * 0x200;

        header.logo_region_offset = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        header.logo_region_size = input.read_u32::<LittleEndian>()? as u64 * 0x200;

        header.exefs_offset = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        header.exefs_size = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        header.exefs_hash_size = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        input.read_exact(&mut header.reserved2)?;

        header.romfs_offset = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        header.romfs_size = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        header.romfs_hash_size = input.read_u32::<LittleEndian>()? as u64 * 0x200;
        input.read_exact(&mut header.reserved3)?;

        input.read_exact(&mut header.exefs_superblock_sha256)?;
        input.read_exact(&mut header.romfs_superblock_sha256)?;

        Ok(header)
    }
}
