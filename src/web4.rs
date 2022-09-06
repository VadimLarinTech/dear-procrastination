use crate::*;

use near_sdk::env;
use near_sdk::json_types::Base64VecU8;
use std::collections::HashMap;

const STYLES_BODY: &str = include_str!("../res/style.css");

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Web4Request {
    #[serde(rename = "accountId")]
    account_id: Option<AccountId>,
    path: String,
    params: Option<HashMap<String, String>>,
    query: Option<HashMap<String, Vec<String>>>,
    preloads: Option<HashMap<String, Web4Response>>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct Web4Response {
    #[serde(rename = "contentType")]
    content_type: Option<String>,
    status: Option<u32>,
    body: Option<Base64VecU8>,
    #[serde(rename = "bodyUrl")]
    body_url: Option<String>,
    #[serde(rename = "preloadUrls")]
    preload_urls: Option<Vec<String>>,
}

impl Web4Response {
    pub fn html_response(text: String) -> Self {
        Self {
            content_type: Some(String::from("text/html; charset=UTF-8")),
            body: Some(text.into_bytes().into()),
            ..Default::default()
        }
    }

    pub fn plain_response(text: String) -> Self {
        Self {
            content_type: Some(String::from("text/plain; charset=UTF-8")),
            body: Some(text.into_bytes().into()),
            ..Default::default()
        }
    }

    pub fn preload_urls(urls: Vec<String>) -> Self {
        Self {
            preload_urls: Some(urls),
            ..Default::default()
        }
    }

    pub fn body_url(url: String) -> Self {
        Self {
            body_url: Some(url),
            ..Default::default()
        }
    }

    pub fn status(status: u32) -> Self {
        Self {
            status: Some(status),
            ..Default::default()
        }
    }
}

#[near_bindgen]
impl Contract {
    #[allow(unused_variables)]
    pub fn web4_get(&self, request: Web4Request) -> Web4Response {
        let path = request.path;

        if path == "/robots.txt" {
            return Web4Response::plain_response("User-agent: *\nDisallow:".to_string());
        }

        if path == "/add-task" {
            return Web4Response::html_response(
                include_str!("../res/add-task.html")
                    .replace("%STYLESHEET%", &STYLES_BODY)
                    .replace("%CONTRACT_ID%", &env::current_account_id().to_string())
                    .replace("%NETWORK%", "testnet"),
            );
        }

        if path == "/complete" {
            return Web4Response::html_response(
                include_str!("../res/complete.html")
                    .replace("%STYLESHEET%", &STYLES_BODY)
                    .replace("%CONTRACT_ID%", &env::current_account_id().to_string())
                    .replace("%NETWORK%", "testnet"),
            );
        }

        let mut app_html = "".to_string();
        let user_id = AccountId::new_unchecked("lrn.testnet".to_string()); // change to your contract, will be corrected in the future

        for (record_id, user_records) in self.get_all_user_tasks(user_id) {
            app_html = format!(
                "{}<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>",
                &app_html,
                record_id.to_string(),
                user_records.task,
                user_records.is_complete_status,
                user_records.guarantee_of_task_completion,
                user_records.deadline_time
            );
        }

        Web4Response::html_response(
            include_str!("../res/index.html")
                .replace("%STYLESHEET%", &STYLES_BODY)
                .replace("%USER_RECORDS%", &app_html)
                .replace("%CONTRACT_ID%", &env::current_account_id().to_string())
                .replace("%NETWORK%", "testnet"),
        )
    }
}
