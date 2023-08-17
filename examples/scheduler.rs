extern crate cron;

use crate::cron::scheduler::{CronTrigger, EveryTrigger, Scheduler};
use tokio::{self};

#[tokio::main]
async fn main() {
    let mut scheduler = Scheduler::new();

    let every_trigger = EveryTrigger::new(std::time::Duration::from_secs(1), String::from("id"));
    scheduler.add_job(Box::new(every_trigger));

    let cron_trigger = CronTrigger::new(String::from("id1"), "* 1/1 * * * ? *").unwrap();
    scheduler.add_job(Box::new(cron_trigger));

    for _i in 0..100 {
        let next = scheduler.get_next_firetime();
        println!("{:?}", next);
    }
}
