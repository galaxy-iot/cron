extern crate cron;

use std::{sync::Arc, time};

use crate::cron::scheduler::{CronTrigger, EveryTrigger, Scheduler};
use chrono::Utc;
use tokio::{self};

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        let mut scheduler = Scheduler::new();

        let every_trigger =
            EveryTrigger::new(std::time::Duration::from_secs(1), String::from("id"));
        scheduler.add_job(Arc::new(every_trigger));

        let cron_trigger = CronTrigger::new(String::from("id1"), "* 1/1 * * * ? *").unwrap();
        scheduler.add_job(Arc::new(cron_trigger));

        let (tx, rx) = async_channel::bounded::<u64>(1024);

        tokio::spawn(async move {
            loop {
                let msg = rx.recv().await;
                println!("{:?}", msg);
            }
        });

        loop {
            let now = Utc::now().timestamp_millis() as u64;
            let next_firetime = scheduler.get_next_firetime().unwrap();

            if now >= next_firetime {
                let _ = tx.send(next_firetime).await;
            } else {
                tokio::time::sleep(time::Duration::from_millis(next_firetime - now)).await;
                let _ = tx.send(next_firetime).await;
            }
        }
    });

    loop {
        tokio::time::sleep(time::Duration::from_secs(14)).await;
    }
}
