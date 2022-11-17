# Priconne Service

Priconne service knows all about priconne, and it's source of truth.
- It handles interactions with priconne server.
- It saves address to news server and all api servers.
- It doesn't care anything about telegram.

Design choice: 
We can save all servers to the service, or currently active one. 
We choose to save all and a "default" one as this service should be source of truth about all priconne stuff.

## Where to save server address?

We need a place to save api and news server address.
Where should we save it?

`teloxide` places it in wrapper of `Client`.
`mongodb` also does, but with more layers of abstraction.

But we need more.

We need a place to save all of them: telegraph, teloxide client, database, ...
If it's not Rust I will just use some DI framework.

## Decouple priconne fetching from telegram bot?

Currently everything is trigger with respect for telegram bot.
Now I want to make priconne fetch itself a service. 
When user send a command, it *asks the service* for something.
 
## How to inject dependencies?

I want to have a API endpoint to get all current events.

## Priconne service doesn't care Telegram, but should it care Telegraph?


