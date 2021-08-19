use vgc_data::*;
use vgc_data::read::VirtualFile;

fn save_to<R: std::io::Read>(reader: &mut R, filename: &str) -> Result<(), std::io::Error> {
    println!("extracting {}", filename);
    std::io::copy(reader, &mut std::fs::File::create(filename)?)?;

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let filename = &std::env::args().collect::<Vec<_>>()[1];
    let file = std::fs::File::open(filename)?;

    let rom = ncsd::NCSD::new(file)?;

    let mut it = rom.partitions();
    while let Some(partition) = it.next()? {
        let filename = format!("{}.{:#018x}", filename, partition.id());
        save_to(&mut partition.reader(), &filename)?;

        if let Some(mut region) = partition.plain_region()? {
            let filename = format!("{}.plain2", filename);
            save_to(&mut region, &filename)?;
        }

        if let Some(mut region) = partition.logo()? {
            let filename = format!("{}.logo2", filename);
            save_to(&mut region, &filename)?;
        }

        if let Some(mut region) = partition.exefs()? {
            let filename = format!("{}.exefs2", filename);
            save_to(&mut region, &filename)?;
        }

        if let Some(region) = partition.romfs()? {
            let filename = format!("{}.romfs2", filename);
            save_to(&mut region.reader(), &filename)?;
        }
    }

    Ok(())
}
