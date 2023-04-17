extern crate cron;

use crate::cron::scheduler::{EveryTrigger, Scheduler};
use tokio;

#[tokio::main]
async fn main() {
    let mut scheduler = Scheduler::new();
    let every_trigger = EveryTrigger::new(std::time::Duration::from_secs(1), String::from("id"));
    scheduler.add_job(every_trigger);

    let receiver = scheduler.get_receiver();

    tokio::spawn(async move {
        loop {
            let msg = receiver.recv_blocking();
            println!("{:?}", msg);
        }
    });

    scheduler.start().await;
}
