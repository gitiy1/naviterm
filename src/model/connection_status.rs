#[derive(Debug, Default)]
pub struct ConnectionStatus {
    status: String,
    server_version: String,
    error_code: String,
    error_message: String,
}

impl ConnectionStatus {
    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn server_version(&self) -> &str {
        &self.server_version
    }

    pub fn error_code(&self) -> &str {
        &self.error_code
    }

    pub fn error_message(&self) -> &str {
        &self.error_message
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn set_server_version(&mut self, server_version: String) {
        self.server_version = server_version;
    }

    pub fn set_error_code(&mut self, error_code: String) {
        self.error_code = error_code;
    }

    pub fn set_error_message(&mut self, error_message: String) {
        self.error_message = error_message;
    }
}
