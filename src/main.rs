extern crate hyper;
extern crate hyper_native_tls;
extern crate pbr;
extern crate clap;
extern crate regex;
extern crate stderrlog;
#[macro_use]
extern crate log;
extern crate youtube_downloader;

use pbr::ProgressBar;
use std::{process,str};
use hyper::client::response::Response;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use hyper::header::ContentLength;
use std::io::Read;
use std::io::prelude::*;
use std::fs::File;
use clap::{Arg, App};
use regex::Regex;
use youtube_downloader::VideoInfo;

fn main() {
    //Regex for youtube URLs.
    let url_regex = Regex::new(r"^.*(?:(?:youtu\.be/|v/|vi/|u/w/|embed/)|(?:(?:watch)?\?v(?:i)?=|\&v(?:i)?=))([^#\&\?]*).*").unwrap();
    let args = App::new("youtube-downloader")
        .version("0.1.0")
        .arg(Arg::with_name("verbose")
             .help("Increase verbosity")
             .short("v")
             .multiple(true)
             .long("verbose"))
        .arg(Arg::with_name("adaptive")
             .help("List adaptive streams, instead of video streams")
             .short("A")
             .long("adaptive"))
        .arg(Arg::with_name("video-id")
            .help("The ID of the video to download.")
            .required(true)
            .index(1))
        .get_matches();

    stderrlog::new()
            .module(module_path!())
            .verbosity(args.occurrences_of("verbose") as usize)
            .init()
            .expect("Unable to initialize stderr output");

    let mut vid = args.value_of("video-id").unwrap();
    if url_regex.is_match(vid) {
        let vid_split = url_regex.captures(vid).unwrap();
        vid = vid_split.get(1).unwrap().as_str();
    }
    let url = format!("https://youtube.com/get_video_info?video_id={}", vid);
    download(&url, args.is_present("adaptive"));
}

fn download(url: &str, adaptive: bool) {
    debug!("Fetching video info from {}", url);
    let mut response = send_request(url);
    let mut response_str = String::new();
    response.read_to_string(&mut response_str).unwrap();
    trace!("Response {}", response_str);
    let info = VideoInfo::parse(&response_str).unwrap();
    debug!("Video info {:#?}", info);

    let streams = if adaptive {
        info.adaptive_streams
    } else {
        info.streams
    };

    for (i, stream) in streams.iter().enumerate() {
        println!("{}- {} {}",
                 i,
                 stream.quality,
                 stream.stream_type);
    }

    println!("Choose quality (0): ");
    let input = read_line().trim().parse().unwrap_or(0);

    println!("Please wait...");

    if let Some(ref stream) = streams.get(input) {
        // get response from selected quality
        debug!("Downloading {}", url);
        let response = send_request(&stream.url);
        println!("Download is starting...");

        // get file size from Content-Length header
        let file_size = get_file_size(&response);

        let filename = match stream.extension() {
            Some(ext) => format!("{}.{}", info.title, ext),
            None => info.title,
        };

        // write file to disk
        write_file(response, &filename, file_size);
    } else {
        error!("Invalid stream index");
    }
}

// get file size from Content-Length header
fn get_file_size(response: &Response) -> u64 {
    let mut file_size = 0;
    match response.headers.get::<ContentLength>(){
        Some(length) => file_size = length.0,
        None => println!("Content-Length header missing"),
    };
    file_size
}

fn write_file(mut response: Response, title: &str, file_size: u64) {
    // initialize progressbar
    let mut pb = ProgressBar::new(file_size);
    pb.format("╢▌▌░╟");

    // Download and write to file
    let mut buf = [0; 128 * 1024];
    let mut file = File::create(title).unwrap();
    loop {
        match response.read(&mut buf) {
            Ok(len) => {
                file.write_all(&buf[..len]).unwrap();
                pb.add(len as u64);
                if len == 0 {
                    break;
                }
                len
            }
            Err(why) => panic!("{}", why),
        };
    }
}

fn send_request(url: &str) -> Response {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    client.get(url).send().unwrap_or_else(|e| {
        error!("Network request failed: {}", e);
        process::exit(1);
    })
}

fn read_line() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Could not read stdin!");
    input
}
