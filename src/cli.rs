use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};
use image::ImageFormat;
use std::path::Path;
use thiserror::Error;
use url::Url;

use favilib::errors::FavilibError;
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

        #[arg(short, long, default_value = "png")]
        format: Option<InternalImageFormat>,

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

fn main() -> Result<(), ExternalError> {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Fetch {
            url,
            size,
            format,
            path,
            url_only,
            stdout,
        }) => {
            let url = parse_url(&url)?;

            let size = size.unwrap_or(ImageSize::Default);
            let format: image::ImageFormat = format.unwrap_or(InternalImageFormat::Png).into();

            let favicon = Favicon::fetch(url, None)?;
            let favicon = favicon.resize(size);

            let path = path.clone().unwrap_or_default();

            let path = Path::new(&path);

            let target = if stdout {
                ExportTarget::Stdout
            } else {
                ExportTarget::File(Path::new(path))
            };

            match url_only {
                true => write_url(favicon.url().clone(), target)?,
                false => write_favicon(favicon, target, format)?,
            };
        }
        None => {
            eprintln!("No command provided. Use --help to see available commands.");
        }
    }

    Ok(())
}

enum ExportTarget<'a> {
    File(&'a Path),
    Stdout,
}

fn write_favicon(
    favicon: Favicon,
    target: ExportTarget,
    format: ImageFormat,
) -> Result<(), FavilibError> {
    match target {
        ExportTarget::File(path) => favicon.export(path, format),
        ExportTarget::Stdout => favicon.write_to_stdout(format),
    }
}

fn write_url(url: Url, target: ExportTarget) -> Result<(), FavilibError> {
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
fn parse_url(url: &str) -> Result<Url, FavilibError> {
    let url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };

    Ok(Url::parse(&url)?)
}

#[derive(Debug, Clone, ValueEnum)]
enum InternalImageFormat {
    Png,
    Jpeg,
    WebP,
    Ico,
}

impl From<InternalImageFormat> for image::ImageFormat {
    fn from(value: InternalImageFormat) -> Self {
        match value {
            InternalImageFormat::Png => image::ImageFormat::Png,
            InternalImageFormat::Jpeg => image::ImageFormat::Jpeg,
            InternalImageFormat::WebP => image::ImageFormat::WebP,
            InternalImageFormat::Ico => image::ImageFormat::Ico,
        }
    }
}

#[derive(Error, Debug)]
enum ExternalError {
    #[error("Invalid Url Provided")]
    InvalidUrlError,

    #[error("No favicon found for given URL")]
    NoFaviconFoundError,

    #[error("Could not write Favicons to file")]
    WriteError,
}

impl From<FavilibError> for ExternalError {
    fn from(value: FavilibError) -> Self {
        match value {
            FavilibError::UrlParseError(_) => ExternalError::InvalidUrlError,
            FavilibError::NoFaviconFoundError => ExternalError::NoFaviconFoundError,
            _ => ExternalError::WriteError,
        }
    }
}
