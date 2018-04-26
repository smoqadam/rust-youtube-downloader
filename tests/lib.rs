
extern crate youtube_downloader;
extern crate reqwest;

use std::io::Read;
use youtube_downloader::{video_info_url_from_url, VideoInfo};

fn get_video_info(url: &str) -> VideoInfo {
    let info_url = youtube_downloader::video_info_url_from_url(url).unwrap();
    let mut resp = reqwest::get(&info_url).unwrap();
    let mut data = String::new();
    resp.read_to_string(&mut data).unwrap();
    youtube_downloader::VideoInfo::parse(&data).unwrap()
}

#[test]
fn live_video() {
    get_video_info("https://www.youtube.com/watch?v=XOacA3RYrXk");
}

#[test]
fn video() {
    get_video_info("https://www.youtube.com/watch?v=aqz-KE-bpKQ");
}
