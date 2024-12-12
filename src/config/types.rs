use serde_json::from_slice;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub enum Auth {
    #[serde(alias = "basic")]
    Basic { user: String, password: String },
    #[default]
    None,
}

#[derive(Deserialize, Default, Debug)]
struct RssConfig {
    feed: String,
    dest: String,
    auth: Auth,
}

#[derive(Deserialize, Debug)]
pub struct MovieConfig {
    #[serde(alias = "watch")]
    pub watch_path: String,
    #[serde(alias = "saveDir")]
    pub save_dir: String,
    #[serde(alias = "movieDir")]
    pub movie_dir: Option<String>,
    #[serde(alias = "tvDir")]
    pub tv_dir: Option<String>,
    #[serde(alias = "rss")]
    rss_config: RssConfig,
}

impl MovieConfig {
    pub fn save_path(&self) -> &Path {
        Path::new(self.save_dir.as_str())
    }
    pub fn new(file: Option<Vec<u8>>) -> Self {
        match file {
            None => MovieConfig {
                watch_path: "/srv/done".to_string(),
                save_dir: "/mnt".to_string(),
                movie_dir: Some("movie".to_string()),
                tv_dir: Some("adult".to_string()),
                rss_config: Default::default(),
            },
            Some(file) => {
                let test = from_slice::<MovieConfig>(&file);
                println!("{:?}", test);
                from_slice::<MovieConfig>(&file).unwrap_or(MovieConfig {
                    watch_path: "/srv/done".to_string(),
                    save_dir: "/mnt".to_string(),
                    movie_dir: Some("movie".to_string()),
                    tv_dir: Some("adult".to_string()),
                    rss_config: Default::default(),
                })
            }
        }
    }
    pub fn rss_feed(&self) -> &str {
        self.rss_config.feed.as_str()
    }
    pub fn get_auth(&self) -> &Auth {
        &self.rss_config.auth
    }
    pub fn get_rpc_dest(&self) -> String {
        self.rss_config.dest.to_string()
    }
}
