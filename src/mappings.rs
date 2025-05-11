use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::debug;

#[derive(Debug, Clone)]
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
}

pub struct Mappings {
    // Keys for the hashmap follow the pattern: subpane_pane_popup_flag_modifier_key
    // For instance: left_albums_none_none_ctrl_l
    // For global shortcuts, popup, pane and subpane slice are absent
    // For popup shortcuts, pane and subpane slice are absent
    // Modifier can be "none" or "ctrl"
    mappings: HashMap<String, ShortcutAction>,
}

impl Default for Mappings {
    fn default() -> Self {
        Self::new()
    }
}

impl Mappings {
    pub fn new() -> Self {
        Mappings {
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
                (String::from("Albums_none_none_none_i"),ShortcutAction::GoPopupAlbumInfo),
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
                (String::from("year_filter_range_year_none_"),ShortcutAction::PopupYearAddDigit),
                (String::from("year_filter_none_none_bkspc"),ShortcutAction::PopupYearRemoveDigit),
                (String::from("year_filter_range_year_none_tab"),ShortcutAction::PopupYearToggleFromTo),
                (String::from("year_filter_range_year_none_enter"),ShortcutAction::PopupYearAcceptFilter),
                (String::from("year_filter_none_none_enter"),ShortcutAction::PopupYearAcceptFilter),
                (String::from("year_filter_none_none_r"),ShortcutAction::PopupYearToggleRangeInput),
                (String::from("year_filter_none_none_q"),ShortcutAction::PopupClose),
                (String::from("year_filter_none_none_esc"),ShortcutAction::PopupYearClearAndClose),
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
                (String::from("none_none_ctrl_d"),ShortcutAction::MovePageDown),
                (String::from("none_none_ctrl_u"),ShortcutAction::MovePageUp),
                ])
        }
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

        let mod_used = if key_event.modifiers == KeyModifiers::CONTROL {"ctrl".to_string()} else {"none".to_string()};
        debug!("subpane:{}, pane:{}, popup:{}, flag:{} mod:{}, char:{}", subpane, pane, popup, flag, mod_used, key_pressed);
        
        let global_key = format!("{}_{}", mod_used, key_pressed);
        if let Some(action) = self.mappings.get(&global_key) {
            return action.clone()
        };

        let flag_string = String::from(flag);
        let flag_key = flag_string.clone() + "_" + global_key.as_str();
        if let Some(action) = self.mappings.get(&flag_key) {
            return action.clone()
        };
        // Check if we are introducing a single char, we don't have an entry in the hashmap for
        // every possibility
        let mut chars = flag_key.chars().collect::<Vec<char>>();
        chars.pop();
        if let Some(action) = self.mappings.get(&chars.iter().collect::<String>()) {
            return action.clone()
        };

        let popup_string = String::from(popup);
        let popup_key = popup_string.clone() + "_" + flag_key.as_str();
        if let Some(action) = self.mappings.get(popup_key.as_str()) {
            return action.clone()
        };
        // Check if we are introducing a single char, we don't have an entry in the hashmap for
        // every possibility
        let mut chars = popup_key.chars().collect::<Vec<char>>();
        chars.pop();
        if let Some(action) = self.mappings.get(&chars.iter().collect::<String>()) {
            return action.clone()
        };

        let pane_string = String::from(pane);
        let pane_key = pane_string.clone() + "_" + popup_key.as_str();
        if let Some(action) = self.mappings.get(pane_key.as_str()) {
            return action.clone()
        };

        let subpane_string = String::from(subpane);
        let subpane_key = subpane_string + "_" + pane_key.as_str();
        if let Some(action) = self.mappings.get(subpane_key.as_str()) {
            return action.clone()
        };
        
        ShortcutAction::None
    }

}
