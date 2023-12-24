use serde::{Deserialize, Serialize};

use crate::feed::Feed;
use crate::js::{AsJson, Runtime};
use crate::util::{Error, Result};

use super::{FeedFilter, FeedFilterConfig};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct JsConfig {
  /// The javascript code to run
  code: String,
}

pub struct JsFilter {
  runtime: Runtime,
}

#[async_trait::async_trait]
impl FeedFilterConfig for JsConfig {
  type Filter = JsFilter;

  async fn build(&self) -> Result<Self::Filter> {
    let runtime = Runtime::new().await?;
    runtime.eval(&self.code).await?;

    Ok(Self::Filter { runtime })
  }
}

impl JsFilter {
  async fn modify_feed(&self, feed: &mut Feed) -> Result<()> {
    use either::Either::{Left, Right};
    use rquickjs::Undefined;

    if !self.runtime.fn_exists("update_feed").await {
      return Ok(());
    }

    let args = (AsJson(&*feed),);

    match self.runtime.call_fn("update_feed", args).await? {
      Left(Undefined) => {
        return Err(Error::Message(
          "update_feed must return the modified feed".into(),
        ));
      }
      Right(AsJson(updated_feed)) => {
        *feed = updated_feed;
      }
    }

    Ok(())
  }

  async fn modify_posts(&self, feed: &mut Feed) -> Result<()> {
    use either::Either::{Left, Right};
    use rquickjs::{Null, Undefined};

    if !self.runtime.fn_exists("update_post").await {
      return Ok(());
    }

    let mut posts = Vec::new();

    for post in feed.take_posts() {
      let args = (AsJson(&*feed), AsJson(&post));

      match self.runtime.call_fn("update_post", args).await? {
        Left(Left(Null)) => {
          // returning null means the post should be removed
        }
        Left(Right(Undefined)) => {
          return Err(Error::Message(
            "update_post must return the modified post or null".into(),
          ));
        }
        Right(AsJson(updated_post)) => {
          posts.push(updated_post);
        }
      }
    }

    feed.set_posts(posts);
  }
}

#[async_trait::async_trait]
impl FeedFilter for JsFilter {
  async fn run(&self, feed: &mut Feed) -> Result<()> {
    self.modify_feed(feed).await;
    self.modify_posts(feed).await;
    Ok(())
  }
}
