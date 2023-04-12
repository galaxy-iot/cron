use async_channel::{Receiver, Sender};
use chrono::DateTime;
use chrono::Utc;
use priority_queue::PriorityQueue;
use std::cmp::{Ord, PartialOrd};
use std::collections::HashMap;

pub trait Trigger {
    fn get_next(&self) -> i64;
    fn get_id(&self) -> String;
}

pub struct Scheduler<T: Trigger> {
    triggers: HashMap<String, T>,
    queue: PriorityQueue<String, i64>,
    sender: Sender<i64>,
    receiver: Receiver<i64>,
    stop: bool,
}

impl<T: Trigger + std::cmp::Ord> Scheduler<T> {
    fn new() -> Self {
        let queue = PriorityQueue::<String, i64>::new();
        let triggers = HashMap::<String, T>::new();
        let (sender, receiver) = async_channel::bounded::<i64>(128);
        Self {
            triggers,
            queue,
            sender,
            receiver,
            stop: false,
        }
    }

    fn add_job(&mut self, job: T) {
        let next_firetime = job.get_next();
        self.queue.push(job.get_id(), next_firetime);
    }

    fn remove_job(&mut self, id: String) {
        self.queue.remove(&id);
        self.triggers.remove(&id);
    }

    fn get_receiver(&self) -> Receiver<i64> {
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
                    let t = job.1;
                    _ = self.sender.send(t).await;

                    let trigger = self.triggers.get(&job_id);

                    match trigger {
                        Some(t) => {
                            let next_firetime = t.get_next();
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
