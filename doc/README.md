# Structure

- Resources
  - Article
    - Information
    - News
  - Cartoon
  - Event
  - Glossary
- Client
- Bot
- Server
  - Receive bot webhooks
  - Provides REST API
- Database

1. Server receive update
2.1 Routing to bot for handling Telegram webhook
2.1.1 Resource-specific requests -> get resource and return it
2.1.2 Check requests -> run check logic and return result
2.1.3 Group commands
2.1.4 Channel message pinned -> unpin it?
2.2 Respond to REST request

## Bot

Bot can run in two modes:

- Webhook mode, when webhook url is provided
- Polling mode, otherwise

### Priconne Commands

#### Everyone

- `/help`: Show this help message

For commands below, bot will get resource and return it to user.

> Q: Who is responsible for formatting the output?

- `/cartoon`: Get cartoon by id
- `/news`: Get news by id
- `/information`: Get information by id
- `/events`: Current & upcoming events

#### Admins

- `/startSchedule`: Start schedule
- `/schedule`: Show schedule
- `/stopSchedule`: Stop the schedule

For commands below, bot should give check result.

- `/checkCartoon`: Check for latest cartoon
- `/checkNews`: Check for latest news
- `/checkInformation`: Check for latest information

### Group admin commands

- `/unpinChannel [on/off]`: Unpin messages from the channel

## Priconne Service

Responsible for checking new resources regularly, and check if APIs are reachable.

## Client

Client is responsible for connecting with database, priconne server, telegram, etc.
We need a specific client because options like proxy can be applied.

### All structs use the same client, is that ok?

From [`reqwest::Client`](https://docs.rs/reqwest/latest/reqwest/struct.Client.html) documentation:

> The Client holds a connection pool internally, so it is advised that you **create one and reuse it**.

> You do not have to wrap the Client in an Rc or Arc to reuse it, because it already uses an Arc internally.

## Server

Server is responsible for receiving webhooks and providing REST API, built on top of [`axum`](https://github.com/tokio-rs/axum).

## Scheduling

Currently we use a custom scheduler, which supports merging multiple crontab entries without duplicating. It's not so flexible as schedule can't be changed after it's started.
However, after checking [mvniekerk/tokio-cron-scheduler](https://github.com/mvniekerk/tokio-cron-scheduler), I believe it's not time to use such a complicated scheduler, so 
at least for now, leave it as it is.

## Future Works

### Sychronize between JP servers

We hope we can synchronize cartoon & post data between JP servers.
