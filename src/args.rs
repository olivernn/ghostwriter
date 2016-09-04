extern crate url;

use std::env;
use url::Url;

#[derive(Debug)]
pub struct Args {
    pub remote: String,
    pub url: Url,
}

impl Args {
    pub fn from_env(mut argv: env::Args) -> Result<Self, String> {
        let remote = try!(argv.nth(1).ok_or("remote arg is required".to_owned()));
        let url = try!(argv.next()
            .ok_or("remote url arg is required".to_owned())
            .and_then(|arg| Url::parse(&arg).map_err(|e| e.to_string()) ));

        return Ok(Args{remote: remote, url: url})
    }
}
