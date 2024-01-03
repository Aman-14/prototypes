use std::sync::{Arc, Mutex};
use std::{collections::HashMap, thread};

fn fill_map(map: &mut HashMap<String, usize>) {
    println!("Inside Fill map: Current map: {:?}", map);
    for i in 0..10 {
        map.insert(i.to_string(), i as usize);
    }
}

fn main() {
    let source_map = Arc::new(Mutex::new(HashMap::new()));

    fill_map(&mut source_map.lock().unwrap());

    let map_for_thread = source_map.clone();
    thread::spawn(move || {
        loop {
            let mut map = map_for_thread.lock().unwrap();
            println!("Map in thread before {:?}", map);

            println!("Map length {}", map.len());
            if map.len() == 0 {
                // unlock and sleep
                drop(map);
                thread::sleep(std::time::Duration::from_secs(2));
                continue;
            }

            // take the value of *map and set it as default HashMap thats what `take` does
            let temp = std::mem::take(&mut *map);
            println!("Map in thread after {:?}", map);

            // unlock the mutex by dropping map
            drop(map);

            process_map(temp);
            thread::sleep(std::time::Duration::from_secs(1))
        }
    });

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        fill_map(&mut source_map.lock().unwrap());
    }
    // handler.join().unwrap();
}

fn process_map(mut map: HashMap<String, usize>) {
    // do anything on map
    map.remove("remove");
}
