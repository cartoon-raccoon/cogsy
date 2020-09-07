# cogsy
a curses-based, command line local Discogs client

as of now, this app is able to query the Discogs API and receive collection and wantlist information.
It still reads from a json file (filename is hardcoded because it's still in development) to display the album names to the screen in a gui.

**The final capabilities of this app are:**
- Query the Discogs database using a user's username and app token. (OAuth integration is expected, but not a priority.)
- Store the data in a database and display it in a TUI when the app is started.
- Log user's listening history, and track the user's expenditure on Discogs purchases.

**This app cannot update the user's collection or make purchases, and there are no plans to support this. This will all have to be done on Discogs itself.**

Read the notes file for more information on the app and what it can do.

If you wish you git clone and run this app, **it will not work at first.** You need the following files in your project directory:

```
discogs_collection.json
discogs_wantlist.json
discogs_token
```

This app is still in development, and thus still needs to read from these files to initialize itself. It will panic and crash without them. The collection and wantlist files must be valid json, and pulled from the Discogs API, while the token file must contain only:

`Discogs token=<your token here>`

This is the token used by Discogs to authenticate your HTTP requests, and can be obtained from Discogs Developer settings.
You can read the API [documentation](https://www.discogs.com/developers) for how to use it. It also probably goes without saying that you need a Discogs account to even use the API.

On Linux systems, you can use `curl` to make the request and redirect the output into a file (and optionally pass it through a json formatter like `jq` to make it more readable).
