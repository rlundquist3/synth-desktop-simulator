use std::collections::VecDeque;

use crate::effects::Effect;

#[derive(Debug)]
pub struct Echo {
    delay: u32,
    prev: VecDeque<f32>,
}

impl Echo {
    pub fn new(delay: u32) -> Self {
        let mut prev = VecDeque::new();
        for _ in 0..delay {
            prev.push_back(0.0);
        }

        Echo { delay, prev }
    }
}

impl Effect for Echo {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Echo::new(self.delay))
    }

    fn process(&mut self, sample: f32) -> f32 {
        let result = sample
            + 0.5
                * match self.prev.pop_front() {
                    Some(v) => v,
                    None => 0.0,
                };
        self.prev.push_back(result);

        result
    }
}
