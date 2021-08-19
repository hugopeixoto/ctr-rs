use vgc_data::romfs;
use vgc_data::read::VirtualFile;

fn save_to<R: std::io::Read>(reader: &mut R, filename: &str) -> Result<(), std::io::Error> {
    println!("extracting {}", filename);
    std::io::copy(reader, &mut std::fs::File::create(filename)?)?;

    Ok(())
}

fn walkdir(mut entries: romfs::NodeIterator, filename: &String) -> Result<(), std::io::Error> {
    while let Some(entry) = entries.next()? {
        match entry {
            romfs::Node::File(file) => {
                let filename = format!("{}/{}", filename, file.basename());
                save_to(&mut file.reader(), &filename)?;
            },
            romfs::Node::Directory(dir) => {
                let filename = format!("{}/{}", filename, dir.basename());

                println!("creating {}", filename);
                std::fs::create_dir(&filename)?;

                walkdir(dir.entries(), &filename)?;
            },
        }
    }

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let filename = &std::env::args().collect::<Vec<_>>()[1];
    let file = vgc_data::read::FileHolder::open(filename)?;

    let rom = romfs::RomFS::new(file.reader())?;

    walkdir(rom.entries(), &format!("{}.dir", filename))?;

    Ok(())
}
