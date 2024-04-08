use anyhow::{Context as _, Result};
use image::{imageops::FilterType, io::Reader as ImageReader};
pub use image::{DynamicImage, ImageFormat};
pub use reqwest::Client;
use std::io::Cursor;
use std::path::Path;
pub use url::Url;

mod scraper;

#[derive(Debug, Clone)]
pub struct Favicon {
    url: Url,
    bytes: Vec<u8>,
    image: DynamicImage,
}

/// Represents the size of the image to be fetched.
/// Default values are: Small (16x16), Medium (32x32), Large (64x64).
/// Custom allows for custom sizes to be set.
/// Default uses the original size of the image.
pub enum ImageSize {
    Small,
    Medium,
    Large,
    Custom(u32, u32),
    Default,
}

impl Favicon {
    /// Fetches a favicon from a URL and returns a new Favicon instance.
    /// The fetching algorithm selects the first valid favicon found on the page.
    pub async fn fetch(url: Url, client: Option<Client>) -> Result<Self> {
        let client = client.unwrap_or(Client::new());
        Ok(scraper::fetch_and_validate_favicon(url.clone(), &client).await?)
    }

    /// Builds a new Favicon instance from a URL and a byte vector.
    /// Does not fetch the image from the URL.
    /// Use the fetch function to fetch the image.
    pub fn build(url: Url, data: Vec<u8>) -> Result<Self> {
        let image = ImageReader::new(Cursor::new(data.clone()))
            .with_guessed_format()
            .context("Can't determine file type")?
            .decode()
            .context("Can't decode image")?;
        Ok(Self {
            url,
            bytes: data,
            image,
        })
    }

    /// Changes the size of the image on the instance. Also returns th instance to allow chaining.
    pub fn resize(mut self, size: ImageSize) -> Favicon {
        let img = self.image;
        let image = match size {
            ImageSize::Small => img.resize_to_fill(16, 16, FilterType::Lanczos3),
            ImageSize::Medium => img.resize_to_fill(32, 32, FilterType::Lanczos3),
            ImageSize::Large => img.resize_to_fill(64, 64, FilterType::Lanczos3),
            ImageSize::Custom(width, height) => {
                img.resize_to_fill(width, height, FilterType::Lanczos3)
            }
            ImageSize::Default => img,
        };
        self.image = image;
        self
    }

    /// Exports the image to a file at the given path.
    pub fn export(&self, path: &Path, format: ImageFormat) -> Result<()> {
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
    pub fn url(&self) -> &Url {
        &self.url
    }
}

/// Fetches a favicon from a URL and saves it to a file at the given path.
pub async fn fetch(
    url: Url,
    image_size: ImageSize,
    format: ImageFormat,
    path: &Path,
) -> Result<()> {
    println!("Fetching favicon from {}", url.as_str());
    let client = Client::new();

    let favicon = Favicon::fetch(url, Some(client)).await?;
    let favicon = favicon.resize(image_size);
    favicon.export(&path, format)?;
    Ok(())
}
