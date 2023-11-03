# Nowplaying

A demo app for [Turso](https://turso.tech)'s embedded replicas running on AWS Lambda

## Intro

This app tracks when users were last active, and what game they played. Users can update their info any time. Results are cached on disk in a libSQL database running in embedded replica mode.

## Running

The development is based on cargo-lambda. Install it via `cargo install cargo-lambda`.

To run a development server, run
```
cargo lambda watch
```

## API

Fetch user data: `GET https://your-lambda-url/`

Update user data: `GET https://your-lambda-url/?name=your-name&playing=game-you-are-playing`

## Examples

The examples include how much time it took to fetch a request, which shows how embedded replicas make reads considerably faster, once the data is synced to local lambda storage.

Cold start, getting user data:
```
$ httpstat -o /dev/stdout http://localhost:9000

Hi anonymous!
	(the database was freshly synced before serving the request)
User glauber,	last seen 14 minutes ago,	playing civilization, what else...
User michael,	last seen 3 minutes ago,	playing some spiderman thing nobody heard of
User pekka,	last seen 14 minutes ago,	playing tic-tac-toe
User sarna,	last seen 14 minutes ago,	playing hopscotch

   DNS Lookup   TCP Connection   Server Processing   Content Transfer
[       3ms  |           0ms  |            518ms  |             0ms  ]
             |                |                   |                  |
    namelookup:3ms            |                   |                  |
                        connect:3ms               |                  |
                                      starttransfer:522ms            |
                                                                 total:522ms  
```

Fetching warm data:
```
$ httpstat -o /dev/stdout http://localhost:9000

Hi anonymous!
	(serving cached results)
User glauber,	last seen 14 minutes ago,	playing civilization, what else...
User michael,	last seen 3 minutes ago,	playing some spiderman thing nobody heard of
User pekka,	last seen 14 minutes ago,	playing tic-tac-toe
User sarna,	last seen 14 minutes ago,	playing hopscotch

   DNS Lookup   TCP Connection   Server Processing   Content Transfer
[       0ms  |           0ms  |              2ms  |             0ms  ]
             |                |                   |                  |
    namelookup:0ms            |                   |                  |
                        connect:0ms               |                  |
                                      starttransfer:3ms              |
                                                                 total:3ms 
```

Cold start, updating data:
```
$ httpstat -o /dev/stdout http://localhost:9000?name=sarna\&playing=hopscotch

Hi sarna!
Enjoying hopscotch huh
	(the database was freshly synced before serving the request)
User glauber,	last seen 15 minutes ago,	playing civilization, what else...
User michael,	last seen 4 minutes ago,	playing some spiderman thing nobody heard of
User pekka,	last seen 15 minutes ago,	playing tic-tac-toe
User sarna,	last seen 15 minutes ago,	playing hopscotch

   DNS Lookup   TCP Connection   Server Processing   Content Transfer
[       3ms  |           0ms  |            508ms  |             0ms  ]
             |                |                   |                  |
    namelookup:3ms            |                   |                  |
                        connect:3ms               |                  |
                                      starttransfer:511ms            |
                                                                 total:511ms 
```

Updating warm data:
```
$ httpstat -o /dev/stdout http://localhost:9000?name=sarna\&playing=hopscotch

Hi sarna!
Enjoying hopscotch huh
	(serving cached results)
User glauber,	last seen 15 minutes ago,	playing civilization, what else...
User michael,	last seen 4 minutes ago,	playing some spiderman thing nobody heard of
User pekka,	last seen 15 minutes ago,	playing tic-tac-toe
User sarna,	last seen 15 minutes ago,	playing hopscotch

   DNS Lookup   TCP Connection   Server Processing   Content Transfer
[       2ms  |           0ms  |            193ms  |             0ms  ]
             |                |                   |                  |
    namelookup:2ms            |                   |                  |
                        connect:3ms               |                  |
                                      starttransfer:196ms            |
                                                                 total:196ms    
```

Fetching warm data after an update:
```
$ httpstat -o /dev/stdout http://localhost:9000

Connected to 127.0.0.1:9000
Hi anonymous!
	(serving cached results)
User glauber,	last seen 15 minutes ago,	playing civilization, what else...
User michael,	last seen 4 minutes ago,	playing some spiderman thing nobody heard of
User pekka,	last seen 15 minutes ago,	playing tic-tac-toe
User sarna,	last seen 15 minutes ago,	playing hopscotch

   DNS Lookup   TCP Connection   Server Processing   Content Transfer
[       3ms  |           0ms  |              9ms  |             0ms  ]
             |                |                   |                  |
    namelookup:3ms            |                   |                  |
                        connect:3ms               |                  |
                                      starttransfer:13ms             |
                                                                 total:13ms  
