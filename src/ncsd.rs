use super::ncch::NCCH;
use super::read::Reader;
use std::io::Read;
use byteorder::ReadBytesExt;
use byteorder::LittleEndian;

fn expect<F>(good: bool, f: F) -> Result<(), std::io::Error> where F: Fn() -> String {
    if good {
        Ok(())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, f()))
    }
}

#[derive(Debug)]
pub struct NCSD<'a> {
    file: Reader<'a>,
    header: Header,
}

pub enum Partition {
    Main,
    Manual,
    DownloadPlay,
    New3DSUpdateData,
    UpdateData,
    Index(usize),
}

impl<'a> NCSD<'a> {
    pub fn new(mut file: Reader<'a>) -> Result<NCSD, std::io::Error> {
        let header = Header::read(&mut file)?;

        Ok(NCSD { file, header })
    }

    pub fn partition(&self, p: Partition) -> Result<NCCH<'a>, std::io::Error> {
        match p {
            Partition::Main => self.partition(Partition::Index(0)),
            Partition::Manual => self.partition(Partition::Index(1)),
            Partition::DownloadPlay => self.partition(Partition::Index(2)),
            Partition::New3DSUpdateData => self.partition(Partition::Index(6)),
            Partition::UpdateData => self.partition(Partition::Index(7)),
            Partition::Index(index) => {
                if index >= 8 {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Partition number out of bounds"));
                }

                if self.header.partition_offset(index) == 0 {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Partition is empty"));
                }

                NCCH::new(self.file.limit(self.header.partition_offset(index), self.header.partition_length(index))?)
            },
        }
    }

    pub fn partitions(&self) -> PartitionIterator {
        PartitionIterator {
            file: self.file.clone(),
            header: &self.header,
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct PartitionIterator<'a> {
    file: Reader<'a>,
    header: &'a Header,
    index: usize,
}

impl<'a> PartitionIterator<'a> {
    pub fn next(&mut self) -> Result<Option<NCCH>, std::io::Error> {
        if self.index < 8 {
            while self.header.partition_offsets[self.index] == 0 {
                self.index += 1;
            }

            let partition = NCCH::new(self.file.limit(self.header.partition_offset(self.index), self.header.partition_length(self.index))?).map(Option::Some);

            self.index += 1;

            partition
        } else {
            Ok(None)
        }
    }
}

#[derive(Default, Debug)]
pub struct Header {
    signature: Vec<u8>, // should be [u8; 0x100] but that doesn't Default :x
    magic: [u8; 4],
    size: u64,
    media_id: [u8; 8],
    partition_fs_type: [u8; 8],
    partition_crypt_type: [u8; 8],
    partition_offsets: [u32; 8],
    partition_lengths: [u32; 8],

    exheader_signature: [u8; 0x20],
    additional_header_size: u32,
    sector_zero_offset: u32,
    partition_flags: [u8; 8],
    partition_ids: [u64; 8],
    unknown0: [u8; 0x20],
    unknown1: [u8; 0xE],
    unknown2: u8,
    unknown3: u8,
}

impl Header {
    fn partition_offset(&self, index: usize) -> u64 {
        self.partition_offsets[index] as u64 * 0x200
    }

    fn partition_length(&self, index: usize) -> u64 {
        self.partition_lengths[index] as u64 * 0x200
    }


    fn read(input: &mut Reader) -> Result<Header, std::io::Error> {
        let mut header = Header::default();
        header.signature.resize(0x100, 0);

        input.read_exact(&mut header.signature)?;
        input.read_exact(&mut header.magic)?;

        expect(header.magic == *b"NCSD", || format!("magic number not NCSD: {:?}", header.magic))?;

        header.size = input.read_u32::<LittleEndian>()? as u64 * 0x200;

        input.read_exact(&mut header.media_id)?;
        input.read_exact(&mut header.partition_fs_type)?;
        input.read_exact(&mut header.partition_crypt_type)?;

        for i in 0..8 {
            header.partition_offsets[i] = input.read_u32::<LittleEndian>()?;
            header.partition_lengths[i] = input.read_u32::<LittleEndian>()?;
        }

        input.read_exact(&mut header.exheader_signature)?;
        header.additional_header_size = input.read_u32::<LittleEndian>()?;
        header.sector_zero_offset = input.read_u32::<LittleEndian>()?;

        input.read_exact(&mut header.partition_flags)?;

        for i in 0..8 {
            header.partition_ids[i] = input.read_u64::<LittleEndian>()?;
        }

        input.read_exact(&mut header.unknown0)?;
        input.read_exact(&mut header.unknown1)?;
        header.unknown2 = input.read_u8()?;
        header.unknown3 = input.read_u8()?;

        Ok(header)
    }
}
