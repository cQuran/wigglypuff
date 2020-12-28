use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseBody<T> {
    pub success: bool,
    pub data: T,
}

impl<T> ResponseBody<T> {
    pub fn new(is_success: bool, data: T) -> ResponseBody<T> {
        ResponseBody {
            success: is_success,
            data,
        }
    }
}
