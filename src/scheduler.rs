use crate::parse::CronParseError;
use crate::Cron;
use chrono::{DateTime, NaiveDateTime, Utc};
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::HashMap;
use tokio::time::Duration;

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
        self.interval.as_millis() as u64 + from
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
    pub fn new(id: String, cron: &str) -> Result<Self, CronParseError> {
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

pub struct Scheduler {
    triggers: HashMap<String, Box<dyn Trigger>>,
    queue: PriorityQueue<String, Reverse<u64>>,
}

impl Scheduler {
    pub fn new() -> Self {
        let queue = PriorityQueue::<String, Reverse<u64>>::new();
        let triggers = HashMap::<String, Box<dyn Trigger>>::new();
        Self { triggers, queue }
    }

    pub fn add_job(&mut self, job: Box<dyn Trigger>) {
        let next_firetime = job.get_next(Utc::now().timestamp_millis() as u64);

        let id = job.get_id();
        self.triggers.insert(id.clone(), job);
        self.queue.push(id, Reverse(next_firetime));
    }

    pub fn remove_job(&mut self, id: String) {
        self.queue.remove(&id);
        self.triggers.remove(&id);
    }

    pub fn get_next_firetime(&mut self) -> Option<u64> {
        let t = self.queue.pop();

        match t {
            Some(job) => {
                let job_id = job.0;
                let last_fire_time = job.1;

                let trigger_opt = self.triggers.get(&job_id);

                match trigger_opt {
                    Some(tigger) => {
                        let next_firetime = tigger.get_next(last_fire_time.0);
                        self.queue.push(job_id, Reverse(next_firetime));
                        Some(next_firetime)
                    }
                    None => None,
                }
            }
            None => None,
        }
    }
}
