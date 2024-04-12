use anyhow::Result;
use clap::Parser;
use favilib::Favicon;
use favilib::ImageSize;
use image::ImageFormat;
use std::path::Path;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL of the website
    url: String,

    #[arg(short, long, default_value = "default")]
    size: Option<ImageSize>,

    /// Path to save the favicon
    #[arg(short, long, required_unless_present = "stdout")]
    path: Option<String>,

    /// Set this flag to only print the URL of the favicon
    #[arg(long, required_unless_present = "path")]
    url_only: bool,

    /// Set this flag to only write the favicon bytes to stdout. Mutually exclusive with `path`.
    #[arg(long)]
    stdout: bool,
}

fn main() {
    let args = Args::parse();

    let url = Url::parse(&args.url).expect("Failed to parse URL");

    let size = args.size.expect("Failed to parse size");

    let favicon = Favicon::fetch(url, None).expect("Failed to fetch favicon");
    let favicon = favicon.resize(size);

    let path = args.path.clone().unwrap();

    let path = Path::new(&path);

    let target = if args.stdout {
        ExportTarget::Stdout
    } else {
        ExportTarget::File(Path::new(path))
    };

    write_favicon(favicon, target, ImageFormat::Png).expect("Failed to write favicon");
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
