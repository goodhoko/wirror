use eyre::Result;
use std::{fs, fs::File, io::Write, path::Path};
use url::Url;

const OUTPUT_DIR: &str = "./out";

fn main() -> Result<()> {
    color_eyre::install()?;

    process_url(Url::parse("https://zadnyspe.ch/")?)?;
    Ok(())
}

fn process_url(url: Url) -> Result<()> {
    let response = reqwest::blocking::get(url)?;
    let dest = Path::new(OUTPUT_DIR).join(&response.url().path()[1..]);

    if !dest.exists() {
        fs::create_dir_all(&dest)?
    }

    let mut file = File::create(dest.join("index.html"))?;
    file.write_all(&response.bytes()?)?;

    Ok(())
}
