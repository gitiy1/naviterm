# naviterm 🎶
![naviterm home page.](/img/home.webp "naviterm home page.")

naviterm is a terminal user interface client for [Navidrome](https://www.navidrome.org/) written in Rust. It is an online client, meaning that it does not download music files for offline playing. It does maintain an offline database for faster operation. Navidrome uses the Subsonic API, so naviterm should be compatible with other music server programs using it, at least *in theory*. It uses the fantastic [mpv](https://mpv.io/) as backend for the music playback. For the tui, [ratatui](https://ratatui.rs/) has been chosen.

I was a happy user of the classic Linux music combo: ncmpcpp and mpd. But then I discovered the benefits of using a dedicated server tool to manage my music collection with Navidrome. There are many different clients for several platforms, and I used [Feishin](https://github.com/jeffvli/feishin). I liked the "home" tab UI, similar to other popular music platforms as Spotify. However, it is a heavy client and a bit slow for my taste, I longed for ncmpcpp and its quick terminal user interface. This project is my attempt to have the best of both worlds.

>⚠️ naviterm is now in a beta stage. I use it in a daily basis but bugs can occur.

## Features
* Home page with recently added/played albums, most played album/songs.
* Explore your music database with the album, playlist and artist panes.
* Add, edit, or delete playlists. Sync them with your server.
* Full MPRIS support using the amazing [zbus](https://github.com/dbus2/zbus).
* ReplayGain support
* Scrobbling of played media to your server.

## Installation

Here you can find the possible installation methods. Please note that if you install from source or use a provided binary, you will need a working version of `mpv` in your path.

### Use provided binary

Head over to the [releases page](https://gitlab.com/detoxify92/naviterm/-/releases) and download the latest one. Then, extract the contents and copy the binary to your desired path, for instance:
```sh
tar -xzf naviterm_amd64_X_Y_Z.tar.gz
sudo cp target/release/naviterm /usr/bin/
``` 

### Use the flake

**Run directly:**
```nix
nix run "gitlab:detoxify92/naviterm"
```
**Add to your NixOS config:** \
Add the flake to your inputs.
```nix
{
  inputs = {
    ...
    naviterm.url = "gitlab:detoxify92/naviterm";
  }
  ...
}
```
Then add the package to your `environment.systemPackages` or `home.packages`.
```nix
{pkgs, inputs, ...}: {
  ...
  environment.systemPackages = [
    ...
    inputs.naviterm.packages.${pkgs.system}.default
  ];
}
```
> Currently only x86_64-linux is supported.

> You don't need to install mpv seperately when using the flake.

### Build from source
First, clone this repository and switch to the desired branch: `main` for latest release or `develop` for unstable. Then run the following commands in the cloned directory:
```sh
cargo build
```
That should build the executable at `CLONED_DIR/target/debug/naviterm`. This can be useful for testing local changes. If a more permanent installation is desired, one can do:
```sh
cargo build --release
sudo cp target/release/naviterm /usr/bin/
``` 
This would generate a more lightweight executable, and it can be placed anywhere in your PATH for convenient access to the executable.

## Configuration

A configuration file is needed for the program to start. It must be at `~/.config/naviterm/config.ini`, and should have the following items:

| Parameter            | Definition                                                                                                                                                     | Default | Mandatory |
|:---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|:-------:|:---------:|
| server_address       | The address your server is running in, including the path. It should have the following format (note no trailing slash): `http(s)://name-or-ip/navidrome`   |    -    |    Yes    |
| user                 | Your user in navidrome                                                                                                                                         |    -    |    Yes    |
| password             | The password for the user                                                                                                                                      |    -    |    Yes    |
| server_auth          | The authentication method to use, to choose from plain or token                                                                                                |  token  |    No     |
| mpv_path             | The path to the mpv executable. If left empty, navidrome will try to use `mpv` from `$PATH`                                                                    |   mpv   |    No     |
| replay_gain          | The replay gain mode. The possible values are: track, album, auto                                                                                              |  track  |    No     |
| primary_accent       | The primary accent color to be used. Possible values: yellow, red, green, blue, magenta, cyan, white, gray                                                     | yellow  |    No     |
| secondary_accent     | The secondary accent color to be used. Possible values: yellow, red, green, blue, magenta, cyan, white, gray                                                   |  gray   |    No     |
| home_list_size       | The size of the lists for the home pane (recently listened, recently added, most listened albums and tracks).                                                  |   30    |    No     |
| follow_cursor_queue  | Whether the cursor will follow the currently playing track in queue                                                                                            |  true   |    No     |
| draw_while_unfocused | This flag controls whether the program will update its ui if the window loses focus. Setting to true could increase CPU usage.                                 |  false  |    No     |

The config file has to be a `ini` config file:
```ini
server_address=https://your-navidrome-instance.com/navidrome
user=joe
password=secret_pass
server_auth=token
mpv_path=/usr/bin/mpv
replay_gain=auto
primary_accent=yellow
secondary_accent=gray
home_list_size=30
follow_cursor_queue=true
draw_while_unfocused=false
```

## Usage
If your config file is correct, the program should start and begin building a local database of your music collection, which will be stored at `~/.config/naviterm/database.bin`. Please wait until is finished. After that you can start using naviterm to play some music!

### Shortcuts
This section includes lists of all shortcuts you can use in the app, heavily Vim inspired (media keys shortcuts have not been included, as they are self-explanatory).

### Global shortcuts

| Shortcut            | Description                                                |
|:--------------------|:-----------------------------------------------------------|
| `1,2,3,4,5`         | Navigate to corresponding pane                             |
| `j,k`               | Move the cursor down, up                                   |
| `g,G`               | Move the cursor to the first, last element in list         |
| `<Ctrl-d>,<Ctrl-u>` | Move the cursor down, up by 5 elements                     |
| `<Crl-c> \| q`      | Quit the program                                           |
| `<Tab>`             | Cycle through sub-panes                                    |
| `<Ctrl-h><Ctrl-l>`  | Move to the sub-pane to the left,right                     |
| `<Enter>`           | Start playing item immediately                             |
| `/`                 | Start search                                               |
| `<Enter>`           | Confirm the entered term to allow navigating the results   |
| `n \| N`            | Cycle through search results in one direction or the other |
| `<ESC>`             | Cancel and clears the search                               |
| `l`                 | Cycle through loop playing modes                           |
| `<space-bar> \| p ` | Toggle play-pause                                          |
| `o`                 | Stop playback                                              |
| `z`                 | Toggles random playback on/off                             |
| `<Right>`           | Seek 10s forward                                           |
| `<Left>`            | Seek 10s backwards                                         |
| `<Up>`              | Increase the volume                                        |
| `<Down>`            | Decreases the volume                                       |
| `u`                 | Open Update Database popup                                 |

### Home pane shortcuts
| Shortcut            | Description                                                |
|:--------------------|:-----------------------------------------------------------|
| `<F1>`              | Open Connection Testing popup                              |
| `<Ctrl-j><Ctrl-k>`  | Move to the sub-pane up,down                               |
| `i`                 | Open the Album Information popup of the selected item      |
| `a`                 | Open the Add To popup for the selected item                |

### Album pane shortcuts
| Shortcut            | Description                                                |
|:--------------------|:-----------------------------------------------------------|
| `A`                 | Open the Add To popup for whole album of selected item     |
| `a`                 | Open the Add To popup for the selected item                |
| `i`                 | Open the Album Information popup of the selected item      |
| `e`                 | Open the Genre Filter popup                                |
| `y`                 | Open the Year Filter popup                                 |
| `m`                 | Toggle the sorting mode: alphabetical, most played         |
| `r`                 | Toggle the sorting direction: ascending, descending        |

### Playlist pane shortcuts
| Shortcut | Description                                                                                            |
|:---------|:-------------------------------------------------------------------------------------------------------|
| `J,K`    | Move the selected song item in playlist down, up                                                       |
| `A`      | Open the Add To popup for whole playlist of selected item                                              |
| `a`      | Open the Add To popup for the selected item                                                            |
| `s`      | Open the Synchronize Playlist popup                                                                    |
| `d`      | Delete selected playlist (with confirmation) or the selected song from playlist (without confirmation) |

### Artist pane shortcuts
| Shortcut | Description                                            |
|:---------|:-------------------------------------------------------|
| `A`      | Open the Add To popup for whole album of selected item |
| `a`      | Open the Add To popup for the selected item            |

### Queue pane shortcuts
| Shortcut | Description                                            |
|:---------|:-------------------------------------------------------|
| `a`      | Go to album of the selected queue item in Albums pane  |
| `r`      | Go to artist of the selected queue item in Artist pane |
| `e`      | Center cursor in currently playing song                |
| `>`      | Play next song in queue                                |
| `<`      | Play previous song in queue                            |
| `c`      | Clear queue and stop playback                          |


## Known limitations
### High CPU usage
naviterm can be a bit heavy on the CPU side. It is in part due to the nature of ratatui, as it re-draws the whole ui on each call to the draw method. Even if there are no changes in the app state, it needs to compute all the widgets based on the app information, and compare the frame with the previous one. This can be CPU expensive, so I tried to reduce the painting calls to only twice per second (to ensure smooth playing time tracking) or whenever a key is pressed. If you find it still too heavy, you can disable the painting whenever the window looses focus, using the following configuration key:
```ini
draw_while_unfocused=false
```
### Big playlists
When dealing with very big playlists, it is possible that you encounter an error stating that "414 Request-URI Too Long". This is due to the fact that when syncing a playlist with the server, we send the content of the playlist in the url. If the playlist have too many items, this could be an issue. There is another Subsonic API that could be used, but it makes playlist updating substantially harder, especially when dealing with songs reordering.

## Bug reporting and contributions
Found a bug (*pretends to be shocked*)? Please open a bug report and try to describe the issue as much as possible. By default, naviterm logs some information at `/tmp/naviterm.log`. The verbosity of this log can be controlled using the environment variable `APP_DEBUG` (by default is set to INFO). Setting it to DEBUG prior to running the program can be very helpful for debugging issues:
```sh
APP_DEBUG=DEBUG naviterm
```
The `mpv` process controlled by naviterm logs at `/tmp/naviterm_mpv.log`, which could also be helpful. Please attach both logs to the issue when possible.

If you have a feature request, also open an issue. No guarantees that I will implement it, but I will always take a look to see the feasibility/impact. Contributions are very welcome.

## Compability
Please note that I developed the application with [navidrome](https://www.navidrome.org/) in mind. Although the Subsonic API is used by other server applications, it might be the case that navidrome does not work well with them. I might not be able to test/reproduce all issues of that kind, so bear that in mind when opening issues. If you are willing to help building bugfix branches and testing, that would help me a lot.

Server applications tested:
- [navidrome](https://www.navidrome.org/)
- [Nextcloud Music app](https://apps.nextcloud.com/apps/music)


## Roadmap
The app does mostly all I expect from a client (at least for my use cases). The following are some pending issues/wishlist that I will work on, time permits:
- [ ] Global fuzzy search in all songs,albums,playlists,artists
- [ ] Allow to reconfigure shortcuts
- [ ] Implement a more secure password storage
- [ ] Allow to reorganize the home pane
- [ ] React to different terminal sizes (responsive design?)
