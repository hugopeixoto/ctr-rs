use vgc_data::*;
use vgc_data::read::VirtualFile;

fn check_garc(garc: &garc::GARC) -> Result<(), std::io::Error> {
    let mut it = garc.entries();
    while let Some(entry) = it.try_next()? {
        let mut jt = entry.entries();
        while let Some(subentry) = jt.try_next()? {

            //println!("  {}.{}", entry.index(), subentry.index());
            if let Ok(table) = pokemon::table::Table::new(subentry.reader()) {
                println!("    BL: {} entries", table.len());

                for subsubentry in table.entries() {
                    println!("      {}", subsubentry.length());
                }
            }
        }
    }

    Ok(())
}

fn walkdir(mut entries: romfs::NodeIterator, filename: &str) -> Result<(), std::io::Error> {
    while let Some(entry) = entries.next()? {
        match entry {
            romfs::Node::File(file) => {
                if let Ok(garc) = garc::GARC::new(file.reader()) {
                    //println!("garc: {}/{}", filename, file.basename());
                    check_garc(&garc)?;
                }
            },
            romfs::Node::Directory(dir) => {
                let filename = format!("{}/{}", filename, dir.basename());
                walkdir(dir.entries(), &filename)?;
            },
        }
    }

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let filename = std::env::args().nth(1).unwrap();
    let file = read::FileHolder::open(&filename)?;
    let ncsd = ncsd::NCSD::new(file.reader())?;
    let romfs = ncsd.partition(ncsd::Partition::Main)?.romfs()?.unwrap();

    let game = games::pokemon::Pokemon::new(file.reader())?;
    let language = games::pokemon::Language::English;
    let species_names = game.species_names(language)?.entries().filter_map(Result::ok).collect::<Vec<_>>();

    //walkdir(romfs.entries(), "")?;

    let file = romfs.file_at("a/1/5/6")?.unwrap();
    let file = if let romfs::Node::File(f) = file { f } else { panic!(""); };
    let garc = garc::GARC::new(file.reader())?;
    let table = pokemon::table::Table::new(garc.file_at(1, 0)?.unwrap().reader())?;
    let entries = table.entries().collect::<Vec<_>>();

    println!("tables: {}", table.len());

    // table 0: form linked list
    // table 1: form shape
    // table 2: species regional dex number
    // table 3: species melemele dex number
    // table 4: species akala dex number
    // table 5: species ula'ula dex number
    // table 6: species poni dex number

    println!("table size: {}", entries[7].length());

    //let values = raw_table_u16(entries[4].clone());

    //for (i, v) in values.iter().enumerate() {
    //    println!("{} {}", v, species_names[i]);
    //}


    Ok(())
}

use std::io::Read;
fn raw_table_u8(mut file: read::Reader) -> Vec<u8> {
    let mut values = vec![];

    loop {
        let mut v = [0u8; 1];
        match file.read_exact(&mut v) {
            Ok(_) => { values.push(v[0]); },
            Err(_) => { break; }
        }
    }

    values
}

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
fn raw_table_u16(mut file: read::Reader) -> Vec<u16> {
    let mut values = vec![];

    loop {
        match file.read_u16::<LittleEndian>() {
            Ok(v) => { values.push(v); },
            Err(_) => { break; }
        }
    }

    values
}


