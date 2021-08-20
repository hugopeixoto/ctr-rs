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

    pub fn entries<'b>(&'b self) -> TableIterator2<'a, 'b> {
        TableIterator2 {
            table: &self,
            index: 0,
        }
    }
}

pub struct TableIterator2<'a, 'b> {
    table: &'b Table<'a>,
    index: u16,
}

impl<'a, 'b> Iterator for TableIterator2<'a, 'b> {
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

        header.table_offsets.push(input.read_u32::<LittleEndian>()?);

        for _ in 0..header.table_count {
            let offset = input.read_u32::<LittleEndian>()?;
            assert!(*header.table_offsets.last().unwrap() < offset);

            header.table_offsets.push(offset);
        }

        Ok(header)
    }

    pub fn table_iter<T>(&self, index: usize) -> Option<TableIterator<T>> {
        if self.table_count as usize <= index {
            None
        } else {
            Some(TableIterator {
                base_offset: self.table_offsets[index],
                index: 0,
                n: (self.table_offsets[index + 1] - self.table_offsets[index]) / std::mem::size_of::<T>() as u32,
                phantom: std::marker::PhantomData,
            })
        }
    }
}

pub struct TableIterator<T> {
    base_offset: u32,
    index: u32,
    n: u32,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Iterator for TableIterator<T> {
    type Item = TableItemOffset<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.n {
            self.index += 1;
            Some(TableItemOffset::<T> {
                offset: self.base_offset + (self.index - 1) * std::mem::size_of::<T>() as u32,
                phantom: std::marker::PhantomData,
            })
        } else {
            None
        }
    }
}

pub struct TableItemOffset<T> {
    offset: u32,
    phantom: std::marker::PhantomData<T>,
}

impl TableItemOffset<u16> {
    pub fn value<R: std::io::Read + std::io::Seek>(&self, input: &mut R) -> Result<u16, std::io::Error> {
        input.seek(std::io::SeekFrom::Start(self.offset as u64))?;
        input.read_u16::<LittleEndian>()
    }
}

