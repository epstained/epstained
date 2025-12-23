use clap::Parser;
use epstain::scrape;

#[derive(clap::Parser)]
struct Args {
    #[clap(short, long, help = "A doj-disclosures resource url")]
    url: String,
    #[clap(
        short,
        long,
        default_value_t = 3,
        help = "The number of files to download at one time"
    )]
    concurrency: usize,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    scrape(&args.url, args.concurrency).await
}
