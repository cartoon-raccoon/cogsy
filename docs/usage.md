# Cogsy usage and config

## Configuration

On first install, your new config.toml file will look something like this:

```toml
[user]
username = cartoon-raccoon
token = <token>
timezone = 8.0
```

You can add an additional `appearance` section with the following fields (containing their default values):

```toml
[appearance]
format = "{artist} - {title}"
sort_by = "default"
folders_width = 30
selectcol = "yellow"
messagecol = { default = "white", error = "red", success = "green", hint = "yellow" }
titlecol = "yellow"
commandcol = "white"

```

`format` is the format taken by the text displayed in the collection screen. Album attributes are enclosed in curly brackets and will be expanded into the album attributes when displayed; everything else will be displayed as is.

`sort_by` is the the predicate by which listed items will be sorted. The possible values are:

- `default` - No sorting is performed.
- `id` - Sort by Discogs ID.
- `title` - Sort by title.
- `artist` - Sort by artist name.
- `year` - Sort by release year.
- `date` - Sort by date added.

`folders_width` is the width of the folders element (left side) in the main view.

`selectcol` is the colour of the text selected.

`messagecol` contains the colours of the text that appears in the message box.

`titlecol` is the title of pop up screens such as album info.

`commandcol` is the colour of the command line.

The possible values are the eight tty colours, or:

- black
- red
- green
- yellow
- blue
- magenta
- cyan
- white

To use their bright variants, prepend the colour with "br" (e.g. "bryellow"). This is not supported by the native Linux TTY, but is supported by almost all terminal emulators.

## Usage

_What's New?_

- CSV updates! You can now update your collection from a CSV file downloaded from Discogs. When passed the `--csv` option, Cogsy will pull the data from the specified CSV file instead of from Discogs.

Cogsy can be run as a TUI text-based interface or as a command line app, depending on what arguments you pass it. You can quit the user interface by pressing `q` or issuing the `quit` command.

Cogsy as an app with a user interface, has 4 main screens:

1. **Collection**:
This is the main screen that pops up when you start Cogsy. On the left are the folders in your collection, on the right are the contents of each folder. Pressing Enter on a selected album will bring up a screen with the album's information. This is also where you access the command line, which can be activated by pressing `:`. You can return to this screen from anywhere in the app by pressing 1.
![cogsy_main](../images/screenshots/cogsy_main.png)
_Release Info Page_
![cogsy_info](../images/screenshots/cogsy_info.png)
This the release info page. It contains more detailed information on the release, such as release year, artist name, format, etc. At the bottom right there are three buttons: Ok, History and Listen.

- `Ok` takes you back to the main page.
- `History` displays the listening history for that album only.
- `Listen` logs a listening session and takes you back to the main screen.

2. **Wantlist**: Pull this up by pressing 2. This displays the contents of your wantlist. Pressing Enter will pull up a screen displaying information on the selected album, and you can press Backspace to go back to the list.
![cogsy_wantlist](../images/screenshots/cogsy_wantlist.png)
3. **Profile**: Your user profile. Pull this up by pressing 3.
![cogsy_profile](../images/screenshots/cogsy_profile.png)
4. **Listen Graph**: This displays your listening history. Pull this up by pressing 4. Each block represents one day, and the size of each block reflects how many times you listened to that album in that day.
![cogsy_listen](../images/screenshots/cogsy_listen.png)
To display history as a list of discrete entries, press `h`.
![cogsy_history](../images/screenshots/cogsy_history.png)
_The history entries are quite sparse as I haven't had time to populate them._

### The Command Line

This is Cogsy's heart. All of Cogsy's features are run from here. Vim users will find this familiar, as you activate it by pressing `:`. From here, you can run Cogsy's core commands. At any time, you can cancel a command by pressing Esc.

Cogsy has four core commands:

