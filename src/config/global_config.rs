use std::fmt::{Display, Formatter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub end_point: String,
    pub send_type: SendType,
}

#[derive(Debug, Deserialize)]
pub enum SendType {
    HTTP,
}

impl Display for GlobalConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "=====Global Config=====\nend_point: {}, send_type: {:?}", self.end_point, self.send_type)
    }
}