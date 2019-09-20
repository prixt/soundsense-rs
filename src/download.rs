use std::fs::File;
use std::io::Write;
use hyper::{Client, Request, Body};
use hyper::rt::{self, Future, Stream};

pub fn run(_thread_count: usize) {
    static SOUNDSENSE_URL: &str = "http://df.zweistein.cz/soundsense/soundpack.zip";
    rt::run(rt::lazy(|| {
        let client = Client::new();
        let head_req = Request::head(SOUNDSENSE_URL)
            .body(Body::from(r#"{"library":"soundsense-rs"}"#))
            .expect("Failed to build HEAD request.");
        
        let get_req = Request::get(SOUNDSENSE_URL)
            .body(Body::from(r#"{"library":"soundsense-rs"}"#))
            .expect("Failed to build GET request.");
        
        client.request(head_req)
            .and_then(|res| {
                println!("Response: {}", res.status());
                println!("Headers: {:#?}", res.headers());
                Ok(())
            })
            .and_then(move |_| client.request(get_req))
            .and_then(|res| {
                println!("Response: {}", res.status());
                let headers = res.headers();
                println!("Headers: {:#?}", headers);

                let file_size: u64 = headers
                    .get("content-length").unwrap()
                    .to_str().unwrap()
                    .parse().unwrap();

                let mut file = File::create("./soundpack_tmp").unwrap();
                file.set_len(file_size).unwrap();
                res.into_body().for_each(move |chunk| {
                    file.write_all(&chunk)
                        .map_err(|e| panic!("Error during writing into file: {}", e))
                })
            })
            .map(|_| {
                std::fs::rename("./soundpack_tmp", "./soundpack.zip").unwrap();
                println!("Download complete!");
            })
            .map_err(|err| {
                panic!("Error: {}" ,err);
            })
    }));
}