use vgc_data::*;
use vgc_data::read::VirtualFile;

fn save_to<R: std::io::Read>(reader: &mut R, filename: &str) -> Result<(), std::io::Error> {
    println!("extracting {}", filename);
    std::io::copy(reader, &mut std::fs::File::create(filename)?)?;

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let filename = &std::env::args().collect::<Vec<_>>()[1];
    let file = vgc_data::read::FileHolder::open(filename)?;

    let garc = garc::GARC::new(file.reader())?;
    let width = (garc.file_count() as f64).log10().ceil() as usize;

    println!("file size: {:#?}", garc.reader().length());

    let mut it = garc.entries();
    while let Some(entry) = it.try_next()? {
        let mut jt = entry.entries();
        while let Some(subentry) = jt.try_next()? {
            let filename = format!("{}.{:0width$}.{02}", filename, entry.index(), subentry.index(), width = width);

            save_to(&mut subentry.reader(), &filename)?;
        }
    }

    Ok(())
}
