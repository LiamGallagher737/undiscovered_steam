use crate::steam::App;
use anyhow::{Context, Result};
use console::{style, Term};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect, Select};
use futures::future::join_all;
use rand::seq::IteratorRandom;
use rand::thread_rng;

mod steam;

const WORDLIST: &str = include_str!("wordlist.txt");
const TITLE_SPLASH: &str = r"
  _   _         _ _                               _
 | | | |_ _  __| (_)___ __ _____ _____ _ _ ___ __| |
 | |_| | ' \/ _` | (_-</ _/ _ \ V / -_) '_/ -_) _` |
  \___/|_||_\__,_|_/__/\__\___/\_/\___|_| \___\__,_|
 / __| |_ ___ __ _ _ __
 \__ \  _/ -_) _` | '  \
 |___/\__\___\__,_|_|_|_|
";

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", style(&TITLE_SPLASH[1..]).cyan().bold());

    let term = Term::stdout();
    term.set_title("Undiscovered Steam");

    let theme = ColorfulTheme::default();

    let max_price: u16 = Input::with_theme(&theme)
        .with_prompt("Max Price")
        .with_initial_text("0")
        .interact()?;

    let max_reviews: usize = Input::with_theme(&theme)
        .with_prompt("Max Reviews")
        .with_initial_text("20")
        .interact()?;

    let results_count: usize = Input::with_theme(&theme)
        .with_prompt("Results")
        .with_initial_text("25")
        .interact()?;

    let required_platforms = MultiSelect::with_theme(&theme)
        .with_prompt("Supported Platforms")
        .items(&["Windows", "Macos", "Linux"])
        .defaults(&[true, false, false])
        .interact()?;

    let client = reqwest::Client::new();
    let mut games = Vec::with_capacity(results_count);
    while games.len() < results_count {
        term.write_line(&format!("[{}/{results_count}]", games.len()))?;
        get_filtered_games(
            &client,
            &mut games,
            max_price,
            max_reviews,
            &required_platforms,
        )
        .await?;
        term.clear_line()?;
    }

    let titles: Vec<String> = games
        .iter()
        .map(|game| {
            format!(
                "{} • ${} • {} reviews",
                style(game.data.name.clone()).bold(),
                game.data.price_overview.final_price as f32 / 100.0,
                game.reviews.num_reviews
            )
        })
        .collect();

    loop {
        let selection = Select::with_theme(&theme)
            .with_prompt("Select a game")
            .default(0)
            .items(&titles[..])
            .interact_opt()?;

        let Some(n) = selection else {
            break;
        };

        open::that(format!(
            "https://store.steampowered.com/app/{}",
            games[n].data.steam_appid
        ))?;
    }

    Ok(())
}

async fn get_filtered_games(
    client: &reqwest::Client,
    list: &mut Vec<App>,
    max_price: u16,
    max_reviews: usize,
    required_platforms: &[usize],
) -> Result<()> {
    let mut rng = thread_rng();
    let term = WORDLIST
        .lines()
        .choose(&mut rng)
        .context("Failed to pick random word")?;
    println!("{term}");
    let results = steam::search(client, max_price, term.into()).await?;

    let mut requests = Vec::with_capacity(results.len());
    for result in results {
        let segments: Vec<&str> = result.logo.split('/').collect();
        requests.push(steam::app(client, segments[5].to_string()));
    }

    let responses = join_all(requests).await;

    'response_loop: for response in responses {
        let Ok(game) = response else {
            continue;
        };

        if game.reviews.total_reviews > max_reviews {
            continue;
        }

        for platform in required_platforms {
            let supports = match *platform {
                0 => game.data.platforms.windows,
                1 => game.data.platforms.mac,
                2 => game.data.platforms.linux,
                _ => unreachable!(),
            };

            if !supports {
                continue 'response_loop;
            }
        }

        list.push(game);
    }

    Ok(())
}
