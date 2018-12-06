use std::time::{SystemTime, UNIX_EPOCH};

pub const WORKER_ID_BITS: i64 = 5;
pub const DATACENTER_ID_BITS: i64 = 5;
pub const SEQUENCE_BITS: i64 = 12;

const MAX_WORKER_ID: i64 = -1 ^ (-1 << WORKER_ID_BITS);
const MAX_DATACENTER_ID: i64 = -1 ^ (-1 << DATACENTER_ID_BITS);

const WORKER_ID_SHIFT: i64 = SEQUENCE_BITS;
const DATACENTER_ID_SHIFT: i64 = SEQUENCE_BITS + WORKER_ID_BITS;
const TIMESTAMP_LEFT_SHIFT: i64 = SEQUENCE_BITS + WORKER_ID_BITS + DATACENTER_ID_BITS;

const SEQUENCE_MASK: i64 = -1 ^ (-1 << SEQUENCE_BITS);

// custom timestmap offset to reduce the generated value length.
const TIMESTAMP_OFFSET_EPOCH: i64 = 1540062491000;

fn gen_time() -> i64 {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let time = (duration.as_secs() * 1000 + duration.subsec_nanos() as u64 / 1_000_000) as i64
        - TIMESTAMP_OFFSET_EPOCH;
    if time < 0 {
        panic!("IdWorker: can't get correct time. current: {}", time);
    }
    time as i64
}

#[derive(Debug)]
pub struct IdWorker {
    worker_id: i64,
    datacenter_id: i64,
    sequence: i64,
    last_timestamp: i64,
}

impl IdWorker {
    pub fn new(worker_id: i64, datacenter_id: i64) -> Self {
        if worker_id < 0 || worker_id > MAX_WORKER_ID {
            panic!(
                "IdWorker: worker_id check failed: {}, MAX: {}",
                worker_id, MAX_WORKER_ID
            );
        }
        if datacenter_id < 0 || datacenter_id > MAX_DATACENTER_ID {
            panic!(
                "IdWorker: datacenter_id check failed: {}, MAX: {}",
                datacenter_id, MAX_DATACENTER_ID
            );
        }

        IdWorker {
            worker_id,
            datacenter_id,
            sequence: 0,
            last_timestamp: gen_time(),
        }
    }

    pub fn next_id(&mut self) -> i64 {
        let mut timestamp = gen_time();
        assert!(timestamp >= self.last_timestamp);

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & SEQUENCE_MASK;

            // overflow and block until next millisecond
            if self.sequence == 0 {
                loop {
                    timestamp = gen_time();
                    if timestamp > self.last_timestamp {
                        break;
                    }
                }
            }
        } else {
            self.sequence = 0;
        }
        self.last_timestamp = timestamp;
        (timestamp << TIMESTAMP_LEFT_SHIFT)
            | (self.datacenter_id << DATACENTER_ID_SHIFT)
            | (self.worker_id << WORKER_ID_SHIFT)
            | self.sequence
    }

    pub fn get_location(id: i64) -> (i64, i64) {
        let mut c_id = id;
        c_id >>= SEQUENCE_BITS;
        c_id = Self::slice(c_id, 64 - WORKER_ID_BITS - DATACENTER_ID_BITS);

        let worker_id = c_id >> 5;
        let dc_id = Self::slice(c_id, 64 - DATACENTER_ID_BITS);

        (worker_id, dc_id)
    }

    fn slice(id: i64, offset: i64) -> i64 {
        let c_id = id << offset - 1;
        let d_id = if c_id < 0 {
            c_id ^ (0 - std::i64::MAX - 1)
        } else {
            c_id
        };
        d_id >> offset - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut worker = IdWorker::new(31, 31);
        for _ in 0..10 {
            let id = worker.next_id();
            println!("{:?} ", worker.next_id());
            let (worker_id, dc_id) = IdWorker::get_location(id);
            println!("dc: {}, worker: {}", dc_id, worker_id);
        }
    }
}
