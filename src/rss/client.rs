use crate::config::types::MovieConfig;
use anyhow::{Ok as AnyOk, Result as AnyResult};
use futures::stream;
use futures::StreamExt;
use reqwest::Response;
use rss::Channel;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Sender as MsSender;
use tokio::sync::oneshot::{channel as one_shot, Sender as OneSender};

pub struct RssWatcher {
    url: String,
    tx: Sender<String>,
    tx_check: MsSender<(String, OneSender<bool>)>,
}

impl RssWatcher {
    pub fn new(
        config: Arc<MovieConfig>,
        sender: Sender<String>,
        tx_check: MsSender<(String, OneSender<bool>)>,
    ) -> Self {
        Self {
            url: String::from(config.rss_feed()),
            tx: sender,
            tx_check,
        }
    }
    pub async fn start(&self) -> AnyResult<()> {
        let url = self.url.clone();

        let txx = self.tx.clone();

        loop {
            match reqwest::get(url.as_str()).await {
                Ok(res) => {
                    if let Ok(ch) = req_to_rss(res).await {
                        let _ = stream::iter(
                            ch.items().iter().filter_map(|t| t.link().map(String::from)),
                        )
                        .filter_map(|link| async {
                            let (send, res) = one_shot::<bool>();
                            self.tx_check.send((link.clone(), send)).await.unwrap_or(());
                            match res.await {
                                Ok(b) => {
                                    if b {
                                        println!("Didn't send this link: [{:?}]", link);

                                        None
                                    } else {
                                        Some(link)
                                    }
                                }
                                Err(_) => {
                                    println!("Error didn't send this link: [{:?}]", link);
                                    None
                                }
                            }
                        })
                        .map(|t| {
                            txx.send(t.clone()).ok();
                        })
                        .collect::<Vec<()>>()
                        .await;
                    }
                }
                Err(err) => {
                    println!("{:?}", err)
                }
            };

            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    }
}

pub async fn req_to_rss(input: Response) -> AnyResult<Channel> {
    let content = input.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    AnyOk(channel)
}
