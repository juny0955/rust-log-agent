use serde::Serialize;

#[derive(Serialize)]
pub struct LogBody {
    name: String,
    data: String,
}

impl LogBody {
    pub fn new(name: &str, data: &str) -> Self {
        Self { 
            name: name.to_string(),
            data: data.to_string(),
        }
    }
}