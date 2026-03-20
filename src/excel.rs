use anyhow::Result;
use calamine::{RangeDeserializerBuilder, Reader, Xlsx, open_workbook};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::path::PathBuf;

use crate::parse_go_map::parse_go_map;

#[derive(Debug, Deserialize)]
pub struct Row {
    pub image_name: String,
    pub url: String,
    pub param: Option<String>,
    #[serde(default, deserialize_with = "deserialize_json_or_go_string")]
    pub args: Option<Value>,
    pub response: Option<String>,
}

fn deserialize_json_or_go_string<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;

    let Some(raw) = s else {
        return Ok(None);
    };

    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }

    // try json first
    if let Ok(v) = serde_json::from_str::<Value>(raw) {
        return Ok(Some(v));
    }

    // try go map
    parse_go_map(raw)
        .map(Some)
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse as JSON or Go map: {e}")))
}

pub fn read_excel(path: &PathBuf) -> Result<Vec<Row>> {
    let mut excel: Xlsx<_> = open_workbook(path)?;

    let range = excel.worksheet_range("Sheet1")?;

    let deserializer = RangeDeserializerBuilder::new().from_range(&range)?;

    let rows: Vec<Row> = deserializer
        .map(|res| {
            let row: Row = res?;
            Ok(row)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(rows)
}
