use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

/** Each chunks size (Ref from - https://github.com/uutils/coreutils/blob/main/src/uu/tail/src/chunks.rs#L20 ) */
const BLOCK_SIZE: u64 = 1 << 16;

pub struct ReverseChunks<'a> {
    file: &'a File,
    max_chunks_to_read: u64,
    current_block_idx: u64,
    file_size: u64,
}

impl<'a> ReverseChunks<'a> {
    pub fn new(file: &'a mut File) -> Self {
        let size = file.seek(SeekFrom::End(0)).unwrap();
        println!("Seize: {}", size);
        let max_chunks_to_read = ((size as f64) / (BLOCK_SIZE as f64)).ceil() as u64;
        println!("Max chunks - {}", max_chunks_to_read);
        return Self {
            file,
            max_chunks_to_read,
            current_block_idx: 0,
            file_size: size,
        };
    }
}

impl<'a> Iterator for ReverseChunks<'a> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_block_idx >= self.max_chunks_to_read {
            return None;
        }

        let block_size = {
            if self.current_block_idx == self.max_chunks_to_read - 1 {
                self.file_size % BLOCK_SIZE
            } else {
                BLOCK_SIZE
            }
        };
        let mut buf = vec![0; block_size as usize];

        println!();
        println!("Max Block size - {}", BLOCK_SIZE);
        println!("Current block size - {}", block_size);
        println!("Block index {}", self.current_block_idx);
        println!("Total blocks- {}", self.max_chunks_to_read);

        self.file
            .seek(SeekFrom::Current(-(block_size as i64)))
            .unwrap();

        self.file.read_exact(&mut buf).unwrap();

        self.file
            .seek(SeekFrom::Current(-(block_size as i64)))
            .unwrap();

        self.current_block_idx += 1;

        return Some(buf);
    }
}
