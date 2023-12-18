use std::time::{SystemTime, UNIX_EPOCH};

fn get_shard_id() -> u64 {
    return 5;
}

fn get_seq_id() -> u64 {
    return 100;
}

fn main() {
    let now = SystemTime::now();
    let now_ms: u64 = now.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let epoch_2011 = 1314220021721;
    let shard_id = get_shard_id();
    let seq_id = get_seq_id();

    let mut result = (now_ms - epoch_2011) << 23;
    result = result | (shard_id << 10);
    result = result | (seq_id);

    println!("Snowflake Generated - {}", result);
}
