use vgc_data::*;
use clap::Clap;

#[derive(Clap)]
struct Opts {
    filename: String,
    #[clap(short, long)]
    alt: bool,
}

pub fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();

    let file = read::FileHolder::open(&opts.filename)?;
    let rom = ncsd::NCSD::new(file.reader())?;

    eprintln!("{:?}", rom.partition(ncsd::Partition::Main)?.product_code());

    let game = games::pokemon::Pokemon::new(file.reader())?;
    let names = if opts.alt {
        game.alt_pokedex_entries(games::pokemon::Language::English)?
    } else {
        game.pokedex_entries(games::pokemon::Language::English)?
    };

    let mut it = names.entries();
    while let Some(text) = it.next()? {
        println!("{:?}", text);
    }


    Ok(())
}
