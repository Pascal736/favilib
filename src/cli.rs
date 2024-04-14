use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use image::ImageFormat;
use std::path::Path;
use url::Url;

use favilib::Favicon;
use favilib::ImageSize;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Fetches favicons from websites.
    Fetch {
        /// URL of the website. If schema is omitted, https will be used.
        url: String,

        #[arg(short, long, default_value = "default")]
        size: Option<ImageSize>,

        /// Path to save the favicon
        #[arg(short, long, required_unless_present = "stdout")]
        path: Option<String>,

        /// Set this flag to only print the URL of the favicon
        #[arg(long)]
        url_only: bool,

        /// Set this flag to only write the favicon bytes to stdout. Mutually exclusive with `path`.
        #[arg(long, required_unless_present = "path")]
        stdout: bool,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Fetch {
            url,
            size,
            path,
            url_only,
            stdout,
        }) => {
            let url = parse_url(&url).expect("Failed to parse URL");

            let size = size.expect("Failed to parse size");

            let favicon = Favicon::fetch(url, None).expect("Failed to fetch favicon");
            let favicon = favicon.resize(size);

            let path = path.clone().unwrap_or(Default::default());

            let path = Path::new(&path);

            let target = if stdout {
                ExportTarget::Stdout
            } else {
                ExportTarget::File(Path::new(path))
            };

            match url_only {
                true => write_url(favicon.url().clone(), target).expect("Failed to write URL"),
                false => write_favicon(favicon, target, ImageFormat::Png)
                    .expect("Failed to write favicon"),
            };
        }
        None => {
            eprintln!("No command provided. Use --help to see available commands.");
        }
    }
}

enum ExportTarget<'a> {
    File(&'a Path),
    Stdout,
}

fn write_favicon(favicon: Favicon, target: ExportTarget, format: ImageFormat) -> Result<()> {
    match target {
        ExportTarget::File(path) => favicon.export(path, format),
        ExportTarget::Stdout => favicon.write_to_stdout(format),
    }
}

fn write_url(url: Url, target: ExportTarget) -> Result<()> {
    match target {
        ExportTarget::File(path) => {
            std::fs::write(path, url.as_str())?;
        }
        ExportTarget::Stdout => {
            println!("{}", url);
        }
    };
    Ok(())
}

/// Parses a URL string into a `Url` struct.
/// If scheme is missing adds https as scheme.
fn parse_url(url: &str) -> Result<Url> {
    let url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };

    Url::parse(&url).map_err(|_| anyhow!("Failed to parse URL"))
}
