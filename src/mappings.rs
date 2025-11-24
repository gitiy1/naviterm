use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Debug;
use config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::{debug, warn};
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum ShortcutAction {
    AddItemEnd,
    AddItemNext,
    AddItemPlaylist,
    CycleLoopMode,
    CyclePane,
    DeleteItemFromPlaylist,
    GoAlbumPane,
    GoArtistPane,
    GoFirstInList,
    GoHomePane,
    GoLastInList,
    GoPlaylistPane,
    GoPopupAddAlbumTo,
    GoPopupAddArtistItemTo,
    GoPopupAddArtistTo,
    GoPopupAddPlaylistTo,
    GoPopupAddSongTo,
    GoPopupAlbumInfo,
    GoPopupDeletePlaylist,
    GoPopupGenreFilter,
    GoPopupSyncPlaylist,
    GoPopupTestConnection,
    GoPopupUpdateDatabase,
    GoPopupGlobalSearch,
    GoPopupYearFilter,
    GoQueuePane,
    GoToTrackAlbum,
    GoToTrackArtist,
    MoveDownInList,
    MovePageDown,
    MovePageUp,
    MovePaneDown,
    MovePaneLeft,
    MovePaneRight,
    MovePaneUp,
    MoveSelectionDown,
    MoveSelectionUp,
    MoveUpInList,
    None,
    PlayImmediatelyAlbum,
    PlayImmediatelyPlaylist,
    PlayImmediatelySong,
    PlayImmediatelyArtist,
    PlayImmediatelyArtistItem,
    PopupClose,
    PopupConfirmDeletionPlaylistNo,
    PopupConfirmDeletionPlaylistYes,
    PopupConnectionErrorRetry,
    PopupConnectionErrorOffline,
    PopupGenreAcceptSelected,
    PopupGenreSelectFavorite,
    PopupGenreToggleFavorite,
    PopupPlaylistAcceptPlaylistName,
    PopupPlaylistAcceptSelected,
    PopupPlaylistAddCharToPlaylistName,
    PopupPlaylistCancelNewPlaylist,
    PopupPlaylistRemoveCharFromPlaylistName,
    PopupSynchronizePlaylistPushLocal,
    PopupSynchronizePlaylistPullRemote,
    PopupSynchronizeLocalPlaylistPushYes,
    PopupSynchronizeLocalPlaylistPushNo,
    PopupTestConnectionGenerate,
    PopupTestConnectionTest,
    PopupUpdateDatabaseUpdateAlbums,
    PopupUpdateDatabaseUpdateAllFull,
    PopupUpdateDatabaseUpdateAllQuick,
    PopupUpdateDatabaseUpdateCurrentlySelected,
    PopupUpdateDatabaseUpdatePlaylists,
    PopupYearAcceptFilter,
    PopupYearAddDigit,
    PopupYearRemoveDigit,
    PopupYearToggleFromTo,
    PopupYearToggleRangeInput,
    PopupYearClearAndClose,
    QueueCenterCursor,
    QueueClear,
    QueueDeleteSong,
    QueuePlaySong,
    QuitApp,
    SearchAccept,
    SearchAddCharToSearchString,
    SearchClear,
    SearchEnd,
    SearchRemoveCharFromSearchString,
    SearchStart,
    SearchGoNext,
    SearchGoPrevious,
    SeekBackwards,
    SeekForward,
    StopPlayback,
    TogglePlayPause,
    ToggleRandomPlayback,
    ToggleSortMethod,
    ToggleSortOrder,
    TrackNext,
    TrackPrevious,
    VolumeDown,
    VolumeUp,
    PopupGlobalSearchAddCharToSearchString,
    PopupGlobalSearchRemoveCharFromSearchString,
    PopupGlobalSearchAcceptSearchString,
    PopupGlobalSearchClearAndClose,
    PopupGlobalSearchPlayItem,
    PopupGlobalSearchAddItemTo,
    PopupGlobalSearchGoToAccordingPane,
}

pub struct Mappings {
    // Keys for the hashmap follow the pattern: subpane_pane_popup_flag_modifier_key
    // For instance: left_albums_none_none_ctrl_l
    // For global shortcuts, popup, pane and subpane slice are absent
    // For popup shortcuts, pane and subpane slice are absent
    // Modifier can be "none" or "ctrl"
    mappings: HashMap<String, ShortcutAction>,
    old_mappings: HashMap<String, ShortcutAction>,
    mappings_to_remove: Vec<String>,
}

