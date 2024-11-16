use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug,PartialEq)]
pub enum Operation {
    GetPlaylistList(bool),
    GetPlaylist(String),
    GetAlbumListAlphabetical(bool, usize),
    GetAlbumListRecent(),
    GetAlbumListRecentlyAdded(),
    GetAlbumListMostListened(usize),
    GetAlbum(String),
    GetGenreList,
}

#[derive(Debug)]
pub struct AsyncOperation {
    operation_id: Operation,
    operation_url: String,
    result: String,
    started: bool,
    finished: bool,
    processed: bool,
    thread_rx_handle: UnboundedReceiver<String>,
    thread_tx_handle: UnboundedSender<String>,
}

impl AsyncOperation {
    pub fn new(
        operation_id: Operation,
        operation_url: String,
        thread_rx_handle: UnboundedReceiver<String>,
        thread_tx_handle: UnboundedSender<String>,
    ) -> Self {
        AsyncOperation {
            operation_url,
            result: String::new(),
            thread_rx_handle,
            thread_tx_handle,
            started: false,
            finished: false,
            processed: false,
            operation_id,
        }
    }

    pub fn operation_id(&self) -> &Operation {
        &self.operation_id
    }

    pub fn operation_url(&self) -> &str {
        &self.operation_url
    }

    pub fn thread_rx_handle(&mut self) -> &mut UnboundedReceiver<String> {
        &mut self.thread_rx_handle
    }

    pub fn thread_tx_handle(&mut self) -> &mut UnboundedSender<String> {
        &mut self.thread_tx_handle
    }

    pub fn started(&self) -> bool {
        self.started
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn processed(&self) -> bool {
        self.processed
    }

    pub fn set_started(&mut self, started: bool) {
        self.started = started;
    }

    pub fn set_finished(&mut self, finished: bool) {
        self.finished = finished;
    }

    pub fn set_processed(&mut self, processed: bool) { self.processed = processed; }
    pub fn set_result(&mut self, result: String) {
        self.result = result;
    }

    pub fn result(&self) -> &str {
        &self.result
    }
}
