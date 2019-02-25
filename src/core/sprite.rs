#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssetId {
    Grunt = 0,
    Missile = 1,
    SmallMissile = 2,
    Tower = 3,
    Explosion = 4,
}
pub fn assetid_as_u8(asset_id: AssetId) -> u8 {
    match asset_id {
        AssetId::Grunt => 0,
        AssetId::Missile => 1,
        AssetId::SmallMissile => 2,
        AssetId::Tower => 3,
        AssetId::Explosion => 4,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    pub asset_id: AssetId,
}
