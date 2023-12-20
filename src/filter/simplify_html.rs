use readability::extractor::extract;
use serde::{Deserialize, Serialize};

use crate::feed::Feed;
use crate::util::Result;

use super::{FeedFilter, FeedFilterConfig};

#[derive(Serialize, Deserialize)]
pub struct SimplifyHtmlConfig {}

pub struct SimplifyHtmlFilter;

#[async_trait::async_trait]
impl FeedFilterConfig for SimplifyHtmlConfig {
  type Filter = SimplifyHtmlFilter;

  async fn build(&self) -> Result<Self::Filter> {
    Ok(SimplifyHtmlFilter)
  }
}

#[async_trait::async_trait]
impl FeedFilter for SimplifyHtmlFilter {
  async fn run(&self, feed: &mut Feed) -> Result<()> {
    for post in feed.posts.iter_mut() {
      if let Some(description) = simplify(&post.description, &post.link) {
        post.description = description;
      };
    }

    Ok(())
  }
}

fn simplify(text: &str, url: &str) -> Option<String> {
  let url = reqwest::Url::parse(url).ok()?;
  let mut text = std::io::Cursor::new(text);
  let product = extract(&mut text, &url).ok()?;
  Some(product.content)
}