use std::{io::Read, time::SystemTime};

use anyhow::{anyhow, Result};
use integer_encoding::{VarInt, VarIntReader};
use store::Store;
mod store;

#[derive(Debug)]
struct Record {
    key: String,
    value: String,
}

impl Record {
    fn validate(crc: &u32, ts: &u32, key: &str, value: &str) -> bool {
        let mut buf = vec![];
        buf.append(&mut ts.encode_var_vec());
        buf.append(&mut key.len().encode_var_vec());
        buf.append(&mut value.len().encode_var_vec());
        buf.extend_from_slice(key.as_bytes());
        buf.extend_from_slice(value.as_bytes());

        let calculated_crc = crc32fast::hash(&buf);
        return *crc == calculated_crc;
    }

    fn serialize(&self) -> Vec<u8> {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        let mut buf = vec![];
        buf.append(&mut ts.encode_var_vec());
        buf.append(&mut self.key.len().encode_var_vec());
        buf.append(&mut self.value.len().encode_var_vec());
        buf.append(&mut self.key.clone().into_bytes());
        buf.append(&mut self.value.clone().into_bytes());

        let crc = calculate_checksum(&buf);
        let mut ser = crc.encode_var_vec();
        ser.append(&mut buf);
        return ser;
    }

    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<(Self, usize)> {
        println!("Len: {}", bytes.len());
        let mut total_bytes = 0;
        let (crc, read_bytes) =
            u32::decode_var(&bytes).ok_or_else(|| anyhow!("Failed to decode u32 from bytes"))?;

        total_bytes += read_bytes;
        let crc_bytes_sz = read_bytes;

        let buf = &bytes[read_bytes..];
        let (ts, read_bytes) =
            u32::decode_var(buf).ok_or_else(|| anyhow!("Failed to decode u32 from bytes"))?;

        total_bytes += read_bytes;

        let buf = &buf[read_bytes..];
        let (key_sz, read_bytes) =
            u32::decode_var(buf).ok_or_else(|| anyhow!("Failed to decode u32 from bytes"))?;

        total_bytes += read_bytes;

        let buf = &buf[read_bytes..];
        let (value_sz, read_bytes) =
            u32::decode_var(buf).ok_or_else(|| anyhow!("Failed to decode u32 from bytes"))?;

        total_bytes += read_bytes;

        let buf = &buf[read_bytes..];

        let key = String::from_utf8(buf[0..key_sz as usize].to_vec())?;

        total_bytes += key.len();
        let value = String::from_utf8(buf[key_sz as usize..(key_sz + value_sz) as usize].to_vec())?;

        total_bytes += value.len();

        let calculated_hash = crc32fast::hash(&bytes[crc_bytes_sz..total_bytes]);

        if crc != calculated_hash {
            return Err(anyhow!("Calculated hash not equal to crc"));
        }

        return Ok((Self { key, value }, total_bytes));
    }

    fn from_reader<R>(reader: &mut R) -> Result<Self>
    where
        R: VarIntReader + Read,
    {
        let crc: u32 = reader.read_varint()?;
        let ts: u32 = reader.read_varint()?;
        let key_sz: u32 = reader.read_varint()?;
        let value_sz: u32 = reader.read_varint()?;

        let mut buf = vec![0; key_sz as usize];
        reader.read_exact(&mut buf)?;
        let key = String::from_utf8(buf)?;

        buf = vec![0; value_sz as usize];
        reader.read_exact(&mut buf)?;
        let value = String::from_utf8(buf)?;

        // TODO: Do something
        let is_valid = Self::validate(&crc, &ts, &key, &value);

        return Ok(Record { key, value });
    }
}

fn calculate_checksum(buf: &Vec<u8>) -> u32 {
    let checksum = crc32fast::hash(buf);
    return checksum;
}

fn main() -> anyhow::Result<()> {
    let mut store = Store::new().unwrap();

    store
        .put("aman".to_string(), "a person".to_string())
        .unwrap();

    for _ in 1..1000000 {
        store
            .put("mac".to_string(), "a laptop".to_string())
            .unwrap();
    }

    let value = store.get("mac".to_string()).unwrap();

    println!("Mac Value before: {:?}", value);

    store.merge_and_compact()?;

    let value = store.get("mac".to_string()).unwrap();
    println!("Mac Value After: {:?}", value);

    let value = store.get("aman".to_string()).unwrap();
    println!("Aman Value After: {:?}", value);

    // store.delete("mac".to_string())?;
    // let value = store.get("mac".to_string()).unwrap();
    // println!("Mac Value after: {:?}", value);

    return Ok(());
}
