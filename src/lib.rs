//! Parser for youtube video information as returned by
//! https://youtube.com/get_video_info?video_id={}

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_urlencoded;
extern crate url;

#[derive(Deserialize, Debug)]
pub struct Stream {
    pub url: String,
    #[serde(default = "String::new")]
    pub quality: String,
    #[serde(rename = "type")]
    pub stream_type: String,
}

impl Stream {
    pub fn extension(&self) -> Option<&str> {
        self.stream_type.split(';')
            .next()
            .and_then(|mimetype| mimetype.split('/').nth(1))
    }
}

#[derive(Deserialize, Debug)]
struct VideoInfoResponse {
    author: String,
    video_id: String,
    status: String,
    title: String,
    thumbnail_url: String,
    url_encoded_fmt_stream_map: String,
    view_count: usize,
    adaptive_fmts: Option<String>,
    hlsvp: Option<String>,
}

impl VideoInfoResponse {
    pub fn fmt_streams(&self) -> Result<Vec<Stream>, serde_urlencoded::de::Error> {
        let mut result = Vec::new();

        // this field may be empty
        if self.url_encoded_fmt_stream_map.is_empty() {
            return Ok(result);
        }

        // This field has a list of encoded stream dicts separated by commas
        for input in self.url_encoded_fmt_stream_map.split(',') {
            result.push(serde_urlencoded::from_str(input)?);
        }
        Ok(result)
    }

    pub fn adaptive_streams(&self) -> Result<Vec<Stream>, serde_urlencoded::de::Error> {
        let mut result = Vec::new();
        if let Some(ref fmts) = self.adaptive_fmts {
            // This field has a list of encoded stream dicts separated by commas
            for input in fmts.split(',') {
                result.push(serde_urlencoded::from_str(input)?);
            }
        }
        Ok(result)
    }
}

#[derive(Debug)]
pub struct VideoInfo {
    pub author: String,
    pub video_id: String,
    pub title: String,
    pub thumbnail_url: String,
    pub streams: Vec<Stream>,
    pub view_count: usize,
    pub adaptive_streams: Vec<Stream>,
    /// Video URL for videos with HLS streams
    pub hlsvp: Option<String>,
}

impl VideoInfo {
    pub fn parse(inp: &str) -> Result<VideoInfo, Error> {
        let resp: VideoInfoResponse = match serde_urlencoded::from_str(inp) {
            Ok(r) => r,
            Err(original_err) => {
                // attempt to decode error info
                let error_info: ErrorInfo = match serde_urlencoded::from_str(inp) {
                    Ok(error_info) => error_info,
                    Err(_) => return Err(Error::from(original_err)),
                };
                return Err(Error::from(error_info));
            }
        };
        let streams = resp.fmt_streams()?;
        let adaptive_streams = resp.adaptive_streams()?;
        Ok(VideoInfo {
            author: resp.author,
            video_id: resp.video_id,
            title: resp.title,
            thumbnail_url: resp.thumbnail_url,
            streams: streams,
            view_count: resp.view_count,
            adaptive_streams: adaptive_streams,
            hlsvp: resp.hlsvp,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ErrorInfo {
    pub reason: String,
}

#[derive(Debug)]
pub enum Error {
    JsonError(serde_urlencoded::de::Error),
    Youtube(ErrorInfo),
    Url(url::ParseError),
    UrlMissingVAttr,
}

impl From<serde_urlencoded::de::Error> for Error {
    fn from(e: serde_urlencoded::de::Error) -> Self {
        Error::JsonError(e)
    }
}

impl From<ErrorInfo> for Error {
    fn from(e: ErrorInfo) -> Self {
        Error::Youtube(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::Url(e)
    }
}

/// The URL to grab video information, the video_id is passed in as a query argument.
///
/// See 'video_info_url()'.
pub const GET_VIDEO_INFO_URL: &str = "https://youtube.com/get_video_info";

/// Build the URL to retrieve the video information from a video id
pub fn video_info_url(vid: &str) -> String {
    let vid = url::percent_encoding::utf8_percent_encode(vid, url::percent_encoding::DEFAULT_ENCODE_SET).to_string();
    format!("{}?video_id={}", GET_VIDEO_INFO_URL, vid)
}

/// Build the URL to retrieve the video information from a video url
pub fn video_info_url_from_url(video_url: &str) -> Result<String, Error> {
    let url = url::Url::parse(video_url)?;

    let mut vid = None;
    for (name, value) in url.query_pairs() {
        if name == "v" {
            vid = Some(value);
        }
    }

    let vid = vid.ok_or(Error::UrlMissingVAttr)?;
    let vid = url::percent_encoding::utf8_percent_encode(&vid, url::percent_encoding::DEFAULT_ENCODE_SET).to_string();
    Ok(format!("{}?video_id={}", GET_VIDEO_INFO_URL, vid))
}
