use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.to_string(),
        }
    }
}

impl ApiResponse<()> {
    pub fn success_no_data(message: &str) -> Self {
        Self {
            success: true,
            data: Some(()),
            message: message.to_string(),
        }
    }
}