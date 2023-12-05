use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
};

use postgres::{Client, Error, NoTls};

struct Pool {
    conns: Mutex<Vec<Client>>,
    cvar: Condvar,
}

impl Pool {
    fn new(max_conns: u8) -> Self {
        return Pool {
            conns: Mutex::new(
                (0..max_conns)
                    .map(|_| {
                        Client::connect(
                            "host=localhost user=aman password=aman dbname=ibowl",
                            NoTls,
                        )
                        .expect("Unable to connect to database")
                    })
                    .collect(),
            ),
            cvar: Condvar::new(),
        };
    }

    fn get(&self) -> Result<Client, Error> {
        println!("Inside get");
        let mut conns = self.conns.lock().expect("Unable to lock connections");
        println!("After locking in get");

        if let Some(conn) = conns.pop() {
            return Ok(conn);
        }

        while conns.is_empty() {
            conns = self.cvar.wait(conns).expect("Error while waiting for cvar");
        }

        return Ok(conns
            .pop()
            .expect("Connection vector is unexpectedly empty"));
    }

    fn put(&self, client: Client) -> () {
        let mut conns = self.conns.lock().expect("Unable to lock connections");
        conns.push(client);
        self.cvar.notify_one();
    }
}

fn main() {
    println!("Hello, world!");
    let pool = Arc::new(Pool::new(5));

    let handles: Vec<_> = (0..100)
        .map(|_| {
            let pool = Arc::clone(&pool);
            thread::spawn(move || {
                let client_res = pool.get();
                match client_res {
                    Ok(mut client) => {
                        let data = client.query("SELECT * FROM result", &[]).unwrap();
                        println!("Fetched - {} rows", data.len());
                        pool.put(client);
                    }

                    Err(err) => {
                        eprintln!("{}", err);
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
