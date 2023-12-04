use reqwest::blocking::Client;
use serde::Deserialize;
use std::{borrow::Cow, collections::HashMap};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "yet-another-aoc-slack-bot", about = "usage")]
struct Opt {
    /// Private leaderboard ID
    #[structopt(long)]
    leaderboard_id: String,

    /// API session ID
    #[structopt(long)]
    session_id: String,

    /// Slack webhook
    #[structopt(long)]
    slack_webhook: String,

    /// Verbose mode
    #[structopt(long)]
    verbose: bool,

    /// AoC leaderboard base URL
    #[structopt(
        long,
        default_value = "https://adventofcode.com/2023/leaderboard/private/view"
    )]
    aoc_base_url: String,
}

#[derive(Debug, Deserialize)]
struct Member {
    last_star_ts: u64,
    local_score: i64,
    id: u64,
    name: Option<String>,
    stars: i64,
}

impl Member {
    fn name(&self) -> Cow<String> {
        match &self.name {
            Some(name) => Cow::Borrowed(name),
            None => Cow::Owned(format!("<anonymous user {}>", self.id)),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Leaderboard {
    members: HashMap<String, Member>,
    event: String,
}

impl Leaderboard {
    fn sorted_members(self) -> Vec<Member> {
        let mut members: Vec<Member> = self.members.into_values().collect();
        members.sort_by_key(|m| -m.local_score);
        members
    }
}

fn fetch_leaderboard(opt: &Opt) -> Result<Leaderboard, impl std::error::Error> {
    let client = Client::new();
    let url = format!("{}/{}.json", opt.aoc_base_url, opt.leaderboard_id);
    let resp = client
        .get(url)
        .header("Cookie", format!("session={}", opt.session_id))
        .send()?;
    resp.json()
}

fn print_leaderboard(leaderboard: Leaderboard) {
    println!("🎄 Advent of Code {} 🎄", leaderboard.event);
    println!("{:>5} {:<30} {:>7} {:>7}", "Pos", "User", "Score", "Stars");
    let members = leaderboard.sorted_members();
    for (place, member) in members.into_iter().enumerate() {
        println!(
            "{:>5} {:<30} {:>7} {:>7}",
            place + 1,
            member.name(),
            member.local_score,
            member.stars
        );
    }
}

fn main() {
    let opt = Opt::from_args();
    let leaderboard = fetch_leaderboard(&opt).unwrap();
    print_leaderboard(leaderboard);
}
