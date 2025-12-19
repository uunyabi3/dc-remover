use tauri::{State, Emitter, AppHandle};
use crate::core::auth::SessionManager;
use crate::core::scraper::{Scraper, GalleryInfo};
use crate::core::cleaner::Cleaner;
use crate::core::captcha::{TwoCaptcha, AntiCaptcha, CaptchaSolver};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Serialize;

pub struct AppState {
    pub session: Arc<SessionManager>,
    pub cleaner: Mutex<Option<Cleaner>>,
}

#[derive(Clone, Serialize)]
pub struct ProgressEvent {
    pub current: u32,
    pub total: u32,
    pub message: String,
}

#[tauri::command]
pub async fn login(state: State<'_, AppState>, id: String, pw: String) -> Result<bool, String> {
    state.session.login(&id, &pw).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_galleries(state: State<'_, AppState>, post_type: String) -> Result<Vec<GalleryInfo>, String> {
    let user_id = state.session.user_id.lock().await;
    if user_id.is_empty() {
        return Err("Not logged in".to_string());
    }
    Scraper::get_galleries(&state.session.client, &user_id, &post_type).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_cleaning(
    app: AppHandle,
    state: State<'_, AppState>,
    post_type: String, 
    gallery_id: Option<String>,
    captcha_key: Option<String>,
    captcha_type: Option<String>
) -> Result<String, String> {
    
    let solver: Option<Arc<dyn CaptchaSolver>> = if let (Some(key), Some(ctype)) = (captcha_key.clone(), captcha_type.clone()) {
        if !key.is_empty() {
            match ctype.as_str() {
                "2captcha" => Some(Arc::new(TwoCaptcha::new(key))),
                "anticaptcha" => Some(Arc::new(AntiCaptcha::new(key))),
                _ => None
            }
        } else {
            None
        }
    } else {
        None
    };

    let cleaner = Cleaner::new(state.session.clone(), solver);
    *state.cleaner.lock().await = Some(cleaner);

    let user_id = state.session.user_id.lock().await.clone();
    if user_id.is_empty() {
        return Err("로그인이 필요합니다".to_string());
    }

    let _ = app.emit("cleaning_progress", ProgressEvent {
        current: 0,
        total: 0,
        message: "게시물 목록 수집 중...".to_string(),
    });

    let gallery_ref = gallery_id.as_deref();
    let mut all_posts: Vec<String> = Vec::new();
    let mut page = 1u32;
    
    loop {
        let posts = Scraper::get_posts(&state.session.client, &user_id, &post_type, gallery_ref, page)
            .await
            .map_err(|e| e.to_string())?;
        
        if posts.is_empty() {
            break;
        }
        
        all_posts.extend(posts);
        page += 1;
        
        if page > 100 {
            break;
        }
    }

    let total = all_posts.len() as u32;
    
    if total == 0 {
        let _ = app.emit("cleaning_progress", ProgressEvent {
            current: 0,
            total: 0,
            message: "삭제할 게시물이 없습니다".to_string(),
        });
        return Ok("삭제할 게시물이 없습니다".to_string());
    }

    let _ = app.emit("cleaning_progress", ProgressEvent {
        current: 0,
        total,
        message: format!("총 {}개 발견, 삭제 시작...", total),
    });

    let cleaner_guard = state.cleaner.lock().await;
    let cleaner = cleaner_guard.as_ref().ok_or("Cleaner not initialized")?;
    
    let mut deleted = 0u32;
    let mut failed = 0u32;
    let mut solve_captcha = false;

    for (i, post_no) in all_posts.iter().enumerate() {
        let result = cleaner.delete_post(post_no, &post_type, solve_captcha).await;
        
        match result {
            Ok(res) => {
                if res == "success" {
                    deleted += 1;
                    solve_captcha = false;
                } else if res == "captcha" || res.contains("captcha") {
                    if cleaner.has_solver() {
                        solve_captcha = true;
                        failed += 1;
                    } else {
                        failed += 1;
                    }
                } else {
                    failed += 1;
                }
            }
            Err(_) => {
                failed += 1;
            }
        }

        let _ = app.emit("cleaning_progress", ProgressEvent {
            current: (i + 1) as u32,
            total,
            message: format!("삭제 중... ({}/{})", i + 1, total),
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    let final_message = format!("완료! 삭제: {}개, 실패: {}개", deleted, failed);
    let _ = app.emit("cleaning_progress", ProgressEvent {
        current: total,
        total,
        message: final_message.clone(),
    });

    Ok(final_message)
}
