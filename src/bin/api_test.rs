use vgc_data::*;
use vgc_data::read::VirtualFile;

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

// Test function to ensure that every object's lifetime is not bound to its parent,
// but to the root reader.
fn lifetime_guarantee<'a, 'b>(rom: &'b ncsd::NCSD<'a>) -> Result<garc::GARC<'a>, std::io::Error> {
    let p0 = rom.partition(ncsd::Partition::Main)?;
    let romfs = p0.romfs()?.unwrap();
    let file = romfs.file_at("a/1/5/2")?;

    let file = match file {
        Some(romfs::Node::File(f)) => f,
        None => { panic!("Couldn't find file"); },
        _ => { panic!("Entry is a directory, not a file"); },
    };

    garc::GARC::new(file.reader())
}

pub fn main() -> Result<(), std::io::Error> {
    let file = read::FileHolder::open("pokemon-sun.3ds")?;
    let rom = ncsd::NCSD::new(file.reader())?;

    let garc = lifetime_guarantee(&rom)?;

    println!("{:?}", garc);

    Ok(())
}
