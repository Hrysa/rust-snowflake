use rust_snowflake::IdWorker;

fn main() {
    let mut worker = IdWorker::new(0, 0, 1586497735375);

    for _ in 0..10 {
        println!("{}", worker.next_id());
    }
}
