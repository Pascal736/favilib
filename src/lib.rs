use anyhow::{Context as _, Result};
use image::{imageops::FilterType, io::Reader as ImageReader};
pub use image::{DynamicImage, ImageFormat};
pub use reqwest::blocking::Client;
use std::io::{self, Cursor, Write as _};
use std::path::Path;
pub use url::Url;

use errors::FavilibError;

pub mod errors;
mod scraper;

#[derive(Debug, Clone)]
pub struct Favicon {
    url: Url,
    bytes: Vec<u8>,
    image: DynamicImage,
}

impl Favicon {
    /// Fetches a favicon from a URL and returns a new Favicon instance.
    /// The fetching algorithm selects the first valid favicon found on the page.
    /// Custom client can be passed to the function. If omitted, a new client will be created.
    pub fn fetch(url: Url, client: Option<Client>) -> Result<Self, FavilibError> {
        let client = client.unwrap_or(Client::new());
        Ok(scraper::fetch_and_validate_favicon(url.clone(), &client)?)
    }

    /// Builds a new Favicon instance from a URL and a byte vector.
    /// Does not fetch the image from the URL.
    /// Use the fetch function to fetch the image.
    pub fn build(url: Url, bytes: Vec<u8>) -> Result<Self, FavilibError> {
        let image = ImageReader::new(Cursor::new(bytes.clone()))
            .with_guessed_format()
            .map(|img| img.decode())
            .map_err(|_| FavilibError::NoFaviconFoundError)??;

        Ok(Self { url, bytes, image })
    }

    /// Crates a new instance with changed image size and image bytes.
    pub fn resize(self, size: ImageSize) -> Favicon {
        let img = self.image;
        let img = match size {
            ImageSize::Small => img.resize_to_fill(16, 16, FilterType::Lanczos3),
            ImageSize::Medium => img.resize_to_fill(32, 32, FilterType::Lanczos3),
            ImageSize::Large => img.resize_to_fill(64, 64, FilterType::Lanczos3),
            ImageSize::Custom(width, height) => {
                img.resize_to_fill(width, height, FilterType::Lanczos3)
            }
            ImageSize::Default => img,
            ImageSize::Invalid => img,
        };

        Self {
            url: self.url,
            bytes: img.clone().into_bytes(),
            image: img,
        }
    }

    pub fn change_format(&self, format: ImageFormat) -> Result<Self> {
        // TODO: Check for formats which do not support transparency.
        // Eventually this function should not return a Result.
        let mut buffer = Cursor::new(Vec::new());

        self.image
            .write_to(&mut buffer, format)
            .context("Can't write image to bytes")?;

        let img = ImageReader::new(buffer)
            .with_guessed_format()
            .context("Can't determine file type")?
            .decode()
            .context("Can't decode image")?;

        Ok(Self {
            url: self.url.clone(),
            bytes: img.clone().into_bytes(),
            image: img,
        })
    }

    /// Writes the images bytes to stdout.
    pub fn write_to_stdout(&self, format: ImageFormat) -> Result<(), FavilibError> {
        let mut buffer = Cursor::new(Vec::new());

        self.image
            .write_to(&mut buffer, format)
            .context("Can't write image to stdout")?;

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(buffer.get_ref())?;

        Ok(())
    }

    /// Exports the image to a file at the given path.
    pub fn export<Q>(&self, path: Q, format: ImageFormat) -> Result<(), FavilibError>
    where
        Q: AsRef<Path>,
    {
        self.image.save_with_format(path, format).context(format!(
            "Failed to save image from domain {}",
            self.url.as_str()
        ))?;
        Ok(())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn image(&self) -> &DynamicImage {
        &self.image
    }

    /// Exact URL of the favicon including it's path.
    pub fn url(&self) -> &Url {
        &self.url
    }
}

/// Fetches a favicon from a URL and saves it to a file at the given path.
pub fn fetch<Q>(
    url: Url,
    image_size: ImageSize,
    format: ImageFormat,
    path: &Q,
    client: Option<Client>,
) -> Result<(), FavilibError>
where
    Q: AsRef<Path>,
{
    let client = client.unwrap_or(Client::new());
    let favicon = Favicon::fetch(url, Some(client))?;
    let favicon = favicon.resize(image_size);
    favicon.export(&path, format)?;
    Ok(())
}

/// Represents the size of the image to be fetched.
/// Default values are: Small (16x16), Medium (32x32), Large (64x64).
/// Custom allows for custom sizes to be set.
/// Default uses the original size of the image.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImageSize {
    Small,
    Medium,
    Large,
    Custom(u32, u32),
    Default,
    Invalid,
}

impl From<&str> for ImageSize {
    fn from(s: &str) -> Self {
        match s {
            "small" => ImageSize::Small,
            "medium" => ImageSize::Medium,
            "large" => ImageSize::Large,
            "default" => ImageSize::Default,
            _ => {
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() != 2 {
                    return ImageSize::Invalid;
                }
                let width = parts[0].parse();
                let height = parts[1].parse();

                if width.is_err() || height.is_err() {
                    return ImageSize::Invalid;
                }

                ImageSize::Custom(width.unwrap(), height.unwrap())
            }
        }
    }
}
