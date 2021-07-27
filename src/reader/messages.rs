use super::floating_timestamp;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct ChannelMessages {
    pub messages: Vec<Entry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    #[serde(flatten)]
    pub details: Option<MessageDetails>,
    #[serde(flatten)]
    pub specific: Specific,
    pub text: String,
    #[serde(rename = "user")]
    pub user_id: String,
    #[serde(rename = "ts")]
    #[serde(with = "floating_timestamp")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageDetails {
    pub client_msg_id: String,
    #[serde(rename = "user_team")]
    pub user_team_id: String,
    #[serde(rename = "source_team")]
    pub source_team_id: String,
    pub user_profile: UserProfile,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    pub avatar_hash: String,
    pub image_72: String,
    pub first_name: String,
    pub real_name: String,
    pub display_name: String,
    #[serde(rename = "team")]
    pub team_id: String,
    pub name: String,
    #[serde(rename = "is_restricted")]
    pub restricted: bool,
    #[serde(rename = "is_ultra_restricted")]
    pub ultra_restricted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Specific {
    Message(Message),
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parsing() {
        let test_json = include_str!("./test_data/msg_test.json");
        let _parsed: super::ChannelMessages = serde_json::from_str(test_json).unwrap();
    }
}
