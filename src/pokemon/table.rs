use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use super::super::read::Reader;

#[derive(Debug)]
pub struct Table<'a> {
    file: Reader<'a>,
    header: Header,
}

impl<'a> Table<'a> {
    pub fn new(mut file: Reader<'a>) -> Result<Self, std::io::Error> {
        let header = Header::read(&mut file)?;

        Ok(Table { file, header })
    }

    pub fn table_count(&self) -> u16 {
        self.header.table_count
    }

    pub fn entries<'b>(&'b self) -> TableIterator<'a, 'b> {
        TableIterator {
            table: &self,
            index: 0,
        }
    }
}

pub struct TableIterator<'a, 'b> {
    table: &'b Table<'a>,
    index: u16,
}

impl<'a, 'b> Iterator for TableIterator<'a, 'b> {
    type Item = Reader<'a>;
    fn next(&mut self) -> Option<Reader<'a>> {
        if self.index == self.table.header.table_count {
            None
        } else {
            let idx = self.index as usize;
            self.index += 1;

            Some(self.table.file.limit(
                self.table.header.table_offsets[idx] as u64,
                (self.table.header.table_offsets[idx + 1] - self.table.header.table_offsets[idx]) as u64,
            ).unwrap())
        }
    }
}

#[derive(Debug, Default)]
pub struct Header {
    magic: u16,
    table_count: u16,
    table_offsets: Vec<u32>,
}

impl Header {
    pub fn read(input: &mut Reader) -> Result<Self, std::io::Error> {
        let mut header = Header::default();

        header.magic = input.read_u16::<LittleEndian>()?;
        assert!(header.magic == 0x4c42);

        header.table_count = input.read_u16::<LittleEndian>()?;
        assert!(4 + (header.table_count as u64 + 1) * 4 <= input.length());

        for i in 0..=header.table_count as usize {
            let offset = input.read_u32::<LittleEndian>()?;

            assert!((offset as u64) <= input.length());
            if i > 0 {
                assert!(header.table_offsets[i - 1] < offset);
            }

            header.table_offsets.push(offset);
        }

        Ok(header)
    }
}
