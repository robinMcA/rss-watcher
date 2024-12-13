use anyhow::Result as AnyResult;
use std::{fs, path::PathBuf, sync::Arc};

use regex::Regex;

use super::types::MovieConfig;

trait TransmissionFilters {
    fn is_movie(&self) -> bool;
    fn get_season(&self) -> Option<i8>;
    fn get_episode(&self) -> Option<String>;
    fn get_show_name(&self) -> Option<String>;
}

impl TransmissionFilters for PathBuf {
    fn is_movie(&self) -> bool {
        let pattern = Regex::new(r"[sS](\d{2})").unwrap();
        match self.file_name() {
            None => false,
            Some(filename) => {

                filename.to_str().map(|t| !pattern.is_match(t)).unwrap_or(true)

            }
        }
    }

    fn get_season(&self) -> Option<i8> {
        let pattern = Regex::new(r"[sS](\d{2})").unwrap();
        match self.file_name() {
            None => None,
            Some(filename) => {
                let season_start = pattern.captures(filename.to_str()?)?.get(1)?.as_str();
                season_start.parse::<i8>().ok()
            }
        }
    }

    fn get_episode(&self) -> Option<String> {
        Some(self.file_name()?.to_str()?.to_string())
    }

    fn get_show_name(&self) -> Option<String> {
        let pattern = Regex::new(r"[sS](\d{2})").unwrap();
        if self.is_movie() {
            return None;
        };
        match self.file_name() {
            None => None,
            Some(filename) => {
                let season_start = pattern.find(filename.to_str()?)?.start();
                Some(filename.to_str().unwrap_or("")[0..season_start - 1].to_string())
            }
        }
    }
}

pub fn generate_target_path(path_buf: &PathBuf, movie_config: Arc<MovieConfig>) -> Option<PathBuf> {
    let movie = movie_config.movie_dir.to_owned()?;
    let tv = movie_config.tv_dir.to_owned()?;

    if path_buf.is_movie() {
        Some(PathBuf::from(format!(
            "{}/{}",
            movie,
            path_buf.get_episode()?
        )))
    } else if path_buf.is_dir() {
        Some(PathBuf::from(format!(
            "{}/{}/{:02}",
            tv,
            path_buf.get_show_name()?,
            path_buf.get_season()?,
        )))
    } else {
        Some(PathBuf::from(format!(
            "{}/{}/{:02}/{}",
            tv,
            path_buf.get_show_name()?,
            path_buf.get_season()?,
            path_buf.get_episode()?,
        )))
    }
}

fn copy_directory(src: &PathBuf, dst: PathBuf) -> AnyResult<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_directory(&entry.path(), dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn copy_file(path: &PathBuf, config: Arc<MovieConfig>) -> AnyResult<()> {
    let save_location = config.save_path();
    let target = generate_target_path(path, Arc::clone(&config));
    let final_location = save_location.join(target.unwrap());
    let target_dir = final_location.parent().unwrap();
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }
    if path.is_dir() {
        copy_directory(path, final_location)?;
        fs::remove_dir_all(path).unwrap_or_default();
    } else {
        fs::copy(path, final_location)?;
        fs::remove_file(path).unwrap_or_default()
    }
    Ok(())
}
