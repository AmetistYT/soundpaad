use anyhow::{Context, Result};
use scraper::{Html, Selector};
use crate::db::models::ShopSound;

const BASE_URL: &str = "https://www.myinstants.com";

pub struct MyInstantsClient {
    client: reqwest::blocking::Client,
}

impl MyInstantsClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .context("Не удалось создать HTTP клиент")?;
        Ok(Self { client })
    }

    pub fn get_sounds(&self, path: &str) -> Result<Vec<ShopSound>> {
        let html = self.fetch_page(path)?;
        self.parse_sound_list(&html)
    }

    pub fn search(&self, query: &str) -> Result<Vec<ShopSound>> {
        let path = format!("/en/search/?name={}", urlencoding::encode(query));
        let html = self.fetch_page(&path)?;
        self.parse_sound_list(&html)
    }

    pub fn get_sound_mp3_url(&self, instant_path: &str) -> Result<String> {
        let html = self.fetch_page(instant_path)?;
        let doc = Html::parse_document(&html);

        // Ищем ссылку "Download MP3" — она содержит /media/sounds/
        let link_selector = Selector::parse("a[href*='/media/sounds/']").unwrap();
        if let Some(link) = doc.select(&link_selector).next() {
            if let Some(href) = link.value().attr("href") {
                let url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("{}{}", BASE_URL, href)
                };
                return Ok(url);
            }
        }

        // Альтернатива: ищем в кнопке onclick
        let btn_selector = Selector::parse("button[onclick]").unwrap();
        for el in doc.select(&btn_selector) {
            if let Some(onclick) = el.value().attr("onclick") {
                if let Some(url) = Self::extract_media_url(onclick) {
                    return Ok(if url.starts_with("http") { url } else { format!("{}{}", BASE_URL, url) });
                }
            }
        }

        anyhow::bail!("Не удалось найти MP3 URL на странице")
    }

    pub fn download_sound(&self, url: &str, dest: &std::path::Path) -> Result<()> {
        let response = self.client.get(url).send()?;
        if !response.status().is_success() {
            anyhow::bail!("HTTP ошибка при скачивании: {}", response.status());
        }
        let bytes = response.bytes()?;
        std::fs::write(dest, &bytes)?;
        Ok(())
    }

    fn fetch_page(&self, path: &str) -> Result<String> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", BASE_URL, path)
        };
        let response = self.client.get(&url).send()?;
        if !response.status().is_success() {
            anyhow::bail!("HTTP ошибка {}: {}", response.status(), url);
        }
        Ok(response.text()?)
    }

    fn parse_sound_list(&self, html: &str) -> Result<Vec<ShopSound>> {
        let doc = Html::parse_document(html);
        let mut sounds = Vec::new();

        // Стратегия 1: Ищем кнопки с onclick содержащим /media/sounds/
        let btn_selector = Selector::parse("button[onclick]").unwrap();
        for el in doc.select(&btn_selector) {
            if let Some(onclick) = el.value().attr("onclick") {
                if onclick.contains("/media/sounds/") {
                    let name = el.text().collect::<String>().trim().to_string();
                    let url = Self::extract_media_url(onclick)
                        .map(|u| if u.starts_with("http") { u } else { format!("{}{}", BASE_URL, u) })
                        .unwrap_or_default();

                    if !name.is_empty() {
                        sounds.push(ShopSound {
                            name,
                            url,
                            category: None,
                        });
                    }
                }
            }
        }

        // Стратегия 2: Ищем ссылки на instant-страницы (всегда, не только если кнопок нет)
        let link_selector = Selector::parse("a[href^='/en/instant/']").unwrap();
        for el in doc.select(&link_selector) {
            let name = el.text().collect::<String>().trim().to_string();
            let href = el.value().attr("href").unwrap_or("").to_string();
            if !name.is_empty() && !href.is_empty() {
                sounds.push(ShopSound {
                    name,
                    url: href,
                    category: None,
                });
            }
        }

        // Убираем дубликаты по имени (приоритет — первый найденный, с прямым MP3 URL)
        let mut seen = std::collections::HashSet::new();
        sounds.retain(|s| seen.insert(s.name.clone()));

        Ok(sounds)
    }

    fn extract_media_url(onclick: &str) -> Option<String> {
        // Формат: play(".../media/sounds/xxx.mp3") или similar
        let start = onclick.find("/media/")?;
        let sub = &onclick[start..];
        let end = sub.find('"').or_else(|| sub.find(')'))?;
        Some(sub[..end].to_string())
    }
}
