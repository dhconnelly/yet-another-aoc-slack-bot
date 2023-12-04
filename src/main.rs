use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
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

#[derive(Debug, Deserialize)]
struct Leaderboard {
    members: HashMap<String, Member>,
    event: String,
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

#[derive(Debug)]
struct MemberScore {
    id: u64,
    name: Option<String>,
    score: i64,
    stars: i64,
}

impl From<Member> for MemberScore {
    fn from(member: Member) -> Self {
        MemberScore {
            id: member.id,
            name: member.name,
            score: member.local_score,
            stars: member.stars,
        }
    }
}

#[derive(Debug)]
struct EventScores {
    event: String,
    scores: Vec<MemberScore>,
}

impl EventScores {
    fn sorted(mut self) -> Self {
        self.scores.sort_by_key(|score| -score.score);
        self
    }
}

impl From<Leaderboard> for EventScores {
    fn from(leaderboard: Leaderboard) -> Self {
        let event = leaderboard.event;
        let scores = leaderboard
            .members
            .into_values()
            .map(MemberScore::from)
            .collect();
        Self { event, scores }
    }
}

fn print_scores(scores: EventScores) {
    println!("🎄 Advent of Code {} 🎄", scores.event);
    println!("{:>5} {:<30} {:>7} {:>7}", "Pos", "User", "Score", "Stars");
    for (place, member) in scores.scores.into_iter().enumerate() {
        let name = member.name.unwrap_or_else(|| member.id.to_string());
        println!(
            "{:>5} {:<30} {:>7} {:>7}",
            place + 1,
            name,
            member.score,
            member.stars
        );
    }
}

fn main() {
    let opt = Opt::from_args();
    let leaderboard = fetch_leaderboard(&opt).unwrap();
    let scores = EventScores::from(leaderboard).sorted();
    print_scores(scores);
}
