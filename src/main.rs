use anyhow::{anyhow, Error};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;
use std::{borrow::Cow, collections::HashMap};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "yet-another-aoc-slack-bot", about = "usage")]
struct Opt {
    /// Private leaderboard ID
    #[structopt(long, env = "LEADERBOARD_ID")]
    leaderboard_id: String,

    /// API session ID
    #[structopt(long, env = "SESSION_ID")]
    session_id: String,

    /// Slack webhook
    #[structopt(long, env = "SLACK_WEBHOOK_ID")]
    slack_webhook: String,

    /// Verbose mode
    #[structopt(long)]
    verbose: bool,

    /// Refresh period
    #[structopt(long, default_value = "900")]
    refresh_period_secs: u64,

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

    fn last_update(&self) -> Option<u64> {
        self.members.iter().map(|(_, m)| m.last_star_ts).max()
    }
}

fn fetch_leaderboard(opt: &Opt) -> Result<Leaderboard, Error> {
    let client = Client::new();
    let url = format!("{}/{}.json", opt.aoc_base_url, opt.leaderboard_id);
    let resp = client
        .get(url)
        .header("Cookie", format!("session={}", opt.session_id))
        .send()?;
    Ok(resp.json()?)
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

#[derive(Default)]
struct Reporter {
    last_update: Option<u64>,
}

impl Reporter {
    fn report_leaderboard(&mut self, leaderboard: Leaderboard) {
        if let Some(new_update) = leaderboard.last_update() {
            if new_update > self.last_update.unwrap_or(u64::MIN) {
                print_leaderboard(leaderboard);
            } else {
                self.log("no new update, ignoring");
            }
            self.last_update = Some(new_update);
        } else {
            self.report_error(anyhow!("leaderboard missing update time!"));
        }
    }

    fn log<S: AsRef<str>>(&self, msg: S) {
        println!("LOG: {}", msg.as_ref());
    }

    fn report_error(&self, err: Error) {
        eprintln!("ERROR: {}", err);
    }

    fn report(&mut self, result: Result<Leaderboard, Error>) {
        match result {
            Ok(leaderboard) => self.report_leaderboard(leaderboard),
            Err(err) => self.report_error(err),
        }
    }
}

fn main() {
    let opt = Opt::from_args();
    let mut reporter = Reporter::default();
    let sleep = Duration::from_secs(opt.refresh_period_secs);
    loop {
        let leaderboard = fetch_leaderboard(&opt);
        reporter.report(leaderboard);
        std::thread::sleep(sleep);
    }
}
