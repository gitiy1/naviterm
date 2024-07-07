use crate::app::AppResult;
use crate::parser::model::ConnectionStatus;

mod model;
pub struct Parser {}

impl Parser {

    pub fn parse_connection_status (response: String) -> AppResult<ConnectionStatus> {
        let root: minidom::Element = response.parse().unwrap();
        let mut connection_status: ConnectionStatus = ConnectionStatus::default() ;

        for attribute in root.attrs() {
            if attribute.0 == "status"{
                connection_status.set_status(attribute.1.to_string());
            }
            else if attribute.0 == "serverVersion" {
                connection_status.set_server_version(attribute.1.to_string());
            }

        }

        for children in root.children() {
            for attribute in children.attrs() {
                if attribute.0 == "code"{
                    connection_status.set_error_code(attribute.1.to_string());
                }
                else if attribute.0 == "message" {
                    connection_status.set_error_message(attribute.1.to_string());
                }
            }
        }

        Ok(connection_status)

    }

}