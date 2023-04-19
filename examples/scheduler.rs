extern crate cron;

use crate::cron::scheduler::{CronTrigger, EveryTrigger, Scheduler};
use chrono::Utc;
use tokio;

#[tokio::main]
async fn main() {
    let mut scheduler = Scheduler::new();
    let every_trigger = EveryTrigger::new(std::time::Duration::from_secs(1), String::from("id"));
    scheduler.add_job(Box::new(every_trigger));

    let cron_trigger = CronTrigger::new(String::from("id1"), "* * * * * ? *").unwrap();
    scheduler.add_job(Box::new(cron_trigger));

    let receiver = scheduler.get_receiver();

    tokio::spawn(async move {
        loop {
            let msg: Result<u64, async_channel::RecvError> = receiver.recv_blocking();
            println!("{:?},{:?}", msg, Utc::now());
        }
    });

    scheduler.start().await;
}
