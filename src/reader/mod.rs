mod messages;
mod timestamp;
mod user;

use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::prelude::*;
use lru::LruCache;
pub use messages::*;
use serde::{Deserialize, Serialize};
pub use timestamp::floating_timestamp;
pub use user::*;

#[derive(Eq, PartialEq, Hash)]
struct MessagesChannelId {
    channel_id: String,
    date: NaiveDate,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
}

pub struct MessagesReader {
    data_path: PathBuf,
    channels: HashMap<String, ChannelInfo>,
    users: HashMap<String, User>,
}

impl MessagesReader {
    pub fn new(data_path: PathBuf) -> Self {
        Self {
            channels: Self::parse_channels(&data_path).unwrap(),
            users: Self::parse_users(&data_path).unwrap(),
            data_path,
        }
    }

    pub fn channel_messages_parse(
        &self,
        channel_id: &str,
        date: &NaiveDate,
    ) -> Result<ChannelMessages> {
        let channel_info = self
            .channels
            .get(channel_id)
            .ok_or(anyhow!("Channel not found"))?;

        let data_path = self
            .data_path
            .join(&channel_info.name)
            .join(format!("{}.json", date));
        let messages = serde_json::from_str(
            &read_to_string(data_path).context(anyhow!("Failed to read day"))?,
        )?;

       Ok(messages)
    }

    fn parse_channels(data_path: &PathBuf) -> Result<HashMap<String, ChannelInfo>> {
        let channels: Vec<ChannelInfo> = serde_json::from_str(
            &read_to_string(data_path.join("channels.json")).context("Failed to load channels")?,
        )
        .context("Failed to parse channels list")?;

        let mut channels_map = HashMap::new();
        for channel in channels {
            channels_map.insert(channel.id.to_owned(), channel);
        }

        Ok(channels_map)
    }

    fn parse_users(data_path: &PathBuf) -> Result<HashMap<String, User>> {
        let users: Vec<User> = serde_json::from_str(
            &read_to_string(data_path.join("users.json")).context("Failed to load users")?,
        )
        .context("Failed to parse users")?;

        let mut users_map = HashMap::new();
        for user in users {
            users_map.insert(user.id.to_owned(), user);
        }

        Ok(users_map)
    }

    pub fn get_channel_name(&self, channel_id: &str) -> Result<&str> {
        self.channels
            .get(channel_id)
            .map(|ch| ch.name.as_str())
            .ok_or(anyhow!("Channel not found"))
    }

    pub fn get_user_info(&self, user_id: &str) -> Result<&User> {
        self.users.get(user_id).ok_or(anyhow!("User not found"))
    }

    pub fn list_channels(&self) -> Vec<&ChannelInfo> {
        self.channels.values().collect()
    }

    pub fn list_dates(&self, channel_id: &str) -> Result<Vec<NaiveDate>> {
        let channel_name = self.get_channel_name(channel_id)?;
        let mut dates = Vec::new();
        for dir in std::fs::read_dir(self.data_path.join(channel_name))
            .with_context(|| format!("Failed to read channel: {}", channel_name))?
        {
            let entry =
                dir.with_context(|| format!("Failed to read channel dir: {}", channel_name))?;
            let file_path = entry.path();
            if file_path.is_file() {
                let file_name = file_path.file_name().unwrap();
                let date = NaiveDate::parse_from_str(&file_name.to_string_lossy(), "%Y-%m-%d.json");
                match date {
                    Ok(date) => {
                        dates.push(date);
                    }
                    Err(err) => {
                        log::warn!(
                            "Invalid date in channel {} history: {:#}",
                            channel_name,
                            err
                        );
                    }
                }
            }
        }

        Ok(dates)
    }
}
