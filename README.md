# cogsy

### a curses-based, command line local Discogs client written in Rust

![cogsy logo](images/cogsy_logo.png)

![version](https://img.shields.io/crates/v/cogsy) ![downloads](https://img.shields.io/crates/d/cogsy)

## About

Cogsy is a curses-based command line app for tracking your Discogs collection. It queries the Discogs API and allows you to view that information offline, without the need for a browser. It also adds additional features such as tracking your listening history, and displaying it nicely for you.

For the uninformed, [Discogs](https://www.discogs.com) is a website/marketplace where music enthusiasts can collect and sell physical music media such as vinyl records and CDs.

**Cogsy has entered 0.2.0! To see what's new, jump to the Usage section.**

**The final capabilities of this app are:**

- Query the Discogs database using a user's username and app token. (OAuth integration is expected, but not a priority.)
- Store the data in a database and display it in a TUI when the app is started.
- Log user's listening history, and display it when needed.

Cogsy's target audience is admittedly, not very large. It is targeted at people who use their computer's terminal on a daily basis, _and_ are also music enthusiasts with a vinyl/CD collection. That being said, if you don't fit into the above demographic, but this app still appeals to you, do give it a try!

**Note: This app cannot make changes to the user's collection or make purchases, and there are no plans to support this. This will all have to be done on Discogs itself.**

## Requirements

- `cargo` v1.46.0 or later (The official Rust build tool and package manager)
- `rustc` v1.46.0 or later (The official Rust compiler)
- `gcc` v10 or later (For linking, `rustc` does not do linking by itself)
- A Discogs account (obviously)

  - Note: your Discogs collection has to have a folder named "All" in it. Not having it will break a lot of Cogsy's functionality.

_The following C shared libraries have to be preinstalled:_

- `libsqlite3` or its equivalent for your distro.
- `openssl` (Should already exist)
- `ncurses`

Most installation errors stem from the linker not being able to find the corresponding `.so` file on your system. Look for the missing package and install its `dev` version. This should fix most errors.

You can install everything using the `rustup` toolchain manager, instructions available from the official Rust [install page](https://www.rust-lang.org/tools/install).

For Linux users, it is recommended you install `rustup` from your distro's repos. This especially applies to Arch Linux users, as Rust AUR packages are compiled using the Arch-packaged version of Rust. `makepkg` won't detect Cargo installed via the official channel.

For Arch Linux users:

```text
pacman -S rustup
rustup default stable/nightly
```

Read the Arch Wiki article on Rust for more information.

## Installation

#### macOS, Windows, and Linux

Cogsy is currently broken on Windows 10, due to an ncurses dependency. This bug is being worked on right now. Windows users could get around this problem by running it on Cygwin or WSL, but I don't have a way to test this yet.

_The stable toolchain will compile Cogsy, but in the past some dependencies have used nightly-only features and resulting in errors when compiling on stable. Thus, I recommend you use the nightly toolchain by default._

To switch to the nightly toolchain, run:

`rustup default nightly`

Cogsy can be installed from crates.io, the official Rust package registry:

`cargo install cogsy`

This command is also used to update Cogsy when a new version is released. Your data should remain intact.

For Arch Linux users, Cogsy is now available on the AUR.

```text
git clone https://aur.archlinux.org/cogsy.git
cd cogsy
makepkg -si
```

Or use your preferred AUR helper:

`paru -S cogsy`

To build from source (please don't do this, use cargo instead):

```text
git clone https://github.com/cartoon-raccoon/cogsy
cd cogsy
make install
```

If anyone is willing to package the app for their own distro, please let me know and then go ahead.

This app has been tested on Arch Linux, Void Linux, Pop!-OS and Fedora. Testing for MacOS and Windows is underway.

_Note: Cogsy is still very much in development and is still considered unstable. It will only enter 1.0.0 when it works on all three target OSes._

## Setup

The app requires some setup: To access the Discogs API, it requires a user token. To obtain this token, go to your Discogs account settings > Developers > Generate new token. Copy the generated string to your clipboard.

On first time startup, the app will query you for your user credentials: Your username, your token and your timezone. To paste the token into the terminal, you may need to use Ctrl-Shift-V instead of Ctrl-V. Once this information has been entered, the app will pull your information from Discogs and start up.

After first time startup, a config.toml file will be created and can be found at:

**Linux:**
`/home/username/.config/cogsy/config.toml`

**MacOS:**
`/Users/username/Library/Application Support/rs.cartoon-raccoon.cogsy/config.toml`

**Windows:**
`C:\Users\username\AppData\Roaming\cartoon-raccoon\cogsy\config.toml`

The config file contains the information you entered during first time startup.

Note: The Discogs API supports OAuth2, and OAuth2 integration for the app is being considered, but it's not likely to happen, and I felt like it doesn't fit the spirit of a small command-line app like this to use such a framework. You will have to use your user token for the foreseeable future.

## Usage

With v0.2.0, Cogsy has a whole lot of new features and fixes, with even more coming up. Since the list of features has gotten so large, it now has its own manual page.

See the documentation [here](docs/usage.md).

## Contributing

Please feel free to fork this repo and send a pull request. This whole project is currently a single person job, and I'll be happy to accept any contributions.

## Issues and Bugs

- **When running `update`, the app freezes up**
  - This is normal behaviour. Cogsy uses a blocking API to query Discogs, which means the entire app is put on pause while the update process is running. Async behaviour is not planned.
- **Large collections may slow the app down fairly noticeably**
  - The computation for displaying the data to the screen is done lazily, i.e. everything is loaded from the database and processed only when the command is invoked. Nothing is pre-computed and cached beforehand. Working on implementing this now.

If there are any other bugs, please raise an issue and I will do my best to respond and fix it.

## Future Additions

_None of these additions are guaranteed._

- Adding a `price` command, allowing the user to set the price they paid for the album, and also a screen to display the increasing amount of money they spend on their music collection as a sparkview graph. The code to parse the command is already written, all that's left is to implement it.
- An option to read user collection data from a CSV file (Discogs supports downloading collection data as CSV). This would prove to be useful for users with larger collections.
- A popup in Listening history that shows the history for the album only.
- OAuth2 integration (unlikely).

## Acknowledgements

[gyscos](https://github.com/gyscos) for the Cursive library that the user interface is built on. Thank you for this amazing crate, and for your assistance on Reddit.

Cogsy was heavily inspired by [dijo](https://github.com/NerdyPepper/dijo) by NerdyPepper. It was this project that inspired me to learn Rust in the first place, and this is the first major project I've built, not just in Rust, but ever. This entire project owes its existence to him. Thank you sir.
