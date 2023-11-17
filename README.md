# Discord Wordle Bot

## Overview

Welcome to my Discord Wordle Bot! This bot, written in Rust using [Serenity](https://crates.io/crates/serenity) and [Tokio](https://crates.io/crates/tokio). The primary feature for now is a Wordle minigame that can be triggered in your Discord server using the command `!wordle`.

## Features

- **Wordle Minigame**: Basic implementation of the classic word-guessing game. Use `!wordle` to start a new game and challenge you and your server members to guess the hidden word.

## Getting Started

1. **Prerequisites**: Ensure you have Rust installed on your machine. You can find the installation instructions [here](https://www.rust-lang.org/learn/get-started).

2. **Clone the Repository**: Clone this repository to your local machine.

```bash
git clone https://github.com/Moonloaf/discord-bot.git
```

3. **Build and Run**: Navigate to the project directory and build the bot.

```bash

cd discord-wordle-bot
cargo build --release
```

4. **Configure the Bot**: Create a .env file in the project root and set your Discord bot token.
```env
DISCORD_TOKEN=your_discord_bot_token_here
```

5. **Run the Bot**: Start the bot using the following command.

```bash
cargo run --release
```
