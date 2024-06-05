use serde::Serialize;
use std::num::NonZeroU32;

/// An API post object
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Post {
    /// The post id
    pub id: Box<str>,

    /// The post title
    pub title: Box<str>,

    /// The post author's username
    pub username: Box<str>,

    /// The privacy of the post
    pub privacy: Box<str>,

    /// ?
    pub report_status: i32,

    /// The number of views
    pub views: u64,

    /// ?
    pub nsfw: i32,

    /// The number of images
    pub image_count: u64,

    /// The time this was created
    pub created: Box<str>,

    /// The images of this post
    pub images: Box<[Image]>,

    /// The url to delete this post
    ///
    /// Only present if the current user owns this post.
    pub delete_url: Option<Box<str>>,
    // #[serde(flatten)]
    // extra: std::collections::HashMap<Box<str>, serde_json::Value>,
}

/// An API image of a post
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Image {
    /// The id of the image
    pub id: Box<str>,

    /// The image description
    #[serde(
        deserialize_with = "de_empty_string_is_none",
        serialize_with = "ser_empty_string_is_none"
    )]
    pub description: Option<Box<str>>,

    /// The link to the image file
    pub link: Box<str>,

    /// The position of the image in the post.
    ///
    /// Starts at 1.
    pub position: NonZeroU32,

    /// The time this image was created
    pub created: Box<str>,

    /// The original name of the image.
    ///
    /// Only present if the current user owns this image.
    pub original_name: Option<Box<str>>,
    // #[serde(flatten)]
    // extra: std::collections::HashMap<Box<str>, serde_json::Value>,
}

fn de_empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<Box<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Box<str> = serde::Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

fn ser_empty_string_is_none<S>(option: &Option<Box<str>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(value) = option {
        value.as_ref().serialize(serializer)
    } else {
        "".serialize(serializer)
    }
}