- `update`: Pulls collection info from Discogs and updates the entire app database. There are also the `-u` and `-t` switches for updating the username and token respectively, but they don't do anything at the moment. The `-v` switch displays verbose output when run from the CLI.
- `listen [album]`: Cogsy's core feature. Pass it an album name and it will log the album title and the current time as a listening session.
- `query [album]`: Query the local database for information on an album. Use the `-w` or `--wantlist` switch to query the wantlist, otherwise it defaults to querying the collection.
- `random`: Use this when you can't decide what to play. It also logs the selected album as a listening session, unless you pass it the `-n`/`--nolog` switch.

### Running from the CLI

Cogsy can also be run as a CLI app, by passing it one of its core commands. Running Cogsy without any arguments will bring up the user interface.

For example, `cogsy update` will cause Cogsy to update its database and exit. `cogsy query [albumname]` will cause Cogsy to display all the matches for `[albumname]` and exit.

Only accessible from the CLI is the `--csv` option for the `update` subcommand. See below for details.

Cogsy also has the `database` command, only accessible as a subcommand from the CLI. This command enables the user to administer the database. There are three options for the `database` command:

- `--reset`: This purges the database and retrieves new data from Discogs. Note that this will also remove your listening history.
- `--orphan`: This performs orphan table removal.
- `--check`: This performs the database integrity check.

Read the notes file for more information on the app, what it can do and how to use it.

## Selective and CSV Updates

Cogsy can also do selective updates of your profile, wantlist, or collection, or any subset of the three. This is done through the `-P`, `-W` and `-C` flags.

_Usage_:

```shell
# Update profile only
cogsy update -P

#or
cogsy update --profile

# Update wantlist and collection only
cogsy update -WC

#or
cogsy update --wantlist --collection
```

The `update` subcommand also has the `--csv` option. When used, Cogsy will pull data from a CSV file at the specified path and use that to update the database. This CSV has to be exported from Discogs under your user profile; Cogsy cannot understand any other format. Currently, Discogs only exports wantlist and collection data as CSV; your user profile can only be updated directly from Discogs.

The `--csv` option accepts up to two arguments, prefaced with either `wantlist=` or `collection=`. Any text after the `=` will be treated as the path of the CSV file.

_Usage_:

```shell
# Full update but update wantlist with data from discogs_wantlist.csv
cogsy update --csv wantlist=discogs_wantlist.csv

# Selectively update collection with data from discogs_collection.csv
cogsy update -C --csv collection=discogs_collection.csv

# Selectively update collection and wantlist, both from csv
cogsy update -WC --csv \
    wantlist=discogs_wantlist.csv \
    collection=discogs_collection.csv
```

## Internals

Cogsy stores all its data at (on Linux) `~/.local/share/cogsy/cogsy_data.db`.
This is a simple sqlite3 database and can be browsed with the sqlite3 browser program.

On startup, Cogsy does a database check for the required folders. If the test does not pass (the required tables are absent), it exits with a database error.

In order to keep track of user-defined folders, Cogsy uses a table called `folders` to store folder names, and uses this table to access the actual tables for each folder. Thus, it is (highly unlikely but) possible to have orphan tables - tables that exist in the database but don't have an entry in the `folders` table, and therefore are not valid user folders to Cogsy. This is detected by the database check on startup, and the relevant error message is shown. To remove orphan tables, you can run `cogsy database --orphan`. Alternatively, you can manually delete the orphan tables inside the sqlite3 browser with `drop table <folder name>;`.

**Important Note on Updating:** The Discogs API limits HTTP requests to 60 per minute, and gives up to maximum 100 albums per (paginated) request. Users with extremely large collections (>5000 albums) will see extremely long download times, and the app itself may become unusable. In addition, the pagination of the responses means that pulling all the items in a folder concurrently is not yet possible. Multithreading is only implemented on a per-folder basis, and only users with a large amount of folders will see any improvement in their update times.

However, it might be possible to work out the URL of each page in advance and pull the info concurrently that way, but the app is still subject to Discogs' rate limiting and this would just make Cogsy hit the request limit faster. Users with extremely large collections will still see a performance hit.

This algorithm may be possible to implement, and may appear in a future release.
