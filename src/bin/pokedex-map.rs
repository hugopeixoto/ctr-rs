use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use vgc_data::*;
use clap::Clap;

#[derive(Clap)]
struct Opts {
    filename: String,
    #[clap(short, long)]
    alt: bool,
    lang: String,
}


fn raw_table(mut file: read::Reader) -> Vec<u16> {
    let mut values = vec![];

    loop {
        match file.read_u16::<LittleEndian>() {
            Ok(v) => { values.push(v); },
            Err(_) => { break; }
        }
    }

    values
}

fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();

    let filename = opts.filename;
    let file = read::FileHolder::open(&filename)?;
    let game = games::pokemon::Pokemon::new(file.reader())?;

    let language = match opts.lang.as_str() {
        "ja" => games::pokemon::Language::English,
        "ja-hrkt" => games::pokemon::Language::English,
        "en" => games::pokemon::Language::English,
        "fr" => games::pokemon::Language::French,
        "it" => games::pokemon::Language::Italian,
        "de" => games::pokemon::Language::German,
        "es" => games::pokemon::Language::Spanish,
        "ko" => games::pokemon::Language::Korean,
        "zh-hans" => games::pokemon::Language::ChineseSimplified,
        "zh-hant" => games::pokemon::Language::ChineseTraditional,
        _ => { panic!(""); }
    };

    let species_names = game.species_names(language)?.entries().filter_map(Result::ok).collect::<Vec<_>>();
    let form_names = game.form_names(language)?.entries().filter_map(Result::ok).collect::<Vec<_>>();
    let dex_entries = if opts.alt { game.alt_pokedex_entries(language) } else { game.pokedex_entries(language) }?.entries().filter_map(Result::ok).collect::<Vec<_>>();

    let pokedex_tables = game.form_linked_list()?;
    let forms = pokedex_tables.entries().nth(0).unwrap();
    let forms = raw_table(forms);

    for i in 0..species_names.len() {
        let mut form = i;
        let mut form_idx = 0;
        while form != 0 {
            println!("{},{},{:?},{:?}", i, form_idx, form_names[form], dex_entries[form]);

            form = forms[form] as usize;
            form_idx += 1;
        }
    }

    Ok(())
}
