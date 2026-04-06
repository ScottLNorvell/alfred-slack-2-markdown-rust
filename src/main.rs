use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use futures_util::StreamExt;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "alfred-slack-2-markdown")]
#[command(about = "Slack emojis to markdown", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for emojis
    Search { query: String },
    /// Update slack emojis
    Download,
    /// Configure slack token
    Config { token: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct SlackEmojiListResponse {
    ok: bool,
    emoji: HashMap<String, String>,
}

#[derive(Serialize)]
struct AlfredItem {
    title: String,
    subtitle: String,
    arg: String,
    #[serde(rename = "uuid")]
    uid: String,
    icon: AlfredIcon,
}

#[derive(Serialize)]
struct AlfredIcon {
    path: String,
}

#[derive(Serialize)]
struct AlfredOutput {
    items: Vec<AlfredItem>,
}

const EMOJI_CACHE_FILE: &str = ".emojis.json";
const EMOJI_DIR: &str = ".emojis";

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Search { query } => {
            search(query).await?;
        }
        Commands::Download => {
            download_emojis().await?;
        }
        Commands::Config { token } => {
            config(token).await?;
        }
    }

    Ok(())
}

async fn search(query: &str) -> Result<()> {
    let emojis = read_emoji_cache()?;
    let matcher = SkimMatcherV2::default();

    let mut scored_items: Vec<(i64, String, String)> = emojis
        .iter()
        .filter_map(|(name, url)| {
            let score = matcher.fuzzy_match(name, query).unwrap_or(0);
            if score > 0 || query.is_empty() {
                Some((score, name.clone(), url.clone()))
            } else {
                None
            }
        })
        .collect();

    scored_items.sort_by(|a, b| b.0.cmp(&a.0));

    let items = scored_items
        .into_iter()
        .take(20)
        .map(|(_, name, url)| {
            let resolved_url = resolve_alias(&url, &emojis).unwrap_or(url.clone());
            let icon_path = get_icon_path(&resolved_url);
            
            AlfredItem {
                title: format!(":{}:", name),
                subtitle: if resolved_url.ends_with(".gif") { "(animated)".to_string() } else { "".to_string() },
                arg: format!("![{}]({})", name, resolved_url),
                uid: resolved_url.clone(),
                icon: AlfredIcon { path: icon_path },
            }
        })
        .collect();

    let output = AlfredOutput { items };
    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}

fn resolve_alias(value: &str, emojis: &HashMap<String, String>) -> Option<String> {
    if value.starts_with("alias:") {
        let alias_name = &value[6..];
        emojis.get(alias_name).cloned()
    } else {
        None
    }
}

fn get_icon_path(url: &str) -> String {
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 2 {
        return "icon.png".to_string();
    }
    let file = parts.last().unwrap();
    let folder = parts[parts.len() - 2];
    format!("{}/{}/{}", EMOJI_DIR, folder, file)
}

fn read_emoji_cache() -> Result<HashMap<String, String>> {
    if Path::new(EMOJI_CACHE_FILE).exists() {
        let content = fs::read_to_string(EMOJI_CACHE_FILE)?;
        let emojis: HashMap<String, String> = serde_json::from_str(&content)?;
        Ok(emojis)
    } else {
        Ok(HashMap::new())
    }
}

async fn download_emojis() -> Result<()> {
    let token = env::var("SLACK_OAUTH_TOKEN")
        .context("SLACK_OAUTH_TOKEN not set. Use `smdc <token>` first.")?;

    let client = reqwest::Client::new();
    let response: SlackEmojiListResponse = client
        .get("https://slack.com/api/emoji.list")
        .bearer_auth(token)
        .send()
        .await?
        .json()
        .await?;

    if !response.ok {
        anyhow::bail!("Slack API error: emoji.list returned ok: false");
    }

    let existing = read_emoji_cache()?;
    let mut updated = existing.clone();

    fs::create_dir_all(EMOJI_DIR)?;

    let mut download_count = 0;
    for (name, url) in &response.emoji {
        if !existing.contains_key(name) && !url.starts_with("alias:") {
            let destination_dir = format!("{}/{}", EMOJI_DIR, name);
            fs::create_dir_all(&destination_dir)?;
            
            let parts: Vec<&str> = url.split('/').collect();
            let filename = parts.last().unwrap();
            let destination_path = format!("{}/{}", destination_dir, filename);

            if !Path::new(&destination_path).exists() {
                println!("Downloading :{}:...", name);
                if let Err(e) = download_file(&client, url, &destination_path).await {
                    eprintln!("Failed to download {}: {}", name, e);
                } else {
                    download_count += 1;
                }
            }
        }
        updated.insert(name.clone(), url.clone());
    }

    fs::write(EMOJI_CACHE_FILE, serde_json::to_string_pretty(&updated)?)?;
    println!("Updated! Downloaded {} new emojis.", download_count);

    Ok(())
}

async fn download_file(client: &reqwest::Client, url: &str, path: &str) -> Result<()> {
    let mut response = client.get(url).send().await?;
    let mut file = fs::File::create(path)?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let data = chunk?;
        std::io::copy(&mut &data[..], &mut file)?;
    }

    Ok(())
}

async fn config(token: &str) -> Result<()> {
    fs::write(".env", format!("SLACK_OAUTH_TOKEN={}", token))?;
    println!("CONFIGURED!");
    Ok(())
}
