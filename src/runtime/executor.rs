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
            return Ok(())
        };

        self.steal_work(); 
        todo!()
        
        
    }

    pub fn even_queues(&self) {

        let mut queues: Vec<_> = self.local_queue.iter().map(|queue|queue.lock().unwrap()).collect();

        let items = queues.iter().map(|queue|queue.len()).sum();
        
        let min_len   = items / queues.len();
        let remainder = items % queues.len();

        if target_len <= 50 {
            return
        }

        queues.sort_by_key(|queue| queue.len());

        for recipient in 0..queues.len() {
            let mut donor = recepient + 1;
            let extra = (recipient < remainder) as usize;
            while queues[recipient].len() < min_len + extra {
                let Some(thread) = queues[donor].pop_back() else {
                    break donor += 1;
                };
                queues[recipient].push_back(thread);
            }
        }
    }



    fn steal_work(&mut self) -> Result<(), ()> {
        todo!()
    }

    fn yield_locally(&mut self) -> Result<(), ()>{
        todo!()
    }
}



fn even_out