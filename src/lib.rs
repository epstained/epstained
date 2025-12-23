//! Epstained - Such is our republic, these days.
//!
//! Example:
//! ```rust
//! use epstained::scrape;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     scrape("Court Records", COURT_RECORDS, 3).await?;
//!     scrape("Disclosures", DISCLOSURES, 3).await?;
//!     scrape("FOIA", FOIA, 3).await?;
//!     Ok(())
//! }
//! ```

use anyhow::Error;
use html5ever::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use indicatif::{ProgressBar, ProgressStyle};
use markup5ever_rcdom::RcDom;
use rquest_util::Emulation;
use scraper::{Html, Selector};
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

const JUSTICE: &'static str = "https://www.justice.gov/";

pub async fn scrape(parent: &str, concurrency: usize) -> Result<(), Error> {
    let client = rquest::Client::builder()
        .emulation(Emulation::Firefox136)
        .cookie_store(true)
        .build()?;
    let soup = soup(&client, parent).await?;
    let links = {
        let pattern = Selector::parse("a").map_err(|a| anyhow::anyhow!("{}", a))?;
        soup.select(&pattern)
            .filter_map(|a| a.attr("href"))
            .filter(|href| href.ends_with("pdf"))
            .filter_map(|href| urlparse::unquote(href).ok())
            .collect::<Vec<String>>()
    };
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let mut tasks = JoinSet::new();

    let bar = Arc::new(ProgressBar::new(links.len() as u64).with_style({
        let name = mkname(parent)?;
        let style = format!("{name}: {{percent}} {{wide_bar}}");
        ProgressStyle::with_template(&style)?
    }));

    for link in links.into_iter() {
        let guard = Arc::clone(&semaphore);
        let bar = Arc::clone(&bar);
        let client = client.clone();

        tasks.spawn(async move {
            let _permit = guard.acquire().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            match get(&client, &link).await {
                Ok(_) => bar.inc(1),
                Err(error) => eprintln!("Error: {}", error),
            }
        });
    }

    tasks.join_all().await;

    Ok(())
}

async fn soup(client: &rquest::Client, source: &str) -> Result<Html, Error> {
    // we load it into a DOM in the hopes we get extra info (idk if it'll work, i don't do web dev D:)

    let request = client.get(source).build()?;
    let response = client.execute(request).await?;

    let raw_content = response.text().await?;

    let mut raw_cursor = Cursor::new(raw_content);
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = html5ever::parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut raw_cursor)
        .map_err(Error::msg)?;

    let handle: markup5ever_rcdom::SerializableHandle = dom.document.into();
    let mut buffer = BufWriter::new(Vec::new());
    html5ever::serialize(&mut buffer, &handle, Default::default())?;

    let contented = buffer.into_inner()?;
    let stringy = String::from_utf8(contented)?;
    Ok(Html::parse_document(&stringy))
}

async fn get(client: &rquest::Client, source: &str) -> Result<(), Error> {
    let request = client.get(source).build()?;
    let response = client.execute(request).await?;

    let path = mkdir(source)?;
    let content = response.bytes().await?;

    tokio::fs::write(path, content).await?;

    Ok(())
}

fn mkdir(str: &str) -> Result<PathBuf, Error> {
    if !str.starts_with(JUSTICE) {
        return Err(anyhow::anyhow!("Bad URL"));
    }

    let string = str.replace("https://", "");
    let components = string.split("/").collect::<Vec<_>>();
    let (path, filename) = (
        &components[0..components.len() - 1],
        &components[components.len() - 1],
    );
    let path = path.join("/");

    std::fs::create_dir_all(&path)?;

    Ok(Path::new(&path).join(filename))
}

fn mkname(str: &str) -> Result<String, Error> {
    if !str.starts_with(JUSTICE) {
        return Err(anyhow::anyhow!("Bad URL"));
    }
    let string = str.replace(JUSTICE, "");
    let parts = string.split("/").collect::<Vec<_>>();

    if parts.is_empty() {
        return Err(anyhow::anyhow!("Bad URL"));
    }

    Ok(parts[0].to_string())
}
