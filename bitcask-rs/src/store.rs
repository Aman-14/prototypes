use std::{
    collections::HashMap,
    fmt::Debug,
    fs::{self, OpenOptions},
    io::{BufReader, Seek, Write},
    os::unix::prelude::{FileExt, MetadataExt},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;

use crate::Record;

const DIR: &str = "dbs/";
const MAX_FILE_SZ: u32 = 41_94_304; // 4 MB
const TOMBSTONE: &str = "<=>";
const ESCAPED_TOMBSTONE: &str = "<=><=>";

fn escape_tombstone(val: String) -> String {
    return val.replace(TOMBSTONE, ESCAPED_TOMBSTONE);
}

fn unescape_tombstone(val: String) -> String {
    return val.replace(ESCAPED_TOMBSTONE, TOMBSTONE);
}

fn get_sortable_id() -> String {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
}

fn open_file(id: &String) -> Result<fs::File> {
    let file_name = format!("{DIR}/{id}");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_name)?;
    return Ok(file);
}

#[derive(Debug)]
struct FilWithId {
    id: String,
    file: fs::File,
}

#[derive(Debug)]
struct KeyDirValue {
    file_id: String,
    value_sz: u32,
    value_posi: u32,
}

#[derive(Debug)]
pub struct Store {
    key_dir: HashMap<String, KeyDirValue>,
    files: Vec<FilWithId>,
    active_file_id: String,
    cur_posi: u32,
}

impl Store {
    pub fn new() -> Result<Self> {
        let dir_path = Path::new(DIR);
        let mut paths: Vec<_> = fs::read_dir(dir_path)?
            .filter_map(|e| {
                if let Ok(e) = e {
                    let path = e.path();
                    if path.is_file() {
                        return path.file_name().map(|el| el.to_string_lossy().to_string());
                    }
                }
                return None;
            })
            .collect();

        println!("Paths: {:?}", paths);
        paths.sort_by(|a, b| a.cmp(b));

        let mut files: Vec<FilWithId> = vec![];
        let mut active_file_id: Option<String> = None;
        let mut key_dir: HashMap<String, KeyDirValue> = HashMap::new();
        let n_paths = paths.len();

        for (i, path) in paths.into_iter().enumerate() {
            active_file_id = Some(path.clone());

            let mut open_options = OpenOptions::new();
            // active file will be at last index so open it with write perms
            if i == n_paths - 1 {
                open_options.write(true);
            }

            let file = open_options.read(true).open(format!("{DIR}{path}"))?;
            let file_sz = file.metadata()?.size() as u32;
            let mut reader = BufReader::new(&file);

            loop {
                let cur_posi = reader.seek(std::io::SeekFrom::Current(0))? as u32;
                if cur_posi >= file_sz {
                    break;
                }
                let record = Record::from_reader(&mut reader)?;
                if record.value == TOMBSTONE {
                    continue;
                };
                key_dir.insert(
                    record.key,
                    KeyDirValue {
                        // ts,
                        value_sz: 1,
                        file_id: active_file_id.clone().unwrap(),
                        value_posi: cur_posi,
                    },
                );
            }

            files.push(FilWithId { id: path, file });
        }

        println!("Active file {:?}", active_file_id);

        if active_file_id.is_none() {
            let id = get_sortable_id();
            active_file_id = Some(id.clone());

            let active_file = open_file(&id)?;

            files.push(FilWithId {
                id,
                file: active_file,
            });
        }
        let files_length = files.len();

        let cur_posi = files
            .get_mut(files_length - 1)
            .unwrap()
            .file
            .seek(std::io::SeekFrom::End(0))? as u32;

        let active_file_id = active_file_id.unwrap();
        println!("Key Dir: {:?}", key_dir);
        return Ok(Self {
            key_dir,
            files,
            cur_posi,
            active_file_id,
        });
    }

