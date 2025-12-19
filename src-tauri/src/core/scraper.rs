use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct GalleryInfo {
    pub id: String,
    pub name: String,
}

pub struct Scraper;

impl Scraper {
    pub async fn get_galleries(client: &Client, user_id: &str, post_type: &str) -> Result<Vec<GalleryInfo>> {
        let url = format!("https://gallog.dcinside.com/{}/{}", user_id, post_type);
        let res = client.get(&url).send().await?.text().await?;
        let document = Html::parse_document(&res);
        
        let mut galleries = Vec::new();
        let selector = Selector::parse("div.option_sort.gallog > div > ul > li").unwrap();
        
        for element in document.select(&selector) {
            if let Some(data_value) = element.value().attr("data-value") {
                let name = element.text().collect::<Vec<_>>().join("");
                if !data_value.is_empty() {
                    galleries.push(GalleryInfo {
                        id: data_value.to_string(),
                        name: name.trim().to_string(),
                    });
                }
            }
        }
        
        Ok(galleries)
    }

    pub async fn get_posts(client: &Client, user_id: &str, post_type: &str, gallery_id: Option<&str>, page: u32) -> Result<Vec<String>> {
        let mut url = format!("https://gallog.dcinside.com/{}/{}/index?", user_id, post_type);
        if let Some(gno) = gallery_id {
            url.push_str(&format!("cno={}&", gno));
        }
        url.push_str(&format!("p={}", page));

        let res = client.get(&url).send().await?.text().await?;
        let document = Html::parse_document(&res);
        
        let selector = Selector::parse(".cont_listbox > li").unwrap();
        let mut post_ids = Vec::new();

        for element in document.select(&selector) {
            if let Some(no) = element.value().attr("data-no") {
                post_ids.push(no.to_string());
            }
        }

        Ok(post_ids)
    }
}
