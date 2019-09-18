use std::fs::{File, OpenOptions};
use std::io::{copy, Seek, SeekFrom};
use std::sync::{Arc, atomic::{Ordering, AtomicUsize}};
use reqwest::header::{CONTENT_LENGTH, RANGE};
use reqwest::StatusCode;

pub fn run(thread_count: usize) {
    const URL: &str = "http://df.zweistein.cz/soundsense/soundpack.zip";

    let client = reqwest::Client::new();
    let response = client.head(URL).send().unwrap();
    let length = response.headers()
        .get(CONTENT_LENGTH).unwrap()
        .to_str().unwrap()
        .parse::<u64>().unwrap();
    // let mut initial_thread_count = 0;
    let chunk_size = length / thread_count as u64;

    let output_file = File::create("./soundpack_tmp").unwrap();
    output_file.set_len(length).unwrap();
    
    let thread_count = Arc::new(AtomicUsize::new(0));
    let client = Arc::new(client);
    for range_start in (0..length).step_by(chunk_size as usize) {
        let thread_count = thread_count.clone();
        let client = client.clone();
        
        // initial_thread_count += 1;
        thread_count.fetch_add(1, Ordering::SeqCst);

        let range_end = std::cmp::min(range_start+chunk_size, length);
        std::thread::spawn(move || {
            loop {
                let request = client.get(URL).header(RANGE, format!("bytes={}-{}", range_start, range_end-1));
                let mut response = request.send().unwrap();
                match response.status() {
                    StatusCode::OK | StatusCode::PARTIAL_CONTENT => {
                        let mut file = OpenOptions::new()
                            .write(true)
                            .open("./soundpack_tmp")
                            .unwrap();
                        file.seek(SeekFrom::Start(range_start)).unwrap();
                        copy(&mut response, &mut file).unwrap();
                        thread_count.fetch_sub(1, Ordering::SeqCst);
                        break
                    },
                    StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => continue,
                    other => {
                        panic!("Encountered error: StatusCode::{}", other)
                    }
                }
            }
        });
    }

    loop {
        let current_thread_count = thread_count.load(Ordering::Relaxed);
        println!("{}", current_thread_count);
        if current_thread_count == 0 {
            break
        }
        std::thread::sleep(
            std::time::Duration::from_secs(2)
        );
    }

    std::fs::rename("./soundpack_tmp", "./soundpack.zip").unwrap();
}