    pub fn put(&mut self, key: String, value: String) -> Result<()> {
        let value = escape_tombstone(value);
        let record = Record { key, value };
        let serialized = record.serialize();
        // println!("Active file id:{}", self.active_file_id);
        // println!("Files:{:?}", self.files);
        let file = self
            .files
            .iter_mut()
            .find(|f| f.id == self.active_file_id)
            .unwrap();

        println!("File: {:?}", file);
        let written = file.file.write(&serialized)? as u32;
        self.key_dir.insert(
            record.key,
            KeyDirValue {
                file_id: self.active_file_id.clone(),
                // value_sz: written,
                value_sz: record.value.len() as u32,
                value_posi: self.cur_posi.clone(),
            },
        );

        self.cur_posi += written;

        if self.cur_posi >= MAX_FILE_SZ {
            let id = get_sortable_id();
            let file = open_file(&id)?;

            //TODO:  consider using lifetimes to avoid clones
            self.files.push(FilWithId {
                id: id.clone(),
                file,
            });
            self.active_file_id = id;
            self.cur_posi = 0
        }

        Ok(())
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        let val = self.key_dir.get(&key);

        println!("Key: {}, val: {:?}", key, val);
        if let Some(val) = val {
            // taking integers as 5 bytes because the maximum size of u32 in google protobuf
            // encoding is 5 bytes
            let n = val.value_sz + key.len() as u32 + 5 /* crc */ + 5 /* ts */+ 5 /* keysz */+ 5 /* valuesz */;
            // let n = val.value_sz;
            let mut buf: Vec<u8> = vec![0; n as usize];
            let FilWithId { file, .. } = self.files.iter().find(|f| f.id == val.file_id).unwrap();
            file.read_at(&mut buf, val.value_posi as u64)?;

            let (record, _read_bytes) = Record::from_bytes(buf)?;
            let value = unescape_tombstone(record.value);
            return Ok(Some(value));
        }
        return Ok(None);
    }

    pub fn delete(&mut self, key: String) -> Result<()> {
        let val = self.key_dir.get(&key);

        if val.is_none() {
            return Ok(());
        }

        let record = Record {
            key,
            value: TOMBSTONE.to_string(),
        };

        let written = self
            .files
            .iter_mut()
            .find(|f| f.id == self.active_file_id)
            .unwrap()
            .file
            .write(&record.serialize())?;

        self.key_dir.remove(&record.key);
        self.cur_posi += written as u32;
        return Ok(());
    }

    pub fn merge_and_compact(&mut self) -> Result<()> {
        let id = (self.files[0].id.parse::<u32>()? - 1).to_string();
        let mut new_file = open_file(&id)?;

        let mut writer_posi = 0;

        let mut processed_ids = vec![];
        for f in self.files.iter_mut() {
            // active file won't be touched
            if f.id == self.active_file_id {
                continue;
            }
            f.file.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut reader = BufReader::new(&f.file);

            let file_sz = f.file.metadata()?.size() as u32;

            loop {
                let reader_posi = reader.seek(std::io::SeekFrom::Current(0)).unwrap() as u32;
                println!("Reader poosi: {}", reader_posi);

                if reader_posi >= file_sz {
                    break;
                }
                let record = Record::from_reader(&mut reader);
                if record.is_err() {
                    println!("Got error when reading row. Ignoring it");
                    continue;
                }
                let record = record.unwrap();
                let key_dir_value = self.key_dir.get(&record.key);
                println!("Record inside merge {:?}", record);

                if let Some(key_dir_value) = key_dir_value {
                    println!(
                        "Key dir value inside merge {:?}\nReader posi {}",
                        key_dir_value, reader_posi
                    );
                    if key_dir_value.file_id == f.id && key_dir_value.value_posi == reader_posi {
                        let written = new_file.write(&record.serialize())? as u32;
                        self.key_dir.insert(
                            record.key,
                            KeyDirValue {
                                file_id: id.clone(),
                                value_sz: record.value.len() as u32,
                                value_posi: writer_posi,
                            },
                        );
                        writer_posi += written;
                    };
                };
            }
            processed_ids.push(f.id.clone());
        }

        self.files.retain(|f| !processed_ids.contains(&f.id));
        self.files.insert(0, FilWithId { file: new_file, id });

        for pid in processed_ids {
            fs::remove_file(format!("{DIR}/{pid}"))?
        }

        return Ok(());
    }
}
