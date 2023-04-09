# Princess Connect Telegram Bot

Push latest 超異域公主連結！Re:Dive informations to Telegram channel.

Replaces [pcrtwinfobot](https://github.com/KiruyaMomochi/pcrtwinfobot).

There is still a lot of work to do, and current program structure is far from perfect.

## Design Consideration

### Quality over quantity

I prefer the quality of software hence I'm not going to rush.
I will try to make the code as clean as possible, though it may take a long time and even never be finished.

### No battle and extra features

I'm not a fun of battle, and not interested in both clan battle and arena, all I like is story of the game.
Entertainment features, like Chieru-lang, gacha simulator, etc., are not my priority.
I may not implement any features that are not related to the storylines.

There are already many bots that provide these features, they use easier languages like Python and JavaScript, with good plugin system. Why not use them?

### Never give a character name to the bot

This bot is a tool, and I want to keep it as a tool.
Putting a character name gives it a personality, which I don't want.

## Environment Setup

As a workaround for [rust#103387](https://github.com/rust-lang/rust/issues/103387), I'm using trait alias.
Therefore, nightly toolchain is required. You can automate this process by using Nix and direnv.

## Why Rust?

![PriConne Rust Meme](https://user-images.githubusercontent.com/65301509/148802177-07d6a5d4-ef65-449b-9655-862f6622700a.png)
