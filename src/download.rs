use std::fs::{File, OpenOptions};
use std::io::{copy, Seek, SeekFrom};
use std::sync::{Arc, Mutex, atomic::{Ordering, AtomicUsize}};
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
    let mut chunk_count = 0;
    let chunk_size = length as usize / thread_count;

    let file = File::create("./soundpack_tmp").unwrap();
    file.set_len(length).unwrap();

    let chunks: Vec<_> = (0..length).step_by(chunk_size)
        .map(|range_start| {
            chunk_count += 1;
            let range_end = std::cmp::min(range_start + chunk_size as u64, length)-1;
            (range_start, range_end)
        })
        .collect();
    
    let client = Arc::new(client);
    let chunks = Arc::new(Mutex::new(chunks));
    let cleared_chunks = Arc::new(AtomicUsize::new(0));
    (0..thread_count).for_each(|_| {
        let client = client.clone();
        let chunks = chunks.clone();
        let cleared_chunks = cleared_chunks.clone();
        std::thread::spawn(move || {
            while let Some((range_start, range_end)) = chunks.lock().unwrap().pop() {
                let mut file = OpenOptions::new()
                    .write(true)
                    .open("./soundpack_tmp")
                    .unwrap();
                file.seek(SeekFrom::Start(range_start)).unwrap();
                loop {
                    let mut response = client.get(URL)
                        .header(RANGE, format!("bytes={}-{}", range_start, range_end))
                        .send().unwrap();
                    match response.status() {
                        StatusCode::OK | StatusCode::PARTIAL_CONTENT => {
                            copy(&mut response, &mut file).unwrap();
                            cleared_chunks.fetch_add(1, Ordering::AcqRel);
                            break
                        },
                        StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => continue,
                        other => panic!("Unexpected response status: StatusCode::{}", other),
                    }
                }
            }
        });
    });

    loop {
        let cleared_chunks = cleared_chunks.load(Ordering::Relaxed);
        if cleared_chunks == chunk_count {
            break
        }
        println!("{:.2}%", (cleared_chunks * 100) as f32 / chunk_count as f32);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    println!("Download Complete!");
    std::fs::rename("./soundpack_tmp", "./soundpack.zip").unwrap();
}