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
    println!("Processing {}", url);
    let response = reqwest::blocking::get(url)?;
    let url = response.url().clone();

    let content = response.text()?;
    let dom = tl::parse(&content, tl::ParserOptions::default())?;
    for node in dom.nodes().iter() {
        match node {
            tl::Node::Tag(tag) if tag.name() == "a" => {
                let href = match tag.attributes().get("href") {
                    None => continue,
                    Some(None) => continue,
                    Some(Some(href)) => href,
                };

                println!("about to process {:?}", href);
            }
            _ => {}
        };
    }

    let dest = Path::new(OUTPUT_DIR).join(&url.path()[1..]);
    if !dest.exists() {
        fs::create_dir_all(&dest)?
    }

    let mut file = File::create(dest.join("index.html"))?;
    file.write_all(content.as_bytes())?;

    Ok(())
}
