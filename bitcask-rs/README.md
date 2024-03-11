Bitcask is a high-performance, append-only storage engine designed for key-value data.
It's known for its simplicity, efficiency, and ability to handle large amounts of data with low latency.

### Record Structure

#### On Disk

One record consists of crc, ts, key_zs, value_sz, key and value on disk.

#### In memory key dir

A hashmap where with key and value as file_id, value_sz, value_pos, ts.
We are only saving the value size not the size of whole record because other fields have fixes
size and size of the record can be calculated.

### Integer encoding in rust

```rust
use integer_encoding::{VarIntReader, VarIntWriter};

fn main() {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("db.dat")
        .unwrap();

    file.write_varint(11 as u32).unwrap();
    file.write_varint(29 as u32).unwrap();

    file.seek(std::io::SeekFrom::Start(0)).unwrap();

    println!("a: {}", file.read_varint::<u32>().unwrap());
    println!("b: {}", file.read_varint::<u32>().unwrap());
}
```

### Checksum Calculation

CRC: https://github.com/srijs/rust-crc32fast
![crc structure]("./assets/Crc-structure.png")
