use crate::app::AppResult;
use crate::model::album::Album;
use crate::model::artist::Artist;
use crate::model::connection_status::ConnectionStatus;
use crate::model::playlist::Playlist;
use crate::model::song::Song;
use crate::server::json_parser::JsonParser;
use crate::server::xml_parser::XmlParser;

#[derive(Debug, Copy, Clone)]
pub enum Parser {
    XmlParser,
    JsonParser
}

impl Default for Parser {
    fn default() -> Self {
        Parser::JsonParser
    }
}

impl Parser {

    pub fn parse_connection_status(response: String, parser_type: Parser) -> AppResult<ConnectionStatus> {
        match parser_type {
            Parser::XmlParser => XmlParser::parse_connection_status(response),
            Parser::JsonParser => JsonParser::parse_connection_status(response)
        }
    }

    pub fn parse_genres_list(response: String, parser_type: Parser) -> AppResult<Vec<String>> {
        match parser_type {
            Parser::XmlParser => XmlParser::parse_genres_list(response),
            Parser::JsonParser => JsonParser::parse_genres_list(response)
        }
    }

    pub fn parse_album_list_simple(response: String, api_version: &str, parser_type: Parser) -> AppResult<Vec<String>> {
        match parser_type {
            Parser::XmlParser => XmlParser::parse_album_list_simple(response, api_version),
            Parser::JsonParser => JsonParser::parse_album_list_simple(response, api_version),
        }
    }

    pub fn parse_album(response: String, parser_type: Parser) -> AppResult<(Album, Vec<Song>, Artist)> {
        match parser_type {
            Parser::XmlParser => XmlParser::parse_album(response),
            Parser::JsonParser => JsonParser::parse_album(response),
        }
    }

    pub fn parse_playlist_list(response: String, parser_type: Parser) -> AppResult<Vec<Playlist>> {
        match parser_type {
            Parser::XmlParser => XmlParser::parse_playlist_list(response),
            Parser::JsonParser => JsonParser::parse_playlist_list(response)
        }
    }

    pub fn parse_playlist(response: String, parser_type: Parser) -> AppResult<Vec<String>> {
        match parser_type {
            Parser::XmlParser => XmlParser::parse_playlist(response),
            Parser::JsonParser => JsonParser::parse_playlist(response)
        }
    }

    pub fn parse_playlist_id(response: String, parser_type: Parser) -> AppResult<String> {
        match parser_type {
            Parser::XmlParser => XmlParser::parse_playlist_id(response),
            Parser::JsonParser => JsonParser::parse_playlist_id(response)
        }
    }
}

