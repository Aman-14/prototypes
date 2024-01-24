use std::fmt::Display;

const HASH_SEED: u32 = 48221234;

struct BloomFilter {
    buf: Vec<u8>,
    size: usize,
}

impl Display for BloomFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.buf))
    }
}

impl BloomFilter {
    fn new(size: usize) -> Self {
        return Self {
            buf: vec![0; size],
            size,
        };
    }
    fn add(&mut self, key: &str) {
        let hashed = hash(key, self.size);
        let index = hashed / 8;
        let bit_index = hashed % 8;

        println!("Before value: {:08b}", self.buf[index]);
        self.buf[index] = self.buf[index] | (1 << bit_index);
        println!("After value: {:08b}", self.buf[index]);
    }

    fn exists(&self, key: &str) -> bool {
        let hashed = hash(key, self.size);
        let index = hashed / 8;
        let bit_index = hashed % 8;
        return self.buf[index] & (1 << bit_index) != 0;
    }
}

fn hash(key: &str, size: usize) -> usize {
    let hashed = murmurhash3::murmurhash3_x86_32(key.as_bytes(), HASH_SEED) as usize;
    return hashed % size;
}

fn main() {
    let mut filter = BloomFilter::new(24);
    println!(
        "key - IsExists - {}",
        filter.exists("72196b97-505d-4e0a-b77asdfsdfasfssdfsdess-c8a8672fbfa3")
    );
    filter.add("72196b97-505d-4e0a-b77asdfsdfasfssdfsdess-c8a8672fbfa3");
    println!(
        "IsExists - {}",
        filter.exists("72196b97-505d-4e0a-b77asdfsdfasfssdfsdess-c8a8672fbfa3")
    );
    println!("aman - IsExists - {}", filter.exists("aman"));
    filter.add("aman");
    println!("aman - IsExists - {}", filter.exists("aman"));
}

#[test]
fn test_bloom_filter_add_exists() {
    let mut filter = BloomFilter::new(24);

    let key1 = "80a64dad-de92-4e06-b2e4-1a00136d0c73";
    let key2 = "aman";

    assert!(!filter.exists(key1));
    assert!(!filter.exists(key2));

    filter.add(key1);

    assert!(filter.exists(key1));
    assert!(!filter.exists(key2));

    filter.add(key2);

    assert!(filter.exists(key1));
    assert!(filter.exists(key2));
}
