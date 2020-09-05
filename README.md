# cogsy
a curses-based, command line local Discogs client

as of now, this app is able to query the Discogs API and receive collection and wantlist information.
It still reads from a json file (filename is hardcoded because it's still in development) to display the album names to the screen in a gui.

**The final capabilities of this app are:**
- Query the Discogs database using a user's username and app token. (OAuth integration is expected, but not a priority.)
- Store the data in a database and display it in a TUI when the app is started.
- Log user's listening history, and track the user's expenditure on Discogs purchases.

**This app cannot update the user's collection, or make purchases. This will all have to be done on Discogs itself.**

Read the notes file for more information.
