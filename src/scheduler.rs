use async_channel::{Receiver, Sender};
use chrono::Utc;
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::time::Duration;

pub trait Trigger {
    fn get_next(&self, from: u64) -> u64;
    fn get_id(&self) -> String;
}

pub struct EveryTrigger {
    interval: Duration,
    id: String,
}

impl EveryTrigger {
    fn new(interval: Duration, id: String) -> Self {
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

pub struct Scheduler<T: Trigger> {
    triggers: HashMap<String, T>,
    queue: PriorityQueue<String, u64>,
    sender: Sender<u64>,
    receiver: Receiver<u64>,
    stop: bool,
}

impl<T: Trigger + std::cmp::Ord> Scheduler<T> {
    fn new() -> Self {
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

    fn add_job(&mut self, job: T) {
        let next_firetime = job.get_next(Utc::now().timestamp() as u64);
        self.queue.push(job.get_id(), next_firetime);
    }

    fn remove_job(&mut self, id: String) {
        self.queue.remove(&id);
        self.triggers.remove(&id);
    }

    fn get_receiver(&self) -> Receiver<u64> {
        return self.receiver.clone();
    }

    async fn start(&mut self) {
        loop {
            if self.stop {
                return;
            }

            let t = self.queue.pop();
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
                        None => continue,
                    }
                }
                None => {}
            }
        }
    }

    async fn stop(&mut self) {
        self.stop = true
    }
}
