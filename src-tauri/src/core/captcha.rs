use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait CaptchaSolver: Send + Sync {
    async fn solve(&self, site_key: &str, url: &str) -> Result<String>;
}

pub struct TwoCaptcha {
    api_key: String,
    client: reqwest::Client,
}

impl TwoCaptcha {
    pub fn new(api_key: String) -> Self {
        Self { api_key, client: reqwest::Client::new() }
    }
}

#[async_trait]
impl CaptchaSolver for TwoCaptcha {
    async fn solve(&self, site_key: &str, url: &str) -> Result<String> {
        let res = self.client.post(format!("http://2captcha.com/in.php?key={}&method=userrecaptcha&googlekey={}&pageurl={}&json=1", self.api_key, site_key, url))
            .send().await?.json::<Value>().await?;
        
        if res["status"].as_i64() != Some(1) {
             return Err(anyhow!("2Captcha Request Failed: {}", res["request"]));
        }
        
        let request_id = res["request"].as_str().unwrap().to_string();
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let poll = self.client.get(format!("http://2captcha.com/res.php?key={}&action=get&id={}&json=1", self.api_key, request_id))
                .send().await?.json::<Value>().await?;
                
            if poll["status"].as_i64() == Some(1) {
                return Ok(poll["request"].as_str().unwrap().to_string());
            }
            
            if poll["request"].as_str() == Some("CAPCHA_NOT_READY") {
                continue;
            } else {
                return Err(anyhow!("2Captcha Error: {}", poll["request"]));
            }
        }
    }
}

pub struct AntiCaptcha {
    api_key: String,
    client: reqwest::Client,
}

impl AntiCaptcha {
    pub fn new(api_key: String) -> Self {
        Self { api_key, client: reqwest::Client::new() }
    }
}

#[async_trait]
impl CaptchaSolver for AntiCaptcha {
    async fn solve(&self, site_key: &str, url: &str) -> Result<String> {
        use serde_json::json;

        let create_task_res = self.client.post("https://api.anti-captcha.com/createTask")
            .json(&json!({
                "clientKey": self.api_key,
                "task": {
                    "type": "RecaptchaV2TaskProxyless",
                    "websiteURL": url,
                    "websiteKey": site_key
                }
            }))
            .send().await?.json::<Value>().await?;
            
        if create_task_res["errorId"].as_i64().unwrap_or(1) != 0 {
             return Err(anyhow!("AntiCaptcha Create Task Failed: {}", create_task_res["errorDescription"]));
        }
        
        let task_id = create_task_res["taskId"].as_i64().unwrap();
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let poll = self.client.post("https://api.anti-captcha.com/getTaskResult")
                .json(&json!({
                    "clientKey": self.api_key,
                    "taskId": task_id
                }))
                .send().await?.json::<Value>().await?;
                
            if poll["errorId"].as_i64().unwrap_or(1) != 0 {
                 return Err(anyhow!("AntiCaptcha Poll Failed: {}", poll["errorDescription"]));
            }
            
            if poll["status"].as_str() == Some("ready") {
                return Ok(poll["solution"]["gRecaptchaResponse"].as_str().unwrap().to_string());
            }
        }
    }
}
