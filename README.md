# Nowplaying

A demo app for [Turso](https://turso.tech)'s embedded replicas running on AWS Lambda

## Intro

This app tracks when users were last active, and what game they played. Users can update their info any time. Results are cached on disk in a libSQL database running in embedded replica mode.

## Demo

The demo is hosted here: http://nowplaying.sarna.dev

## Deploy your own Lambda

Here are the steps that should more or less work, assuming you have `aws` CLI installed and set up (and mind all the user-specific variables):

```sh
npm prune --omit=dev
zip -r nowplaying.zip .
aws lambda create-function --function-name nowplaying --zip-file fileb://nowplaying.zip --handler index.handler --runtime nodejs18.x --role $YOUR_AWS_ROLE
aws lambda update-function-configuration --function-name nowplaying --environment "Variables={LIBSQL_SYNC_URL=$YOUR_TURSO_DB_URL,LIBSQL_AUTH_TOKEN=$(turso db tokens create $YOUR_TURSO_DB_NAME)}"
# and for updating the code
aws lambda update-function-code --function-name nowplaying --zip-file fileb://nowplaying.zip
```

## API

Fetch user data: `GET https://your-lambda-url/`

Update user data: `GET https://your-lambda-url/?name=your-name&playing=game-you-are-playing`