impl Default for Mappings {
    fn default() -> Self {
        Self::new()
    }
}

impl Mappings {
    pub fn new() -> Self {
        Mappings {
            old_mappings: HashMap::new(),
            mappings_to_remove: vec![],
            mappings: HashMap::from([
                (String::from("Home_none_none_none_i"),ShortcutAction::GoPopupAlbumInfo),
                (String::from("Home_none_none_none_f1"),ShortcutAction::GoPopupTestConnection),
                (String::from("bottom_right_Home_none_none_none_a"),ShortcutAction::GoPopupAddSongTo),
                (String::from("bottom_right_Home_none_none_none_enter"),ShortcutAction::PlayImmediatelySong),
                (String::from("bottom_left_Home_none_none_none_a"),ShortcutAction::GoPopupAddAlbumTo),
                (String::from("bottom_left_Home_none_none_none_enter"),ShortcutAction::PlayImmediatelyAlbum),
                (String::from("top_left_Home_none_none_none_a"),ShortcutAction::GoPopupAddAlbumTo),
                (String::from("top_left_Home_none_none_none_enter"),ShortcutAction::PlayImmediatelyAlbum),
                (String::from("top_right_Home_none_none_none_a"),ShortcutAction::GoPopupAddAlbumTo),
                (String::from("top_right_Home_none_none_none_enter"),ShortcutAction::PlayImmediatelyAlbum),
                (String::from("Albums_none_none_none_e"),ShortcutAction::GoPopupGenreFilter),
                (String::from("Albums_none_none_none_y"),ShortcutAction::GoPopupYearFilter),
                (String::from("Albums_none_none_none_m"),ShortcutAction::ToggleSortMethod),
                (String::from("Albums_none_none_none_r"),ShortcutAction::ToggleSortOrder),
                (String::from("left_Albums_none_none_none_a"),ShortcutAction::GoPopupAddAlbumTo),
                (String::from("left_Albums_none_none_none_enter"),ShortcutAction::PlayImmediatelyAlbum),
                (String::from("right_Albums_none_none_none_a"),ShortcutAction::GoPopupAddSongTo),
                (String::from("right_Albums_none_none_none_enter"),ShortcutAction::PlayImmediatelySong),
                (String::from("right_Albums_none_none_none_A"),ShortcutAction::GoPopupAddAlbumTo),
                (String::from("Playlists_none_none_none_s"),ShortcutAction::GoPopupSyncPlaylist),
                (String::from("Playlists_none_none_none_d"),ShortcutAction::GoPopupDeletePlaylist),
                (String::from("left_Playlists_none_none_none_a"),ShortcutAction::GoPopupAddPlaylistTo),
                (String::from("left_Playlists_none_none_none_enter"),ShortcutAction::PlayImmediatelyPlaylist),
                (String::from("right_Playlists_none_none_none_a"),ShortcutAction::GoPopupAddSongTo),
                (String::from("right_Playlists_none_none_none_d"),ShortcutAction::DeleteItemFromPlaylist),
                (String::from("right_Playlists_none_none_none_A"),ShortcutAction::GoPopupAddPlaylistTo),
                (String::from("right_Playlists_none_none_none_enter"),ShortcutAction::PlayImmediatelySong),
                (String::from("right_Playlists_none_none_none_J"),ShortcutAction::MoveSelectionDown),
                (String::from("right_Playlists_none_none_none_K"),ShortcutAction::MoveSelectionUp),
                (String::from("left_Artists_none_none_none_a"),ShortcutAction::GoPopupAddArtistTo),
                (String::from("left_Artists_none_none_none_enter"),ShortcutAction::PlayImmediatelyArtist),
                (String::from("right_Artists_none_none_none_a"),ShortcutAction::GoPopupAddArtistItemTo),
                (String::from("right_Artists_none_none_none_enter"),ShortcutAction::PlayImmediatelyArtistItem),
                (String::from("right_Artists_none_none_none_A"),ShortcutAction::GoPopupAddAlbumTo),
                (String::from("Queue_none_none_none_>"),ShortcutAction::TrackNext),
                (String::from("Queue_none_none_none_<"),ShortcutAction::TrackPrevious),
                (String::from("Queue_none_none_none_a"),ShortcutAction::GoToTrackAlbum),
                (String::from("Queue_none_none_none_r"),ShortcutAction::GoToTrackArtist),
                (String::from("Queue_none_none_none_e"),ShortcutAction::QueueCenterCursor),
                (String::from("Queue_none_none_none_c"),ShortcutAction::QueueClear),
                (String::from("Queue_none_none_none_d"),ShortcutAction::QueueDeleteSong),
                (String::from("Queue_none_none_none_enter"),ShortcutAction::QueuePlaySong),
                (String::from("none_none_none_tab"),ShortcutAction::CyclePane),
                (String::from("none_none_none_q"),ShortcutAction::QuitApp),
                (String::from("none_none_none_esc"),ShortcutAction::SearchClear),
                (String::from("connection_test_none_none_r"),ShortcutAction::PopupTestConnectionGenerate),
                (String::from("connection_test_none_none_t"),ShortcutAction::PopupTestConnectionTest),
                (String::from("connection_test_none_none_q"),ShortcutAction::PopupClose),
                (String::from("connection_test_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("album_information_none_none_a"),ShortcutAction::GoPopupAddSongTo),
                (String::from("album_information_none_none_A"),ShortcutAction::GoPopupAddAlbumTo),
                (String::from("album_information_none_none_enter"),ShortcutAction::PlayImmediatelySong),
                (String::from("album_information_none_none_q"),ShortcutAction::PopupClose),
                (String::from("album_information_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("add_to_none_none_n"),ShortcutAction::AddItemNext),
                (String::from("add_to_none_none_e"),ShortcutAction::AddItemEnd),
                (String::from("add_to_none_none_p"),ShortcutAction::AddItemPlaylist),
                (String::from("add_to_none_none_q"),ShortcutAction::PopupClose),
                (String::from("add_to_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("genre_filter_none_none_enter"),ShortcutAction::PopupGenreAcceptSelected),
                (String::from("genre_filter_none_none_f"),ShortcutAction::PopupGenreToggleFavorite),
                (String::from("genre_filter_none_none_"),ShortcutAction::PopupGenreSelectFavorite),
                (String::from("genre_filter_none_none_q"),ShortcutAction::PopupClose),
                (String::from("genre_filter_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("year_filter_none_none_"),ShortcutAction::PopupYearAddDigit),
                (String::from("year_filter_none_none_bkspc"),ShortcutAction::PopupYearRemoveDigit),
                (String::from("year_filter_none_none_enter"),ShortcutAction::PopupYearAcceptFilter),
                (String::from("year_filter_none_none_r"),ShortcutAction::PopupYearToggleRangeInput),
                (String::from("year_filter_none_none_q"),ShortcutAction::PopupClose),
                (String::from("year_filter_none_none_esc"),ShortcutAction::PopupYearClearAndClose),
                (String::from("year_filter_range_year_none_"),ShortcutAction::PopupYearAddDigit),
                (String::from("year_filter_range_year_none_bkspc"),ShortcutAction::PopupYearRemoveDigit),
                (String::from("year_filter_range_year_none_tab"),ShortcutAction::PopupYearToggleFromTo),
                (String::from("year_filter_range_year_none_enter"),ShortcutAction::PopupYearAcceptFilter),
                (String::from("year_filter_range_year_none_r"),ShortcutAction::PopupYearToggleRangeInput),
                (String::from("year_filter_range_year_none_q"),ShortcutAction::PopupClose),
                (String::from("year_filter_range_year_none_esc"),ShortcutAction::PopupYearClearAndClose),
                (String::from("update_database_none_none_b"),ShortcutAction::PopupUpdateDatabaseUpdateAlbums),
                (String::from("update_database_none_none_y"),ShortcutAction::PopupUpdateDatabaseUpdatePlaylists),
                (String::from("update_database_none_none_s"),ShortcutAction::PopupUpdateDatabaseUpdateAllQuick),
                (String::from("update_database_none_none_a"),ShortcutAction::PopupUpdateDatabaseUpdateAllFull),
                (String::from("update_database_none_none_enter"),ShortcutAction::PopupUpdateDatabaseUpdateCurrentlySelected),
                (String::from("update_database_none_none_q"),ShortcutAction::PopupClose),
                (String::from("update_database_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("select_playlist_introducing_playlist_none_"),ShortcutAction::PopupPlaylistAddCharToPlaylistName),
                (String::from("select_playlist_introducing_playlist_none_bkspc"),ShortcutAction::PopupPlaylistRemoveCharFromPlaylistName),
                (String::from("select_playlist_introducing_playlist_none_enter"),ShortcutAction::PopupPlaylistAcceptPlaylistName),
                (String::from("select_playlist_introducing_playlist_none_esc"),ShortcutAction::PopupPlaylistCancelNewPlaylist),
                (String::from("select_playlist_none_none_enter"),ShortcutAction::PopupPlaylistAcceptSelected),
                (String::from("select_playlist_none_none_q"),ShortcutAction::PopupClose),
                (String::from("select_playlist_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("synchronize_playlist_none_none_r"),ShortcutAction::PopupSynchronizePlaylistPullRemote),
                (String::from("synchronize_playlist_none_none_l"),ShortcutAction::PopupSynchronizePlaylistPushLocal),
                (String::from("synchronize_playlist_none_none_y"),ShortcutAction::PopupSynchronizeLocalPlaylistPushYes),
                (String::from("synchronize_playlist_none_none_n"),ShortcutAction::PopupSynchronizeLocalPlaylistPushNo),
                (String::from("synchronize_playlist_none_none_q"),ShortcutAction::PopupClose),
                (String::from("synchronize_playlist_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("confirm_playlist_deletion_none_none_y"),ShortcutAction::PopupConfirmDeletionPlaylistYes),
                (String::from("confirm_playlist_deletion_none_none_n"),ShortcutAction::PopupConfirmDeletionPlaylistNo),
                (String::from("confirm_playlist_deletion_none_none_q"),ShortcutAction::PopupClose),
                (String::from("confirm_playlist_deletion_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("connection_error_none_none_r"),ShortcutAction::PopupConnectionErrorRetry),
                (String::from("connection_error_none_none_o"),ShortcutAction::PopupConnectionErrorOffline),
                (String::from("connection_error_none_none_q"),ShortcutAction::PopupClose),
                (String::from("connection_error_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("searching_none_enter"),ShortcutAction::SearchAccept),
                (String::from("searching_none_esc"),ShortcutAction::SearchEnd),
                (String::from("searching_none_bkspc"),ShortcutAction::SearchRemoveCharFromSearchString),
                (String::from("searching_none_"),ShortcutAction::SearchAddCharToSearchString),
                (String::from("none_none_none_ "),ShortcutAction::TogglePlayPause),
                (String::from("none_none_none_u"),ShortcutAction::GoPopupUpdateDatabase),
                (String::from("none_none_ctrl_f"),ShortcutAction::GoPopupGlobalSearch),
                (String::from("global_search_none_ctrl_f"),ShortcutAction::GoPopupGlobalSearch),
                (String::from("global_search_introducing_global_none_"),ShortcutAction::PopupGlobalSearchAddCharToSearchString),
                (String::from("global_search_introducing_global_none_bkspc"),ShortcutAction::PopupGlobalSearchRemoveCharFromSearchString),
                (String::from("global_search_introducing_global_none_enter"),ShortcutAction::PopupGlobalSearchAcceptSearchString),
                (String::from("global_search_introducing_global_none_esc"),ShortcutAction::PopupGlobalSearchClearAndClose),
                (String::from("global_search_none_none_tab"),ShortcutAction::CyclePane),
                (String::from("global_search_none_ctrl_h"),ShortcutAction::MovePaneLeft),
                (String::from("global_search_none_ctrl_l"),ShortcutAction::MovePaneRight),
                (String::from("global_search_none_ctrl_j"),ShortcutAction::MovePaneDown),
                (String::from("global_search_none_ctrl_k"),ShortcutAction::MovePaneUp),
                (String::from("global_search_none_none_enter"),ShortcutAction::PopupGlobalSearchPlayItem),
                (String::from("global_search_none_none_a"),ShortcutAction::PopupGlobalSearchAddItemTo),
                (String::from("global_search_none_none_r"),ShortcutAction::PopupGlobalSearchGoToAccordingPane),
                (String::from("global_search_none_none_q"),ShortcutAction::PopupClose),
                (String::from("global_search_none_none_esc"),ShortcutAction::PopupGlobalSearchClearAndClose),
                (String::from("error_message_none_none_q"),ShortcutAction::PopupClose),
                (String::from("error_message_none_none_esc"),ShortcutAction::PopupClose),
                (String::from("none_none_none_z"),ShortcutAction::ToggleRandomPlayback),
                (String::from("none_none_none_l"),ShortcutAction::CycleLoopMode),
                (String::from("none_none_none_right"),ShortcutAction::SeekForward),
                (String::from("none_none_none_left"),ShortcutAction::SeekBackwards),
                (String::from("none_none_none_up"),ShortcutAction::VolumeUp),
                (String::from("none_none_none_down"),ShortcutAction::VolumeDown),
                (String::from("none_none_j"),ShortcutAction::MoveDownInList),
                (String::from("none_none_k"),ShortcutAction::MoveUpInList),
                (String::from("none_none_g"),ShortcutAction::GoFirstInList),
                (String::from("none_none_G"),ShortcutAction::GoLastInList),
                (String::from("none_none_none_1"),ShortcutAction::GoHomePane),
                (String::from("none_none_none_2"),ShortcutAction::GoAlbumPane),
                (String::from("none_none_none_3"),ShortcutAction::GoPlaylistPane),
                (String::from("none_none_none_4"),ShortcutAction::GoArtistPane),
                (String::from("none_none_none_5"),ShortcutAction::GoQueuePane),
                (String::from("none_none_none_/"),ShortcutAction::SearchStart),
                (String::from("none_none_none_n"),ShortcutAction::SearchGoNext),
                (String::from("none_none_none_N"),ShortcutAction::SearchGoPrevious),
                (String::from("none_none_none_o"),ShortcutAction::StopPlayback),
                (String::from("none_none_ctrl_h"),ShortcutAction::MovePaneLeft),
                (String::from("none_none_ctrl_l"),ShortcutAction::MovePaneRight),
                (String::from("none_none_ctrl_j"),ShortcutAction::MovePaneDown),
                (String::from("none_none_ctrl_k"),ShortcutAction::MovePaneUp),
                (String::from("none_ctrl_d"),ShortcutAction::MovePageDown),
                (String::from("none_ctrl_u"),ShortcutAction::MovePageUp),
                ])
        }
    }
    
    pub fn init_shortcuts(&mut self, config: Config) {

        if let Ok(value) = config.get::<String>("move_pane_right") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MovePaneRight, None);
            }
        }

        if let Ok(value) = config.get::<String>("move_pane_left") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MovePaneLeft, None);
            }
        }

        if let Ok(value) = config.get::<String>("move_pane_up") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MovePaneUp, None);
            }
        }

        if let Ok(value) = config.get::<String>("move_pane_down") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MovePaneDown, None);
            }
        }

        if let Ok(value) = config.get::<String>("cycle_subpane") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::CyclePane, None);
            }
        }

        if let Ok(value) = config.get::<String>("move_list_down") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MoveDownInList, None);
            }
        }

        if let Ok(value) = config.get::<String>("move_list_up") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MoveUpInList, None);
            }
        }

        if let Ok(value) = config.get::<String>("move_to_first") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoFirstInList, None);
            }
        }

        if let Ok(value) = config.get::<String>("move_to_last") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoLastInList, None);
            }
        }

        if let Ok(value) = config.get::<String>("move_page_up") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MovePageUp, None);
            }
        }

        if let Ok(value) = config.get::<String>("move_page_down") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MovePageDown, None);
            }
        }

        if let Ok(value) = config.get::<String>("volume_up") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::VolumeUp, None);
            }
        }

        if let Ok(value) = config.get::<String>("volume_down") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::VolumeDown, None);
            }
        }

        if let Ok(value) = config.get::<String>("seek_forward") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::SeekForward, None);
            }
        }

        if let Ok(value) = config.get::<String>("seek_backwards") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::SeekBackwards, None);
            }
        }

        if let Ok(value) = config.get::<String>("track_next") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::TrackNext, None);
            }
        }

        if let Ok(value) = config.get::<String>("track_previous") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::TrackPrevious, None);
            }
        }

        if let Ok(value) = config.get::<String>("add_item_next") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::AddItemNext, None);
            }
        }

        if let Ok(value) = config.get::<String>("add_item_last") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::AddItemEnd, None);
            }
        }

        if let Ok(value) = config.get::<String>("add_item_playlist") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::AddItemPlaylist, None);
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_info") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAlbumInfo, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_test") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupTestConnection, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_add_item_to") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddSongTo, None );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddAlbumTo, Some("Home") );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddAlbumTo, Some("left_Albums") );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddPlaylistTo, Some("left") );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddArtistTo, None );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddArtistItemTo, None );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupGlobalSearchAddItemTo, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_add_parent_to") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddAlbumTo, Some("right_Albums") );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddAlbumTo, Some("right_Artists") );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddAlbumTo, Some("album_information") );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupAddPlaylistTo, Some("right") );
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_genre") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupGenreFilter, None );
            }
        }

        if let Ok(value) = config.get::<String>("genre_toggle_favorite") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupGenreToggleFavorite, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_year") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupYearFilter, None );
            }
        }

        if let Ok(value) = config.get::<String>("year_input_range") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupYearToggleRangeInput, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_sync_playlist") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupSyncPlaylist, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_delete_playlist") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupDeletePlaylist, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_update") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupUpdateDatabase, None );
            }
        }

        if let Ok(value) = config.get::<String>("update_all_quick") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupUpdateDatabaseUpdateAllQuick, None );
            }
        }

        if let Ok(value) = config.get::<String>("update_all_full") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupUpdateDatabaseUpdateAllFull, None );
            }
        }

        if let Ok(value) = config.get::<String>("update_albums") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupUpdateDatabaseUpdateAlbums, None );
            }
        }

        if let Ok(value) = config.get::<String>("update_playlists") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupUpdateDatabaseUpdatePlaylists, None );
            }
        }

        if let Ok(value) = config.get::<String>("update_current") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupUpdateDatabaseUpdateCurrentlySelected, None );
            }
        }

        if let Ok(value) = config.get::<String>("close_popup") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupClose, None );
            }
        }

        if let Ok(value) = config.get::<String>("play_immediately") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PlayImmediatelySong, None );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PlayImmediatelyAlbum, None );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PlayImmediatelyPlaylist, None );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PlayImmediatelyArtist, None );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PlayImmediatelyArtistItem, None );
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupGlobalSearchPlayItem, None );
            }
        }

        if let Ok(value) = config.get::<String>("stop_playback") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::StopPlayback, None );
            }
        }

        if let Ok(value) = config.get::<String>("queue_clear") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::QueueClear, None );
            }
        }

        if let Ok(value) = config.get::<String>("queue_delete_song") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::QueueDeleteSong, None );
            }
        }

        if let Ok(value) = config.get::<String>("queue_center_cursor") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::QueueCenterCursor, None );
            }
        }

        if let Ok(value) = config.get::<String>("toggle_play_pause") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::TogglePlayPause, None );
            }
        }

        if let Ok(value) = config.get::<String>("toggle_random") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::ToggleRandomPlayback, None );
            }
        }

        if let Ok(value) = config.get::<String>("cycle_loop_mode") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::CycleLoopMode, None );
            }
        }

        if let Ok(value) = config.get::<String>("search_start") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::SearchStart, None );
            }
        }

        if let Ok(value) = config.get::<String>("search_accept") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::SearchAccept, None );
            }
        }

        if let Ok(value) = config.get::<String>("search_next") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::SearchGoNext, None );
            }
        }

        if let Ok(value) = config.get::<String>("search_previous") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::SearchGoPrevious, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_pane_home") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoHomePane, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_pane_albums") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoAlbumPane, None );
            }
        }

        if let Ok(value) = config.get::<String>("toggle_album_sort_method") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::ToggleSortMethod, None );
            }
        }

        if let Ok(value) = config.get::<String>("toggle_album_sort_order") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::ToggleSortOrder, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_pane_playlists") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPlaylistPane, None );
            }
        }

        if let Ok(value) = config.get::<String>("playlist_delete_item") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::DeleteItemFromPlaylist, None );
            }
        }

        if let Ok(value) = config.get::<String>("playlist_move_selected_up") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MoveSelectionUp, None );
            }
        }

        if let Ok(value) = config.get::<String>("playlist_move_selected_down") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::MoveSelectionDown, None );
            }
        }

        if let Ok(value) = config.get::<String>("playlist_pull_remote") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupSynchronizePlaylistPullRemote, None );
            }
        }

        if let Ok(value) = config.get::<String>("playlist_push_local") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupSynchronizePlaylistPushLocal, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_pane_artists") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoArtistPane, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_pane_queue") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoQueuePane, None );
            }
        }

        if let Ok(value) = config.get::<String>("queue_go_to_album") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoToTrackAlbum, None );
            }
        }

        if let Ok(value) = config.get::<String>("queue_go_to_artist") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoToTrackArtist, None );
            }
        }

        if let Ok(value) = config.get::<String>("quit_application") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::QuitApp, None );
            }
        }

        if let Ok(value) = config.get::<String>("go_popup_global_search") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::GoPopupGlobalSearch, None );
            }
        }

        if let Ok(value) = config.get::<String>("global_search_go_to_pane") {
            if self.validate_shortcut(value.as_str()) {
                self.use_custom_shortcut(value.as_str(), ShortcutAction::PopupGlobalSearchGoToAccordingPane, None );
            }
        }

        self.clean_old_shortcuts();

    }

    pub fn get_action_from_shortcut(&self, key_event: KeyEvent, pane: &str, subpane: &str, popup: &str, flag: &str) -> ShortcutAction {
        let key_pressed = match key_event.code {
            KeyCode::Backspace => "bkspc".to_string(),
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Left => "left".to_string(),
            KeyCode::Right => "right".to_string(),
            KeyCode::Up => "up".to_string(),
            KeyCode::Down => "down".to_string(),
            KeyCode::Home => "home".to_string(),
            KeyCode::End => "end".to_string(),
            KeyCode::PageUp => "pageup".to_string(),
            KeyCode::PageDown => "pagedown".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::F(i) => format!("f{}", i),
            KeyCode::Char(c) => format!("{}", c),
            KeyCode::Esc => "esc".to_string(),
            _ => "invalid".to_string()
        };

        let mod_used = match key_event.modifiers {
            KeyModifiers::CONTROL => "ctrl".to_string(),
            KeyModifiers::ALT => "alt".to_string(),
            KeyModifiers::SUPER => "super".to_string(),
            _ => "none".to_string()
        };
        
        debug!("subpane:{}, pane:{}, popup:{}, flag:{} mod:{}, char:{}", subpane, pane, popup, flag, mod_used, key_pressed);
        
        let global_key = format!("{}_{}", mod_used, key_pressed);
        if let Some(action) = self.mappings.get(&global_key) {
            return action.clone()
        };

        let flag_string = String::from(flag);
        let flag_key = flag_string.clone() + "_" + global_key.as_str();
        if let Some(action) = self.mappings.get(&flag_key) {
            debug!("Got global shortcut with key: {}", &flag_key);
            return action.clone()
        };
        // Check if we are introducing a single char, we don't have an entry in the hashmap for
        // every possibility
        let mut chars = flag_key.chars().collect::<Vec<char>>();
        chars.pop();
        if chars.last() == Some(&'_') {
            if let Some(action) = self.mappings.get(&chars.iter().collect::<String>()) {
                debug!("Got global shortcut with key: {}", &chars.iter().collect::<String>());
                return action.clone()
            };
        }

        let popup_string = String::from(popup);
        let popup_key = popup_string.clone() + "_" + flag_key.as_str();
        if let Some(action) = self.mappings.get(popup_key.as_str()) {
            debug!("Got popup shortcut with key: {}", &popup_key);
            return action.clone()
        };
        // Check if we are introducing a single char, we don't have an entry in the hashmap for
        // every possibility
        let mut chars = popup_key.chars().collect::<Vec<char>>();
        chars.pop();
        if chars.last() == Some(&'_') {
            if let Some(action) = self.mappings.get(&chars.iter().collect::<String>()) {
                debug!("Got popup shortcut with key: {}", &chars.iter().collect::<String>());
                return action.clone()
            };
        }

        let pane_string = String::from(pane);
        let pane_key = pane_string.clone() + "_" + popup_key.as_str();
        if let Some(action) = self.mappings.get(pane_key.as_str()) {
            debug!("Got pane shortcut with key: {}", &pane_key);
            return action.clone()
        };

        let subpane_string = String::from(subpane);
        let subpane_key = subpane_string + "_" + pane_key.as_str();
        if let Some(action) = self.mappings.get(subpane_key.as_str()) {
            debug!("Got pane shortcut with key: {}", &subpane_key);
            return action.clone()
        };
        
        ShortcutAction::None
    }

    fn use_custom_shortcut(&mut self, shortcut: &str, action: ShortcutAction, context: Option<&str>) {
        let shortcut = if shortcut == "space" {
            " "
        } else { shortcut };
        let context_str = context.unwrap_or("");
        let mut old_key_strings = self.mappings.iter()
            .filter(|(key,value)| 
                **value == action && !key.contains("esc") && (context_str.is_empty() || key.contains(context_str)))
            .map(|(key, _)| key.clone()).collect::<Vec<String>>();
        
        if !old_key_strings.is_empty() {
            for key in &old_key_strings { self.mappings_to_remove.push(key.to_string()); }
        } else {
            old_key_strings = self.old_mappings.iter()
                .filter(|(key, value)| 
                    **value == action && !key.contains("esc") && (context_str.is_empty() || key.contains(context_str)))
                .map(|(key, _)| key.clone()).collect::<Vec<String>>();
        }
        
        for old_key_string in old_key_strings {
            let old_key = old_key_string.split('_').collect::<Vec<&str>>();
            let mut new_shortcut = shortcut.split('_').collect::<Vec<&str>>();

            if new_shortcut.len() == 1 {
                new_shortcut.insert(0,"none");
            }

            let mut key = old_key[0..old_key.len() - 2].to_vec();
            key.append(&mut new_shortcut);
            let new_key = key.join("_").to_string();

            if let Some(old_action) = self.mappings.get(&new_key) {
                warn!("The shortcut {} was previously assigned to {:?}, and will override it", shortcut, old_action);
                self.old_mappings.insert(new_key.clone(), old_action.clone());
                self.mappings.remove(&new_key);
                self.mappings_to_remove.retain(|key| key != &new_key);
            }
            self.mappings.insert(new_key, action.clone());
        }

    }

    fn validate_shortcut(&self, shortcut: &str) -> bool {
        let modifier_regex = Regex::new(r"^(ctrl_|alt_|super_)?([ -~]|space|right|left|up|down|enter|home|end|pageup|pagedown|tab|esc|f[1-9])$").unwrap();

        if modifier_regex.is_match(shortcut) {
            debug!("shortcut {} parsed correctly", shortcut);
            true
        }
        else {
            warn!("Could not parse shortcut {}. Should follow one of the following options: (modifier_shortcut | shortcut)", shortcut);
            false
        }

    }
    
    pub fn get_key_combo_for_operation(&self, action: ShortcutAction, context: Option<&str>) -> String {
        let key = self.mappings.iter()
            .find_map(|(key, map_action)| {
                match context {
                    Some(context) => {
                        if *map_action == action && key.contains(context) { Some(key.to_string()) } else { None }
                    }
                    None => {
                        if *map_action == action { Some(key.to_string()) } else { None }
                    }
                }
            });
        
        match key {
            None => {
                warn!("Could not find key for operation {:?}", action);
                "()".to_string()
            }
            Some(key_string) => {
                let key_parts = key_string.split('_').collect::<Vec<&str>>();
                if key_parts[key_parts.len() - 2] == "none" {
                    format!("({})", key_parts[key_parts.len() - 1])
                }
                else {
                    format!("({})", key_parts[key_parts.len()-2..].join("-"))
                }
            }
        }
        
    }

    fn clean_old_shortcuts(&mut self)  {
        for key in self.mappings_to_remove.iter() {
            self.mappings.remove(key);
        }
    }
    
}
