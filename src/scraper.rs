use anyhow::{Context as _, Result};
use futures::{stream, StreamExt};
use scraper::{self, Html, Selector};
use url::Url;

use super::Favicon;

const N_CONCURRENT_REQUESTS: usize = 10;

pub(crate) async fn fetch_and_validate_favicon(
    url: Url,
    client: &reqwest::Client,
) -> Result<Favicon> {
    let url = add_www_to_host(url)?;
    let page = get_web_page(url.clone(), client).await?;
    let head = get_page_head_section(page).await?;
    let favicon_urls = get_favicon_urls_from_header(head, url).await;
    Ok(fetch_all_favicons(favicon_urls, client).await?)
}

async fn get_web_page(url: Url, client: &reqwest::Client) -> Result<String> {
    let response = client.get(url).send().await?;

    let body = response.text().await?;
    Ok(body)
}

async fn get_page_head_section(page: String) -> Result<Html> {
    let document = scraper::Html::parse_document(&page);
    let selector = scraper::Selector::parse("head").unwrap();
    let header = document
        .select(&selector)
        .next()
        .context("No header section found")?;
    Ok(Html::parse_fragment(&header.html()))
}

async fn get_favicon_urls_from_header(header: Html, base_url: Url) -> Vec<Url> {
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

async fn fetch_favicon_from_url(url: Url, client: &reqwest::Client) -> Result<Favicon> {
    let response = client.get(url.clone()).send().await?;
    let data = response.bytes().await?.to_vec();
    Ok(Favicon::build(url, data)?)
}

async fn fetch_all_favicons(urls: Vec<Url>, client: &reqwest::Client) -> Result<Favicon> {
    let favicons = stream::iter(urls)
        .map(|url| async move { fetch_favicon_from_url(url, client).await })
        .buffered(N_CONCURRENT_REQUESTS);

    let valid_favicons: Vec<Favicon> = favicons
        .filter_map(|result| async {
            match result {
                Ok(favicon) => Some(favicon),
                Err(_) => None,
            }
        })
        .collect()
        .await;

    match valid_favicons.first() {
        Some(favicon) => Ok(favicon.clone()),
        None => Err(anyhow::anyhow!("No favicon found")),
    }
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

    #[tokio::test]
    async fn test_page_head_section() -> Result<()> {
        let html = r#"<html><head><link rel="icon" type="image/svg+xml" href="/favicon.svg"></head><body><p>Content</p></body></html>"#;
        let link_selector = Selector::parse("link").unwrap();

        let head = get_page_head_section(html.to_string()).await?;
        assert!(head.select(&link_selector).next().is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_favicon_urls_from_header() -> Result<()> {
        let head =
            Html::parse_fragment(r#"<link rel="icon" type="image/svg+xml" href="/favicon.svg">"#);
        let base_url = Url::parse("https://example.com")?;

        let urls = get_favicon_urls_from_header(head, base_url).await;

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], Url::parse("https://example.com/favicon.svg")?);

        Ok(())
    }
    #[tokio::test]
    async fn test_get_favicon_urls_from_header_multiple_links() -> Result<()> {
        let html = r#"
            <head>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg">
                <link rel="icon" type="image/svg+xml" href="/favicon2.svg">
            </head>
           "#;
        let head = get_page_head_section(html.to_string()).await?;
        let base_url = Url::parse("https://example.com")?;

        let urls = get_favicon_urls_from_header(head, base_url).await;

        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], Url::parse("https://example.com/favicon.svg")?);
        assert_eq!(urls[1], Url::parse("https://example.com/favicon2.svg")?);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_favicon_urls_from_header_excludes_non_favicon_links() -> Result<()> {
        let html = r#"
            <head>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg">
                <link rel="style sheet" type="image/svg+xml" href="/style.css">
            </head>
           "#;
        let head = get_page_head_section(html.to_string()).await?;
        let base_url = Url::parse("https://example.com")?;

        let urls = get_favicon_urls_from_header(head, base_url).await;

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], Url::parse("https://example.com/favicon.svg")?);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_favicon_url_from_meta_tag() -> Result<()> {
        let html = r#"<meta content="/favicon.svg" itemprop="image">"#;

        let head = get_page_head_section(html.to_string()).await?;
        let base_url = Url::parse("https://example.com")?;

        let urls = get_favicon_urls_from_header(head, base_url).await;

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], Url::parse("https://example.com/favicon.svg")?);

        Ok(())
    }
}
