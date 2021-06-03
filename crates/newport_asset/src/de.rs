use crate::{
    serde,
    UUID,
};

use std::str;

use serde::{
    ron,

    Serialize,
    Deserialize,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde", rename = "Asset")]
pub struct AssetFile<T> {
    pub id: UUID,
    #[serde(bound(deserialize = "T: Deserialize<'de>"))]
    pub asset: T,
}

pub fn deserialize<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<(UUID, T), ()> {
    let contents = str::from_utf8(bytes).map_err(|_| ())?;

    let t: AssetFile<T> = ron::from_str(contents).map_err(|_| ())?;
    Ok((t.id, t.asset))
}