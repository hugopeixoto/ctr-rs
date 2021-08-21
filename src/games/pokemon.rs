use super::super::*;
use super::super::read::Reader;
use super::super::read::VirtualFile;

#[derive(Debug)]
pub struct Pokemon<'a> {
    ncsd: super::super::ncsd::NCSD<'a>,
    romfs: super::super::romfs::RomFS<'a>,
    product: Game,
}

#[derive(Debug)]
pub enum Game {
    Sun,
    Moon,
    UltraSun,
    UltraMoon,
}

impl Game {
    pub fn from_product_code(code: &str) -> Option<Self> {
        match code {
            "CTR-P-BNDA" => Some(Self::Sun),
            "CTR-P-BNEA" => Some(Self::Moon),
            "CTR-P-A2AA" => Some(Self::UltraSun),
            _ => None,
        }
    }
}

pub enum Language {
    Japanese = 0,
    JapaneseHiragana = 1,
    English = 2,
    French = 3,
}

impl<'a> Pokemon<'a> {
    pub fn new(file: Reader<'a>) -> Result<Self, std::io::Error> {
        let ncsd = ncsd::NCSD::new(file)?;
        let romfs = ncsd.partition(ncsd::Partition::Main)?.romfs()?.unwrap();
        let code = ncsd.partition(ncsd::Partition::Main)?.product_code().unwrap();

        Ok(Self {
            ncsd,
            romfs,
            product: Game::from_product_code(&code).unwrap(),
        })
    }

    pub fn pokemon_names(&self, language: Language) -> Result<pokemon::text::Texts, std::io::Error> {
        self.text_entries(&format!("a/0/3/{}", language as u8), 55, 0)
    }

    pub fn pokedex_entries(&self, language: Language) -> Result<pokemon::text::Texts, std::io::Error> {
        let offset = match self.product {
            Game::Sun | Game::Moon => 119,
            Game::UltraSun | Game::UltraMoon => 124,
        };

        self.text_entries(&format!("a/0/3/{}", language as u8), offset, 0)
    }

    pub fn alt_pokedex_entries(&self, language: Language) -> Result<pokemon::text::Texts, std::io::Error> {
        let offset = match self.product {
            Game::Sun | Game::Moon => 120,
            Game::UltraSun | Game::UltraMoon => 125,
        };

        self.text_entries(&format!("a/0/3/{}", language as u8), offset, 0)
    }

    fn text_entries(&self, filename: &str, idx: usize, subidx: usize) -> Result<pokemon::text::Texts, std::io::Error> {
        let garc = match self.romfs.file_at(filename)? {
            Some(romfs::Node::File(f)) => f,
            _ => { panic!("derp"); },
        };

        let file = match garc::GARC::new(garc.reader())?.file_at(idx, subidx)? {
            Some(file) => file,
            None => { panic!("Couldn't find garc subfile"); },
        };

        pokemon::text::Texts::new(file.reader())
    }
}
