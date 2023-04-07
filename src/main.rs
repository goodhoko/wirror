use eyre::{eyre, Result};
use reqwest::{blocking::Response, header::CONTENT_TYPE};
use std::{
    collections::HashSet,
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    str,
};
use url::Url;

const OUTPUT_DIR: &str = "./out";

struct State {
    urls_in_process: HashSet<Url>,
    origin: Url,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let url = Url::parse("http://localhost:4000/")?;
    process_url(
        url.clone(),
        &mut State {
            urls_in_process: HashSet::new(),
            origin: url,
        },
    )?;

    Ok(())
}

fn process_url(url: Url, state: &mut State) -> Result<()> {
    match should_descent(&url, state) {
        ShouldDescend::No(reason) => {
            println!("skipping {url}: {reason}");
            return Ok(());
        }
        ShouldDescend::Yes => {}
    }

    println!("processing {url}");

    let response = reqwest::blocking::get(url.clone())?;
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .ok_or(eyre!("cannot get content type"))?
        .to_str()?;

    // Get the final URL after redirects
    let final_url = response.url().clone();

    // Mark both urls as being processed before descending down
    state.urls_in_process.insert(url);
    state.urls_in_process.insert(final_url.clone());

    let content = match content_type {
        _ if content_type.starts_with("text/html") => process_html(response, state),
        _ => process_file(response, state),
    }?;

    // Persist the file to disk
    let path = url_to_path(&final_url);
    fs::create_dir_all(path.parent().expect("Parent directory exists"))?;
    let mut file = File::create(path)?;
    file.write_all(&content)?;

    Ok(())
}

fn process_html(response: Response, state: &mut State) -> Result<Vec<u8>> {
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

                let Ok(href) = str::from_utf8(href.as_bytes()) else {
                    println!("skipping {:?}: can't decode it as utf8", href);
                    continue;
                };

                let Ok(resolved_href) = url.join(href) else {
                    println!("skipping {}: can't join it onto {}", href, url);
                    continue;
                };

                if let Err(err) = process_url(resolved_href.clone(), state) {
                    println!("skipping {}: failed to process: {}", resolved_href, err);
                    continue;
                }
            }
            _ => {}
        };
    }

    Ok(content.clone().into_bytes())
}

fn process_file(_response: Response, _state: &mut State) -> Result<Vec<u8>> {
    Err(eyre!("Not implemented"))
}

fn url_to_path(url: &Url) -> PathBuf {
    // TODO: distinguish HTML files that go into ".../name/index.html" and all other files that go into ".../name"
    Path::new(OUTPUT_DIR)
        .join(&url.path()[1..])
        .join("index.html")
}

enum ShouldDescend {
    No(&'static str),
    Yes,
}

fn should_descent(url: &Url, state: &State) -> ShouldDescend {
    if state.urls_in_process.contains(url) {
        return ShouldDescend::No("already being processed");
    }

    let host = match url.host() {
        Some(h) => h,
        None => return ShouldDescend::No("url with no host"),
    };
    let current_host = state
        .origin
        .host()
        .expect("existence of host was enforced before");

    if host != current_host {
        return ShouldDescend::No("different host");
    }

    ShouldDescend::Yes
}
