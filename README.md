# naviterm 🎶
![naviterm home page.](/img/home.webp "naviterm home page.")

naviterm is a terminal user interface client for [Navidrome](https://www.navidrome.org/) written in Rust. It is an online client, meaning that it does not download music files for offline playing. It does maintain an offline database for faster operation. Navidrome uses the Subsonic API, so naviterm should be compatible with other music server programs using it, at least *in theory*. It uses the fantastic [mpv](https://mpv.io/) as backend for the music playback. For the tui, [ratatui](https://ratatui.rs/) has been used.

I was a happy user of the classic Linux music combo: ncmpcpp and mpd. But then I discovered the benefits of using a dedicated server tool to manage my music collection with Navidrome. There are many different clients for several platforms, and I used [Feishin](https://github.com/jeffvli/feishin). I liked the "home" tab UI, similar to other popular music platforms as Spotify. However, it is a heavy client and a bit slow for my taste, I longed for ncmpcpp and its quick terminal user interface. This project is my attempt to have the best of both worlds.

>⚠️ naviterm is now in a beta stage. I use it in a daily basis but bugs could occur.

## Features
* Home page with recently added/played albums, most played album/songs.
* Explore your music database with the album, playlist and artist panes.
* Add, edit, or delete playlists. Sync them with your server.
* Full MPRIS support using the amazing [zbus](https://github.com/dbus2/zbus).
* Scrobbling of played media to your server.

## Installation

## Configuration

## Usage

## Known limitations
naviterm can be a bit heavy on the CPU side. It is in part due to the nature of ratatui, as it re-draws the whole ui on each call to the draw method. Even if there are no changes in the app state, it needs to compute all the widgets based on the app information, and compare the frame with the previous one. This can be CPU expensive, so I tried to reduce the painting calls to only twice per second (to ensure smooth playing time tracking) or whenever a key is pressed. If you find it still too heavy, you can disable the painting whenever the window looses focus, using the following configuration key:
```ini
draw_while_unfocused=false
```

## Contribution
