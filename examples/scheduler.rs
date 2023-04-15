extern crate cron;

use crate::cron::parse::CronExpr;
use crate::cron::Cron;

fn main() {
    let ret = "* * * * * ? *".parse::<Cron>().unwrap();
    println!("{:?}", ret);
}
