use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    thread, time,
};

use crate::tail::chunks::ReverseChunks;

/** Seeks to the position where n files are found */
fn find_lines_backwards(file: &mut File, num_lines: usize) {
    let blocks = ReverseChunks::new(file);

    let mut found_lines = 0;

    for block in blocks {
        // Reverse the enumerate so `i` will be preserved
        for (i, ch) in block.iter().enumerate().rev() {
            if *ch == b'\n' {
                found_lines += 1;
                if found_lines >= num_lines {
                    file.seek(SeekFrom::Current((i + 1) as i64)).unwrap();
                    return;
                }
            }
        }
    }
}

pub fn get_last_lines(file: &mut File, num_lines: usize) -> Vec<u8> {
    let num_lines = num_lines + 1;
    find_lines_backwards(file, num_lines);

    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();

    return buf;
}

pub fn follow_file<W: Write, F: Fn(&mut Vec<u8>) -> Vec<u8>>(
    file: &mut File,
    writer: &mut W,
    formatter: F,
) {
    let mut size = file.metadata().unwrap().len();

    loop {
        let new_size = file.metadata().unwrap().len();

        if new_size == size {
            continue;
        }

        if new_size < size {
            file.seek(SeekFrom::Start(0)).unwrap();
        }
        let mut buf = vec![];
        file.read_to_end(&mut buf).unwrap();

        let buf = formatter(&mut buf);

        writer.write(&buf).unwrap();
        writer.flush().unwrap();
        size = new_size;
        thread::sleep(time::Duration::from_millis(100));
    }
}
