use vgc_data::*;

fn walkdir(mut entries: romfs::NodeIterator, depth: usize) -> Result<(), std::io::Error> {
    let indent = "  ".repeat(depth);

    while let Some(f) = entries.next()? {
        match f {
            romfs::Node::File(file) => {
                println!("{}{}", indent, file.basename());
            },
            romfs::Node::Directory(dir) => {
                println!("{}{}/", indent, dir.basename());
                walkdir(dir.entries(), depth + 1)?;
            },
        }
    }

    Ok(())
}

pub fn main() -> Result<(), std::io::Error> {
    let file = std::fs::File::open("pokemon-sun.3ds")?;

    let rom = ncsd::NCSD::new(file)?;
    let p0 = rom.partition(ncsd::Partition::Main)?;
    let romfs = p0.romfs()?.unwrap();

    walkdir(romfs.entries(), 0)?;

    Ok(())
}