use crate::datastore::client::{check, Client, Restorable, RssSave};
use crate::rpc::client::RpcClient;
use crate::rpc::methods::AddType::FileName;
use crate::rpc::methods::TorrentActions::Add;
use crate::rss::client::RssWatcher;
use anyhow::{Ok as AnyOk, Result as AnyResult};
use config::{path_functions::copy_file, types::MovieConfig};
use futures::future::join_all;
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NResult, Watcher,
};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::main;
use tokio::sync::broadcast::channel;
use tokio::sync::mpsc::{channel as ms_channel, Receiver as ms_Receiver};
use tokio::sync::oneshot::Sender as OneSender;
use tokio::task::JoinHandle;

mod config;
mod datastore;
mod rpc;
mod rss;

/// Async, futures channel based event watching
#[main]
async fn main() {
    let raw_config_file = fs::read("./mover/config.json").ok();

    let config = Arc::new(MovieConfig::new(raw_config_file));

    let (tx, mut rx) = channel::<String>(20);
    let (tx_check, mut rx_check) = ms_channel::<(String, OneSender<bool>)>(1);

    let one: JoinHandle<AnyResult<()>> = tokio::spawn({
        let txx = tx.clone();
        let local_config = Arc::clone(&config);
        async move {
            RssWatcher::new(local_config, txx, tx_check).start().await?;
            AnyOk(())
        }
    });
    let two: JoinHandle<AnyResult<()>> = tokio::spawn({
        let local_config = Arc::clone(&config);

        async {
            if let Err(e) = async_watch(local_config).await {
                println!("error: {:?}", e)
            }
            AnyOk(())
        }
    });

    let three: JoinHandle<AnyResult<()>> = tokio::spawn({
        let local_config = Arc::clone(&config);

        async move {
            let trans_client = RpcClient::new(local_config);

            loop {
                let link = rx.recv().await;
                {
                    if let Ok(s) = link {
                        let action = Add(FileName(s));
                        trans_client.request(action.to_action(), None).await?;
                    }
                }
            }
        }
    });

    let mut save_base = Client::new(PathBuf::from("./save"));
    save_base.restore().expect("no save");
    let save = RefCell::new(save_base);
    let four: JoinHandle<AnyResult<()>> = tokio::spawn({
        async move {
            loop {
                if let Some((s, sender)) = rx_check.recv().await {
                    let _ = sender.send(check::<RssSave>(&s, &save));
                }
            }
        }
    });

    join_all(vec![one, two, three, four]).await;
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, ms_Receiver<Vec<PathBuf>>)> {
    let (tx, rx) = ms_channel(16);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res: NResult<Event>| {
            let local = res.unwrap_or_default();
            println!("{:?}", local);
            let all_exist = local.paths.iter().all(|part| part.exists());
            match (all_exist, local.kind, local.paths) {
                (true, EventKind::Access(_), _) => (),
                (true, EventKind::Create(_), paths) => {
                    let _ = futures::executor::block_on(async { tx.send(paths).await });
                }
                (true, EventKind::Modify(_), paths) => {
                    let _ = futures::executor::block_on(async { tx.send(paths).await });
                }
                (true, EventKind::Remove(_), _) => (),
                (true, EventKind::Other, _) => (),
                (true, EventKind::Any, _) => (),
                _ => (),
            };
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn async_watch(config: Arc<MovieConfig>) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(config.watch_path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        println!("changed: {:?}", res);

        res.iter().for_each(|path| {
            let local_config = Arc::clone(&config);
            copy_file(path, local_config).ok().unwrap_or_default()
        });
    }

    Ok(())
}
