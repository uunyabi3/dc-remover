use crate::core::auth::SessionManager;
use crate::core::captcha::CaptchaSolver;
use anyhow::{Result, anyhow};
use std::sync::Arc;

const DCINSIDE_SITE_KEY: &str = "6LcJyr4UAAAAAOy9Q_e9sDWPSHJ_aXus4UnYLfgL";

pub struct Cleaner {
    session: Arc<SessionManager>,
    captcha_solver: Option<Arc<dyn CaptchaSolver>>,
}

impl Cleaner {
    pub fn new(session: Arc<SessionManager>, captcha_solver: Option<Arc<dyn CaptchaSolver>>) -> Self {
        Self { session, captcha_solver }
    }

    pub fn has_solver(&self) -> bool {
        self.captcha_solver.is_some()
    }

    pub async fn delete_post(&self, post_no: &str, post_type: &str, solve_captcha: bool) -> Result<String> {
        let user_id = self.session.user_id.lock().await.clone();
        let gallog_url = format!("https://gallog.dcinside.com/{}/{}", user_id, post_type);
        
        let page_res = self.session.client
            .get(&gallog_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await?;
        
        let mut ci_t_value = String::new();
        for cookie in page_res.cookies() {
            if cookie.name() == "ci_c" {
                ci_t_value = cookie.value().to_string();
                break;
            }
        }
        
        let mut form_data = std::collections::HashMap::new();
        form_data.insert("ci_t", ci_t_value.clone());
        form_data.insert("no", post_no.to_string());
        form_data.insert("service_code", "undefined".to_string());
        
        if solve_captcha {
            if let Some(solver) = &self.captcha_solver {
                let token = solver.solve(DCINSIDE_SITE_KEY, &gallog_url).await?;
                form_data.insert("g-recaptcha-response", token);
            } else {
                return Err(anyhow!("Captcha solver required but not configured"));
            }
        }

        let delete_url = format!("https://gallog.dcinside.com/{}/ajax/log_list_ajax/delete", user_id);
        
        let res = self.session.client.post(&delete_url)
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Accept-Language", "ko-KR,ko;q=0.9")
            .header("Connection", "keep-alive")
            .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .header("Host", "gallog.dcinside.com")
            .header("Origin", "https://gallog.dcinside.com")
            .header("Referer", format!("https://gallog.dcinside.com/{}/{}", user_id, post_type))
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
            .header("X-Requested-With", "XMLHttpRequest")
            .form(&form_data)
            .send()
            .await?;
            
        let _status = res.status();
        let text = res.text().await?;
        
        if text.contains("\"result\":\"success\"") || text.contains("success") {
            Ok("success".to_string())
        } else if text.contains("captcha") {
            Ok("captcha".to_string())
        } else {
            Ok(format!("fail: {}", text))
        }
    }
}
