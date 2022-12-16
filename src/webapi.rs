use anyhow::Result;
use reqwest::{
    blocking::{Client, Response},
    header::{AUTHORIZATION, USER_AGENT},
};
use serde::Serialize;

const GRILL_API: &str = "http://grillpm.vercel.app/api";
const GITHUB_API: &str = "http://api.github.com";

pub fn grill<S, B>(sub: S, body: &B) -> Result<Response>
where
    S: AsRef<str>,
    B: Serialize,
{
    Client::new()
        .post(format!("{}{}", GRILL_API, sub.as_ref()))
        .json(body)
        .send()
        .map_err(anyhow::Error::msg)
}

pub fn github<S, B>(sub: S, body: &B) -> Result<Response>
where
    S: AsRef<str>,
    B: Serialize,
{
    println!("{}", format!("{}{}", GITHUB_API, sub.as_ref()));
    Client::new()
        .post(format!("{}{}", GITHUB_API, sub.as_ref()))
        .json(body)
        .header(USER_AGENT, "grill-cli")
        .send()
        .map_err(anyhow::Error::msg)
}
