mod scheduler;

extern crate cron;

use crate::cron::Cron;
use chrono::Utc;

fn main() {
    let ret = "1 1/1 * * * ? *".parse::<Cron>().unwrap();
    let now = Utc::now();
    let next = ret.next_after(now);
    println!("{:?}", next);
}
