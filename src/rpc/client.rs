use super::methods::RpcAction;
use crate::config::types::Auth::Basic;
use crate::config::types::MovieConfig;
use anyhow::bail;
use anyhow::Ok as AnyOk;
use anyhow::Result as AnyResult;
use reqwest::{self, Client, StatusCode};
use std::cell::RefCell;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RpcClient {
    session_id: Mutex<RefCell<String>>,
    common_client: Client,
    url: String,
    config: Arc<MovieConfig>,
}

impl RpcClient {
    async fn update_session_id(&self, id: &str) {
        self.session_id.lock().await.replace(id.to_string());
    }
    pub fn new(config: Arc<MovieConfig>) -> Self {
        Self {
            session_id: Mutex::new(RefCell::new("".to_string())),
            common_client: Client::new(),
            url: config.get_rpc_dest(),
            config,
        }
    }
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub async fn request(&self, action: RpcAction, session: Option<String>) -> AnyResult<String> {
        let session_id = if let Some(tst) = session {
            tst
        } else {
            let id = self.session_id.lock().await;
            id.clone().into_inner()
        };
        println!("{:?}", action);

        let semi_client = self
            .common_client
            .post(&self.url)
            .json(&action)
            .header("X-Transmission-Session-Id", session_id);

        let res = if let Basic { user, password } = self.config.get_auth() {
            semi_client.basic_auth(user, Some(password)).send().await
        } else {
            semi_client.send().await
        };

        match res {
            Err(err) => {
                println!("{:?}", err);
                bail!(err)
            }
            Ok(res) if res.status() == StatusCode::CONFLICT => {
                println!("{:?}", res);
                let session_id = res
                    .headers()
                    .get("X-Transmission-Session-Id")
                    .map_or("".to_string(), |t| t.to_str().unwrap().to_string());
                self.update_session_id(&session_id).await;
                let pin = Box::pin(self.request(action, Some(session_id)));
                let end = pin.await;
                AnyOk(end?)
            }
            Ok(test) => AnyOk(test.text().await?),
        }
    }
}
