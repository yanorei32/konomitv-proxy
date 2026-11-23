use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn sign(secret: &str, channel: &str, resolution: &str, unixtime: u64) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can be initialized");

    mac.update(channel.as_bytes());

    mac.update(resolution.as_bytes());
    mac.update(&unixtime.to_le_bytes());

    let result = mac.finalize();

    hex::encode(result.into_bytes())
}

pub fn verify(
    secret: &str,
    channel: &str,
    resolution: &str,
    unixtime: u64,
    signature: &str,
) -> bool {
    sign(secret, channel, resolution, unixtime) == signature
}
