use serde::{Deserialize, Serialize};

pub enum AddType {
    Meta(String),
    FileName(String),
}

pub enum TorrentActions {
    Start(Option<Vec<String>>),
    StartNow(Option<Vec<String>>),
    Stop(Option<Vec<String>>),
    Verify(Option<Vec<String>>),
    Reannounce(Option<Vec<String>>),
    Set,
    Get,
    Add(AddType),
}

#[derive(Debug, Serialize, Deserialize)]
enum Feilds {
    #[serde(rename = "id")]
    Id,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "errorString")]
    ErrorString,
    #[serde(rename = "eta")]
    Eta,
    #[serde(rename = "isFinished")]
    IsFinished,
    #[serde(rename = "isStalled")]
    IsStalled,
    #[serde(rename = "leftUntilDone")]
    LeftUntilDone,
    #[serde(rename = "metadataPercentComplete")]
    MetadataPercentComplete,
    #[serde(rename = "peersConnected")]
    PeersConnected,
    #[serde(rename = "peersGettingFromUs")]
    PeersGettingFromUs,
    #[serde(rename = "peersSendingToUs")]
    PeersSendingToUs,
    #[serde(rename = "percentDone")]
    PercentDone,
    #[serde(rename = "queuePosition")]
    QueuePosition,
    #[serde(rename = "rateDownload")]
    RateDownload,
    #[serde(rename = "rateUpload")]
    RateUpload,
    #[serde(rename = "recheckProgress")]
    RecheckProgress,
    #[serde(rename = "seedRatioMode")]
    SeedRatioMode,
    #[serde(rename = "seedRatioLimit")]
    SeedRatioLimit,
    #[serde(rename = "sizeWhenDone")]
    SizeWhenDone,
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "trackers")]
    Trackers,
    #[serde(rename = "downloadDir")]
    DownloadDir,
    #[serde(rename = "uploadedEver")]
    UploadedEver,
    #[serde(rename = "uploadRatio")]
    UploadRatio,
    #[serde(rename = "webseedsSendingToUs")]
    WebseedsSendingToUs,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Arguments {
    ids: Option<Vec<String>>,
    fields: Option<Vec<Feilds>>,
    filename: Option<String>,
    metainfo: Option<String>,
}

impl Arguments {
    fn standard() -> Self {
        Self {
            fields: Some(vec![
                Feilds::Id,
                Feilds::Error,
                Feilds::ErrorString,
                Feilds::Eta,
                Feilds::IsStalled,
                Feilds::IsFinished,
                Feilds::QueuePosition,
                Feilds::PercentDone,
            ]),
            ..Self::default()
        }
    }
    fn new(ids: &Option<Vec<String>>) -> Self {
        Self {
            ids: ids.to_owned(),
            ..Self::standard()
        }
    }
    fn add_file(add: &AddType) -> Self {
        match add {
            AddType::Meta(metainfo) => Self {
                metainfo: Some(metainfo.to_owned()),
                ..Self::default()
            },
            AddType::FileName(file_name) => Self {
                filename: Some(file_name.to_owned()),
                ..Self::default()
            },
        }
    }
}

#[derive(Serialize, Debug)]
pub struct RpcAction {
    method: String,
    arguments: Arguments,
}

impl TorrentActions {
    pub fn to_action(&self) -> RpcAction {
        match self {
            Self::Stop(id) => RpcAction {
                method: "torrent-stop".to_string(),
                arguments: Arguments::new(id),
            },
            Self::Start(id) => RpcAction {
                method: "torrent-start".to_string(),
                arguments: Arguments::new(id),
            },

            Self::StartNow(id) => RpcAction {
                method: "torrent-start-now".to_string(),
                arguments: Arguments::new(id),
            },

            Self::Verify(id) => RpcAction {
                method: "torrent-verify".to_string(),
                arguments: Arguments::new(id),
            },

            Self::Reannounce(id) => RpcAction {
                method: "torrent-reannounce".to_string(),
                arguments: Arguments::new(id),
            },
            Self::Set => RpcAction {
                method: "".to_string(),
                arguments: Arguments::standard(),
            },
            Self::Get => RpcAction {
                method: "".to_string(),
                arguments: Arguments::standard(),
            },
            Self::Add(action_type) => RpcAction {
                method: "torrent-add".to_string(),
                arguments: Arguments::add_file(action_type),
            },
        }
    }
}
