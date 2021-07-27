use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub profile: Profile,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub display_name: String,
    pub real_name: String,
}
