use reqwest::blocking::{Client, Response};
use serde::Serialize;

pub const API_BASE: &'static str = "http://grillpm.vercel.app/api";

pub fn api<S, B>(sub: S, body: &B) -> anyhow::Result<Response>
where
    S: AsRef<str>,
    B: Serialize,
{
    Client::new()
        .post(format!("{}/{}", API_BASE, sub.as_ref()))
        .json(body)
        .send()
        .map_err(anyhow::Error::msg)
}
