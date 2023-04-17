mod scheduler;

extern crate cron;

use crate::cron::parse::CronExpr;
use crate::cron::Cron;
use chrono::Utc;

fn main() {
    let ret = "* * * * * ? *".parse::<Cron>().unwrap();
    let now = Utc::now();
    let next = ret.next_after(now);
    println!("{:?}", next);
}
