use crate::parse::CronParseError;
use crate::Cron;
use async_channel::{Receiver, Sender};
use chrono::{DateTime, NaiveDateTime, Utc};
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

pub trait Trigger {
    fn get_next(&self, from: u64) -> u64;
    fn get_id(&self) -> String;
}

pub struct EveryTrigger {
    interval: Duration,
    id: String,
}

impl EveryTrigger {
    pub fn new(interval: Duration, id: String) -> Self {
        Self { interval, id }
    }
}

impl Trigger for EveryTrigger {
    fn get_next(&self, from: u64) -> u64 {
        self.interval.as_secs() + from
    }

    fn get_id(&self) -> String {
        return self.id.clone();
    }
}

pub struct CronTrigger {
    id: String,
    cron: Cron,
}

impl CronTrigger {
    fn new(id: String, cron: &str) -> Result<Self, CronParseError> {
        let opt = cron.parse::<Cron>();

        match opt {
            Ok(expr) => Ok(CronTrigger { id: id, cron: expr }),
            Err(e) => Err(e),
        }
    }
}

impl Trigger for CronTrigger {
    fn get_next(&self, from: u64) -> u64 {
        let naive = NaiveDateTime::from_timestamp_millis(from as i64).unwrap();
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let next = self.cron.next_after(datetime).unwrap();
        next.timestamp_millis() as u64
    }

    fn get_id(&self) -> String {
        return self.id.clone();
    }
}

pub struct Scheduler<T: Trigger> {
    triggers: HashMap<String, T>,
    queue: PriorityQueue<String, u64>,
    sender: Sender<u64>,
    receiver: Receiver<u64>,
    stop: bool,
}

impl<T: Trigger> Scheduler<T> {
    pub fn new() -> Self {
        let queue = PriorityQueue::<String, u64>::new();
        let triggers = HashMap::<String, T>::new();
        let (sender, receiver) = async_channel::bounded::<u64>(128);
        Self {
            triggers,
            queue,
            sender,
            receiver,
            stop: false,
        }
    }

    pub fn add_job(&mut self, job: T) {
        let next_firetime = job.get_next(Utc::now().timestamp() as u64);
        let id = job.get_id();
        self.triggers.insert(id.clone(), job);
        self.queue.push(id, next_firetime);
    }

    pub fn remove_job(&mut self, id: String) {
        self.queue.remove(&id);
        self.triggers.remove(&id);
    }

    pub fn get_receiver(&self) -> Receiver<u64> {
        return self.receiver.clone();
    }

    pub async fn start(&mut self) {
        loop {
            if self.stop {
                return;
            }

            let t = self.queue.pop();

            let sleep_interval = Duration::from_millis(500);

            match t {
                Some(job) => {
                    let job_id = job.0;
                    let last_fire_time = job.1;
                    _ = self.sender.send(last_fire_time).await;

                    let trigger_opt = self.triggers.get(&job_id);

                    match trigger_opt {
                        Some(tigger) => {
                            let next_firetime = tigger.get_next(last_fire_time);
                            self.queue.push(job_id, next_firetime);
                        }
                        None => {}
                    }
                }
                None => {}
            }

            sleep(sleep_interval).await;
        }
    }

    pub async fn stop(&mut self) {
        self.stop = true
    }
}
