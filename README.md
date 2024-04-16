[![crates.io version](https://img.shields.io/crates/v/favilib)](https://crates.io/crates/favilib)
[![docs.rs](https://img.shields.io/docsrs/favilib)](https://docs.rs/crate/favilib/latest)

# favilib
Favicon Fetcher written in Rust. Contains library as well as CLI tool.

> [!WARNING]
> This is a work in progress. The library is not yet stable.


## Library
```rust
use favilib::{fetch, Favicon, ImageSize, ImageFormat, Url, Client};

let url = Url::parse("https://github.com").unwrap();

// Fetch and export image directly
let _ = fetch(&url, ImageSize::Large, ImageFormat::Png, "favicon.png", None);

// Fetch image and get it as a struct
let client = Client::new(); 
let favicon = Favicon::fetch(&url, Some(client)).unwrap();

let resized_favicon = favicon.resize(ImageSize::Custom(32,32)).unwrap();
let reformatted_favicon = resized_favicon.change_format(ImageFormat::Png).unwrap();
reformatted_favicon.export("favicon.png").unwrap();
```


## CLI
### Installation
CLI can be installed via cargo by running `cargo install favilib`

### Interface

```bash
# Format will be changed based on the file ending
favilib fetch github.com --size large --path favicon.png 

# Format can also be specified explicitly and bytes can be printed to stdout. Size can be specified explicitly
favilib fetch github.com --size 32,32 --format ico --stdout

# Prints the extracted URL of the favicon to stdout
favilib fetch github.com --url-only --stdout

```
