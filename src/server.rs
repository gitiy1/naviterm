use reqwest::header::{CONTENT_TYPE,ACCEPT};
use chrono;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::Client;
use md5;

use crate::app::AppResult;
use crate::parser::Parser;

#[derive(Debug)]
pub struct Server{
    pub server_address: String,
    pub server_version: String,
    /// server token
    pub token: String,
    /// salt
    pub salt: String,
    pub connection_status: String,
    pub connection_message: String,
    pub connection_code: String,
    pub last_connection_timestamp: String,
    /// user
    pub user: String,
    /// password
    password: String,
    /// http client
    client: Client,

}

impl Default for Server{
    fn default() -> Self {
        Self {
            token: "".to_string(),
            salt: "".to_string(),
            connection_status: "".to_string(),
            connection_message: "".to_string(),
            connection_code: "".to_string(),
            last_connection_timestamp: "".to_string(),
            server_address: "".to_string(),
            server_version: "".to_string(),
            user: "".to_string(),
            password: "".to_string(),
            client: Client::new(),
        }
    }
}

impl Server{
    
    /// Constructs a new instance of [`Server`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn renew_credentials(&mut self) -> AppResult<()> {
        let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 10).to_lowercase();
        let mut concatenation: String = String::from(&self.password);
        concatenation.push_str(&salt);
        let token = md5::compute(concatenation.as_bytes());

        self.salt = salt;
        self.token = format!("{:x}", token);

        Ok(())
    }

    pub async fn test_connection(&mut self) -> AppResult<()> {
        let url = format!("{}/navidrome/rest/ping.view?u={}&t={}&s={}&v=0.1&c=naviterm",
                          self.server_address, self.user, self.token, self.salt);
        let response = self.client.get(url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .send().await;

        match response {
            Ok(success_response) => match success_response.status() {
                reqwest::StatusCode::OK => {
                    let response_text = success_response.text().await.unwrap();
                    let connection_status = Parser::parse_connection_status(response_text).unwrap();
                    self.connection_status = connection_status.status().to_string();
                    self.server_version = connection_status.server_version().to_string();
                    self.connection_code = connection_status.error_code().to_string();
                    self.connection_message = connection_status.error_message().to_string();
                    self.last_connection_timestamp = chrono::offset::Local::now().to_string();
                },
                reqwest::StatusCode::UNAUTHORIZED => {
                    println!("Need to grab a new token");
                },
                _ => {
                    panic!("Uh oh! Something unexpected happened.");
                },
            },
            Err(error) => panic!("Error while doing request: {:?}", error)
        };

        Ok(())
    }

    pub fn set_password(&mut self, password: String) {
        self.password = password;
    }
}

