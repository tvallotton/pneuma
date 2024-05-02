use std::{cell::RefCell, collections::VecDeque, sync::Mutex};

use thread_local::ThreadLocal;

use crate::{sys::stack::Stack, uthread::UThread};

#[derive(Default)]
pub(crate) struct Executor {
    pub current: ThreadLocal<Mutex<UThread>>,
    pub local_queue: ThreadLocal<Mutex<VecDeque<UThread>>>,
    pub global_queue: Mutex<VecDeque<UThread>>,
    pub all: Mutex<VecDeque<UThread>>,
    pub _unused_stacks: Mutex<Vec<Stack>>,
}

impl Executor {
    pub fn yield_to(&self) -> Result<(), ()> {
        let Err(_) = self.yield_locally() else {
            return Ok(());
        };

        self.steal_work();
        todo!()
    }

    pub fn even_queues(&self) {
        let mut queues: Vec<_> = self
            .local_queue
            .iter()
            .filter_map(|queue| queue.try_lock().ok())
            .collect();

        let items = queues.iter().map(|queue| queue.len()).sum();

        let min_len = items / queues.len();
        let remainder = items % queues.len();

        if target_len <= 50 {
            return;
        }

        queues.sort_by_key(|queue| queue.len());

        for recipient in 0..queues.len() {
            let mut donor = recipient + 1;
            let extra = (recipient < remainder) as usize;
            while queues[recipient].len() < min_len + extra {
                let Some(thread) = queues[donor].pop_back() else {
                    donor += 1;
                    break;
                };
                queues[recipient].push_back(thread);
            }
        }
    }

    fn steal_work(&mut self) -> Result<(), ()> {
        let mut donor = self
            .local_queue
            .iter()
            .filter_map(|queue| queue.try_lock().ok())
            .max_by_key(|queue| queue.len())
            .ok_or(())?;

        let mut queue = self.local_queue.get().unwrap().lock().unwrap()
        while  queue.len() >= donor.len() {
            let Some(thread) = donor.pop_front() else {
                break;
            };
            queue.push_back(thread);
        }
        
        if queue.is_empty() {
            return Err(())
        } 
        Ok(())
    }

    fn yield_locally(&mut self) -> Result<(), ()> {
        todo!()
    }
}
