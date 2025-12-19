use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct SessionManager {
    pub client: Client,
    pub user_id: Arc<Mutex<String>>,
}

impl SessionManager {
    pub fn new() -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .user_agent(USER_AGENT)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
            
        Self {
            client,
            user_id: Arc::new(Mutex::new(String::new())),
        }
    }

    fn parse_form_inputs(html: &str, selector_str: &str) -> HashMap<String, String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse(selector_str).unwrap();
        let mut form_data = HashMap::new();
        
        for element in document.select(&selector) {
            if let (Some(name), Some(value)) = (
                element.value().attr("name"),
                element.value().attr("value")
            ) {
                form_data.insert(name.to_string(), value.to_string());
            }
        }
        
        form_data
    }

    pub async fn login(&self, user_id: &str, user_pw: &str) -> Result<bool> {
        let login_url = "https://sign.dcinside.com/login/member_check";
        
        let main_page = self.client
            .get("https://www.dcinside.com/")
            .header("Referer", "https://www.dcinside.com/")
            .send()
            .await?
            .text()
            .await?;

        let mut login_data = Self::parse_form_inputs(&main_page, "#login_process > input");
        login_data.insert("user_id".to_string(), user_id.to_lowercase());
        login_data.insert("pw".to_string(), user_pw.to_string());

        let _res = self.client.post(login_url)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", "https://www.dcinside.com/")
            .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .form(&login_data)
            .send()
            .await?;

        let check_res = self.client
            .get("https://www.dcinside.com/")
            .send()
            .await?
            .text()
            .await?;
        
        let is_logged_in = {
            let document = Html::parse_document(&check_res);
            let logout_selector = Selector::parse(".logout").unwrap();
            document.select(&logout_selector).next().is_some()
        };
        
        if is_logged_in {
            *self.user_id.lock().await = user_id.to_lowercase();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
