# favilib
Favicon Fetcher written in Rust. Contains library as well as CLI tool


> Warning: This is a work in progress. The library is not yet stable and the CLI tool is not yet implemented.


## Library Usage
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
## CLI Usage

```bash
# Format will be changed based on the file ending
favilib fetch github.com --size large --path favicon.png 

# Format can also be specified explicitly and bytes can be printed to stdout if path is omitted. And size can be specified explicitly
favilib fetch github.com --size 32,32 --format ico

# Prints the extracted URL of the favicon to stdout
favilib fetch github.com --url

# Adds custom headers to the client fetching the Favicon
favilib fetch github.com --header "User-Agent: Mozilla/5.0" --path favicon.png
```
