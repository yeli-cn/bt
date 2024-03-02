mod mikan;

use super::RssSubscription;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ParsingError {
    DownloadFailed(String, String),
    InvalidRss(String),
    UnrecognizedEpisode(String),
}

pub trait RssParser {
    fn parse_content(&self, content: &str) -> Result<RssSubscription, ParsingError>;

    async fn parse(&self, url: &str) -> Result<RssSubscription, ParsingError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();
        let content = match client.get(url).send().await {
            Ok(response) => match response.text().await {
                Ok(content) => content,
                Err(err) => {
                    return Err(ParsingError::DownloadFailed(url.to_string(), err.to_string()))
                }
            },
            Err(err) => return Err(ParsingError::DownloadFailed(url.to_string(), err.to_string())),
        };
        self.parse_content(&content)
    }
}

pub async fn parse(url: &str) -> Result<RssSubscription, ParsingError> {
    // TODO: now we only have mikan parser
    let parser = mikan::MikanParser::new();
    parser.parse(url).await
}