use std::collections::BTreeMap;
use std::sync::Mutex;

use anyhow::Context;
use async_trait::async_trait;

/// The bucket's inbound prefix. The real one is S3; tests use [`FakeStore`], never AWS.
#[async_trait]
pub trait MailStore: Send + Sync {
    async fn list(&self) -> anyhow::Result<Vec<String>>;
    async fn get(&self, key: &str) -> anyhow::Result<Vec<u8>>;
    async fn delete(&self, key: &str) -> anyhow::Result<()>;
}

pub struct S3Store {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: String,
}

impl S3Store {
    /// Credentials and region come from the standard AWS chain (env, profile), so the keys
    /// stay out of bridle's own files.
    pub async fn connect(bucket: &str, prefix: &str, region: Option<&str>) -> Self {
        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
        if let Some(r) = region {
            loader = loader.region(aws_config::Region::new(r.to_string()));
        }
        S3Store {
            client: aws_sdk_s3::Client::new(&loader.load().await),
            bucket: bucket.to_string(),
            prefix: prefix.to_string(),
        }
    }
}

#[async_trait]
impl MailStore for S3Store {
    async fn list(&self) -> anyhow::Result<Vec<String>> {
        let mut keys = Vec::new();
        let mut pages = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&self.prefix)
            .into_paginator()
            .send();
        while let Some(page) = pages.next().await {
            let page = page.context("listing the inbound prefix")?;
            keys.extend(
                page.contents()
                    .iter()
                    .filter_map(|o| o.key())
                    .map(str::to_string),
            );
        }
        // SES writes a setup-notification object under some receipt rules; a key that is
        // just the prefix (a folder marker) is not mail either.
        keys.retain(|k| k != &self.prefix && !k.ends_with("AMAZON_SES_SETUP_NOTIFICATION"));
        Ok(keys)
    }

    async fn get(&self, key: &str) -> anyhow::Result<Vec<u8>> {
        let out = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .with_context(|| format!("getting {key}"))?;
        Ok(out.body.collect().await?.to_vec())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .with_context(|| format!("deleting {key}"))?;
        Ok(())
    }
}

/// An in-memory bucket.
#[derive(Default)]
pub struct FakeStore {
    objects: Mutex<BTreeMap<String, Vec<u8>>>,
}

impl FakeStore {
    pub fn put(&self, key: &str, raw: impl Into<Vec<u8>>) {
        self.objects
            .lock()
            .expect("fake store lock")
            .insert(key.to_string(), raw.into());
    }

    pub fn keys(&self) -> Vec<String> {
        self.objects
            .lock()
            .expect("fake store lock")
            .keys()
            .cloned()
            .collect()
    }
}

#[async_trait]
impl MailStore for FakeStore {
    async fn list(&self) -> anyhow::Result<Vec<String>> {
        Ok(self.keys())
    }

    async fn get(&self, key: &str) -> anyhow::Result<Vec<u8>> {
        self.objects
            .lock()
            .expect("fake store lock")
            .get(key)
            .cloned()
            .with_context(|| format!("no object {key}"))
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.objects.lock().expect("fake store lock").remove(key);
        Ok(())
    }
}
