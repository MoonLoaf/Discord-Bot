use std::env;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self};

use tokio::sync::Mutex;

use serenity::prelude::*;
use serenity::model::gateway::Ready;
use serenity::model::channel::Message;
use serenity::async_trait;

use rand::seq::SliceRandom;

use serde_json::Value;


#[derive(Debug, Clone, PartialEq, Eq)]
enum GameState {
    Idle,
    InProgress { secret_word: String, attempts: u8 },
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Idle
    }
}

struct Handler {
    game_state: Mutex<GameState>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Lock the game state for reading
        let mut game_state = self.game_state.lock().await;

        // Check if the message content is "!wordle" and no game is in progress
        if msg.content == "!wordle" && *game_state == GameState::Idle {
            // Call the function to get a random word
            match read_and_select_word("assets/wordle.json") {
                Ok(secret_word) => {
                    println!("Randomly selected word: {}", secret_word);

                    // Set the game state to "InProgress" with the secret word and 5 attempts
                    *game_state = GameState::InProgress {
                        secret_word: secret_word.clone(),
                        attempts: 5,
                    };

                    // Send a message to start the game
                    let _ = msg
                        .channel_id
                        .send_message(&ctx.http, |m| {
                            m.content("Wordle game started! You have 5 attempts. Make a guess using `!guess <word>`.")
                        })
                        .await;
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        } else if msg.content.starts_with("!guess") {
            // Check if the message content starts with "!guess"
            match *game_state {
                // If no game is in progress, inform the user
                GameState::Idle => {
                    let _ = msg
                        .channel_id
                        .send_message(&ctx.http, |m| {
                            m.content("No Wordle game is currently in progress. Start a game with `!wordle`.")
                        })
                        .await;
                }
                // If a game is in progress, process the guess
                GameState::InProgress {
                    ref secret_word,
                    ref mut attempts,
                } => {
                    // Extract the guessed word from the message
                    let guessed_word = msg.content["!guess".len()..].trim();

                    if !guessed_word.chars().all(|c| c.is_alphabetic()) {
                        // Handle the case where the guessed word contains non-letter characters
                        let _ = msg
                            .channel_id
                            .send_message(&ctx.http, |m| {
                                m.content(":skull: Invalid guess. Please provide a word containing only letters. :skull:")
                            })
                            .await;
                        }
                    else if guessed_word.len() != 5 {
                        // Handle the case where the guessed word is not 5 characters long
                        let _ = msg
                            .channel_id
                            .send_message(&ctx.http, |m| {
                                m.content(":angry: Invalid guess length. Please provide a 5-letter word. :angry:")
                            })
                            .await;
                    } else if !check_word_in_file("assets/wordle.json", guessed_word) {
                        // Handle the case where the guessed word is not in the wordle.json file
                        let _ = msg
                            .channel_id
                            .send_message(&ctx.http, |m| {
                                m.content(":man_facepalming: Invalid guess. The word is not in the wordle dictionary. :man_facepalming:")
                            })
                            .await;
                    }
                    // Check if the guessed word is correct
                    else if guessed_word.eq_ignore_ascii_case(&secret_word) {
                        let _ = msg
                            .channel_id
                            .send_message(&ctx.http, |m| {
                                m.content("Congratulations! You guessed the word correctly! \n:green_square: :green_square: :green_square: :green_square: :green_square:")
                            })
                            .await;
                        // Reset the game state to "Idle"
                        *game_state = GameState::Idle;
                    } else {
                        // Decrement the attempts and check for game over
                        *attempts -= 1;
                        if *attempts == 0 {
                            let _ = msg
                                .channel_id
                                .send_message(&ctx.http, |m| {
                                    m.content(format!(
                                        "Out of attempts! The correct word was: {}",
                                        secret_word
                                    ))
                                })
                                .await;
                            // Reset the game state to "Idle"
                            *game_state = GameState::Idle;
                        } else {
                            let feedback_message = check_guess(&secret_word, guessed_word);
                            let _ = msg
                                .channel_id
                                .send_message(&ctx.http, |m| {
                                    m.content(format!(
                                        "Incorrect guess. Attempts remaining: {}\n{}",
                                        attempts, feedback_message
                                    ))
                                })
                                .await;
                        }
                    }
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Initialize the game state mutex
    let game_state = Mutex::new(GameState::default());

    // Create the handler with the game state
    let handler = Handler { game_state };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

fn check_guess(secret_word: &str, guessed_word: &str) -> String {
    let mut result = String::new();

    // Convert secret word and guessed word to lowercase for case-insensitive comparison
    let secret_word_lower: Vec<char> = secret_word.chars().collect();
    let guessed_word_lower: Vec<char> = guessed_word.chars().collect();

    // Create a HashSet of the correct letters in the correct positions
    let correct_positions: HashSet<usize> = secret_word_lower
        .iter()
        .enumerate()
        .filter(|&(i, &c)| guessed_word_lower.get(i) == Some(&c))
        .map(|(i, _)| i)
        .collect();

    for (i, &letter) in guessed_word_lower.iter().enumerate() {
        if let Some(_pos) = correct_positions.get(&i) {
            // Green emoji for correct letter in correct position
            result.push_str(":green_square: ");
        } else if secret_word_lower.contains(&letter) {
            // Yellow emoji for correct letter in incorrect position
            result.push_str(":yellow_square: ");
        } else {
            // Red emoji for incorrect letter
            result.push_str(":red_square: ");
        }
    }

    result
}

fn read_and_select_word(file_path: &str) -> io::Result<String> {
    // Read the contents of the file
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    // Parse the JSON content
    let json: Result<Value, _> = serde_json::from_reader(reader);

    match json {
        Ok(json_value) => {
            // Check if the parsed value is an array
            if let Some(array) = json_value.as_array() {
                // Collect words from each array element, filtering out non-alphabetic characters
                let mut words: Vec<String> = array
                    .iter()
                    .flat_map(|value| {
                        if let Some(string) = value.as_str() {
                            Some(
                                string
                                    .chars()
                                    .filter(|c| c.is_alphabetic())
                                    .collect::<String>(),
                            )
                        } else {
                            None
                        }
                    })
                    .collect();

                // Shuffle the words randomly
                let mut rng = rand::thread_rng();
                words.shuffle(&mut rng);

                // Select and return a random word
                if let Some(random_word) = words.pop() {
                    Ok(random_word)
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "No words found in the file"))
                }
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Invalid JSON format"))
            }
        }
        Err(err) => {
            eprintln!("Error parsing JSON: {:?}", err);
            Err(io::Error::new(io::ErrorKind::Other, "Error parsing JSON"))
        }
    }
}

fn check_word_in_file(file_path: &str, word: &str) -> bool {
    // Read the contents of the file
    if let Ok(file) = File::open(file_path) {
        let reader = io::BufReader::new(file);

        // Parse the JSON content
        let json: Result<Value, _> = serde_json::from_reader(reader);

        match json {
            Ok(json_value) => {
                // Check if the word exists in the JSON array
                if let Some(array) = json_value.as_array() {
                    return array.iter().any(|value| {
                        if let Some(string) = value.as_str() {
                            string.trim_matches('"').eq_ignore_ascii_case(word.trim_matches('"').trim())
                        } else {
                            false
                        }
                    });
                }
            }
            Err(err) => eprintln!("Error parsing JSON: {:?}", err),
        }
    }
    false
}