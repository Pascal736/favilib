use thiserror::Error;

#[derive(Error, Debug)]
pub enum FavilibError {
    #[error("Failed to fetch favicon")]
    FetchError(#[from] reqwest::Error),

    #[error("No favicon found on website")]
    NoFaviconFoundError,

    #[error("Failed to decode favicon")]
    ImageDecodeError(#[from] image::ImageError),

    #[error("Failed to parse URL")]
    UrlParseError(#[from] url::ParseError),

    #[error("Failed to parse size")]
    SizeParseError,

    #[error("Failed to write to file")]
    IoError(#[from] std::io::Error),

    #[error("Failed to write to stdout")]
    OtherError(#[from] anyhow::Error),
}
