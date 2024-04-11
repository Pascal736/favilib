use anyhow::{Context as _, Result};
use scraper::{self, Html, Selector};
use std::sync::mpsc;
use std::thread;
use url::Url;

use super::Favicon;

pub(crate) fn fetch_and_validate_favicon(
    url: Url,
    client: &reqwest::blocking::Client,
) -> Result<Favicon> {
    let url = add_www_to_host(url)?;
    let page = get_web_page(url.clone(), client)?;
    let head = get_page_head_section(page)?;
    let favicon_urls = get_favicon_urls_from_header(head, url);
    Ok(fetch_all_favicons(favicon_urls, client)?)
}

fn get_web_page(url: Url, client: &reqwest::blocking::Client) -> Result<String> {
    let response = client.get(url).send()?;

    let body = response.text()?;
    Ok(body)
}

fn get_page_head_section(page: String) -> Result<Html> {
    let document = scraper::Html::parse_document(&page);
    let selector = scraper::Selector::parse("head").unwrap();
    let header = document
        .select(&selector)
        .next()
        .context("No header section found")?;
    Ok(Html::parse_fragment(&header.html()))
}

fn get_favicon_urls_from_header(header: Html, base_url: Url) -> Vec<Url> {
    let link_selector = Selector::parse("link").unwrap();
    let meta_selector = Selector::parse("meta").unwrap();

    let href_attr = "href";
    let rel_attr = "rel";
    let content_attr = "content";

    let icon_types = [
        "icon",
        "shortcut icon",
        "apple-touch-icon",
        "favicon",
        "mask-icon",
        "fluid-icon",
        "image",
    ];

    let mut urls = vec![];

    for link in header.select(&link_selector) {
        match link.value().attr(href_attr) {
            Some(href) => {
                let rel = link.value().attr(rel_attr).unwrap_or_default();
                if icon_types.iter().any(|&icon_type| rel.contains(icon_type)) {
                    if let Ok(url) = base_url.join(href) {
                        urls.push(url);
                    }
                }
            }
            None => continue,
        }
    }

    for meta in header.select(&meta_selector) {
        match meta.value().attr(content_attr) {
            Some(content) => {
                if icon_types
                    .iter()
                    .any(|&icon_type| content.contains(icon_type))
                {
                    if let Ok(url) = base_url.join(content) {
                        urls.push(url);
                    }
                }
            }

            None => continue,
        }
    }

    match urls.is_empty() {
        // If no favicon urls are found, add the default favicon url
        true => vec![base_url.join("/favicon.ico").unwrap()],
        false => urls,
    }
}

fn fetch_favicon_from_url(url: Url, client: &reqwest::blocking::Client) -> Result<Favicon> {
    println!("Fetching favicon from: {}", url);
    let response = client.get(url.clone()).send()?;
    println!("Response: {:?}", response);
    let data = response.bytes()?.to_vec();
    Ok(Favicon::build(url, data)?)
}

fn fetch_all_favicons(urls: Vec<Url>, client: &reqwest::blocking::Client) -> Result<Favicon> {
    let (tx, rx) = mpsc::channel();

    let mut join_handlers = Vec::with_capacity(urls.len());

    for url in urls.clone() {
        let tx_clone = tx.clone();
        let client = client.clone();
        let handle = thread::spawn(move || {
            let result = fetch_favicon_from_url(url, &client);
            tx_clone.send(result).unwrap();
        });
        join_handlers.push(handle);
    }

    for handle in join_handlers {
        handle.join().unwrap();
    }

    for _ in 0..urls.len() {
        match rx.recv().unwrap() {
            Ok(favicon) => return Ok(favicon),
            Err(_) => continue,
        }
    }

    Err(anyhow::anyhow!("No favicon found"))
}

/// Some websites host static files on a domain without the `www` subdomain.
fn add_www_to_host(url: Url) -> Result<Url> {
    let host = url.host_str().context("No host found")?;
    let mut new_url = url.clone();
    if !host.starts_with("www.") {
        let new_host = format!("www.{}", host);
        new_url.set_host(Some(&new_host))?;
    }
    Ok(new_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_head_section() -> Result<()> {
        let html = r#"<html><head><link rel="icon" type="image/svg+xml" href="/favicon.svg"></head><body><p>Content</p></body></html>"#;
        let link_selector = Selector::parse("link").unwrap();

        let head = get_page_head_section(html.to_string())?;
        assert!(head.select(&link_selector).next().is_some());
        Ok(())
    }

    #[test]
    fn test_get_favicon_urls_from_header() -> Result<()> {
        let head =
            Html::parse_fragment(r#"<link rel="icon" type="image/svg+xml" href="/favicon.svg">"#);
        let base_url = Url::parse("https://example.com")?;

        let urls = get_favicon_urls_from_header(head, base_url);

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], Url::parse("https://example.com/favicon.svg")?);

        Ok(())
    }
    #[test]
    fn test_get_favicon_urls_from_header_multiple_links() -> Result<()> {
        let html = r#"
            <head>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg">
                <link rel="icon" type="image/svg+xml" href="/favicon2.svg">
            </head>
           "#;
        let head = get_page_head_section(html.to_string())?;
        let base_url = Url::parse("https://example.com")?;

        let urls = get_favicon_urls_from_header(head, base_url);

        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], Url::parse("https://example.com/favicon.svg")?);
        assert_eq!(urls[1], Url::parse("https://example.com/favicon2.svg")?);

        Ok(())
    }

    #[test]
    fn test_get_favicon_urls_from_header_excludes_non_favicon_links() -> Result<()> {
        let html = r#"
            <head>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg">
                <link rel="style sheet" type="image/svg+xml" href="/style.css">
            </head>
           "#;
        let head = get_page_head_section(html.to_string())?;
        let base_url = Url::parse("https://example.com")?;

        let urls = get_favicon_urls_from_header(head, base_url);

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], Url::parse("https://example.com/favicon.svg")?);

        Ok(())
    }

    #[test]
    fn test_get_favicon_url_from_meta_tag() -> Result<()> {
        let html = r#"<meta content="/favicon.svg" itemprop="image">"#;

        let head = get_page_head_section(html.to_string())?;
        let base_url = Url::parse("https://example.com")?;

        let urls = get_favicon_urls_from_header(head, base_url);

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], Url::parse("https://example.com/favicon.svg")?);

        Ok(())
    }
}
