use std::fs::File;
use std::io::Write;
use std::io::{Seek, SeekFrom};
use std::sync::atomic::{Ordering, AtomicBool};
use hyper::{Client, Request, Body};
use hyper::rt::{self, Future, Stream};
use web_view::Handle;

pub fn run(is_downloading: &'static AtomicBool, handle1: Handle<()>, handle2: Handle<()>) {
    static SOUNDSENSE_URL: &str = "http://df.zweistein.cz/soundsense/soundpack.zip";
    static BODY_STR: &str = r#"{"library":"soundsense-rs"}"#;
    rt::run(rt::lazy(move || {
        let client = Client::new();
        let head_req = Request::head(SOUNDSENSE_URL)
            .body(Body::from(BODY_STR))
            .expect("Failed to build HEAD request.");
        
        let get_req = Request::get(SOUNDSENSE_URL)
            .body(Body::from(BODY_STR))
            .expect("Failed to build GET request.");
        
        client.request(head_req)
            .and_then(|res| {
                println!("Reseponse: {}\nHeaders: {:#?}", res.status(), res.headers());
                Ok(())
            })
            .and_then(move |_| client.request(get_req))
            .and_then(move |res| {
                let headers = res.headers();
                println!("Reseponse: {}\nHeaders: {:#?}", res.status(), headers);

                let file_size: u64 = headers
                    .get("content-length").unwrap()
                    .to_str().unwrap()
                    .parse().unwrap();
        
                handle1.dispatch(|webview| {
                    let script = r#"
let download_bar = document.getElementById('download_bar');
download_bar.className = download_bar.className.replace(/(?:^|\s)w3-hide(?!\S)/g, '');
let progress_bar = document.getElementById('download_progress_bar');
progress_bar.innerText = '0%';
progress_bar.style.width = '0%';
                    "#;
                    webview.eval(script)
                }).unwrap();                

                let mut file = File::create("./soundpack_tmp").unwrap();
                file.set_len(file_size).unwrap();
                res.into_body().for_each(move |chunk| {
                    let current_pos = file.seek(SeekFrom::Current(0)).unwrap();

                    let script = format!(r#"
let progress_bar = document.getElementById('download_progress_bar');
progress_bar.innerText = '{percentage}%';
progress_bar.style.width = '{percentage}%';
                    "#,
                        percentage = (current_pos * 100 / file_size),
                    );
                
                    handle1.dispatch(move |webview| webview.eval(&script))
                        .unwrap();
                    file.write_all(&chunk)
                        .map_err(|e| panic!("Error during writing into file: {}", e))
                })
            })
            .map(move |_| {
                std::fs::rename("./soundpack_tmp", "./soundpack.zip").unwrap();
                handle2.dispatch(|webview| {
                    let script = r#"
let download_bar = document.getElementById('download_bar');
download_bar.className += ' w3-hide';
                    "#;
                    webview.eval(script)
                }).unwrap();
                is_downloading.swap(false, Ordering::SeqCst);
                println!("Download complete!");
            })
            .map_err(|err| {
                panic!("Error: {}" ,err);
            })
    }));
}