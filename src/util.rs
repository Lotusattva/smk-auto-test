use anyhow::Result;
use base64::prelude::*;
use std::{fs, path::PathBuf};

pub fn to_base64(img_path: &PathBuf) -> Result<String> {
    let bytes = fs::read(img_path)?;
    Ok(BASE64_STANDARD.encode(bytes))
}
