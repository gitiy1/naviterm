use std::io::Write;
use std::os::unix::net::UnixStream;

pub fn quit() {
    let mut stream = UnixStream::connect("/tmp/naviterm_mpv").expect("Cannot create ipc stream");
    stream.write_all(b"{\"command\":[\"quit\"]}\n").expect("ipc: Error while exiting ipc connection");
}

pub fn load_file(file_url: &str) {
    let mut stream = UnixStream::connect("/tmp/naviterm_mpv").expect("Cannot create ipc stream");
    let msg = r#"{"command":["loadfile", ""#.to_owned() + file_url + r#""]}"# + "\n";
    stream.write_all(msg.as_bytes()).expect("ipc: Error while loading file");
}