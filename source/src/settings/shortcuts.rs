//! Persist shortcuts as VK in the low byte and the Windows modifier mask above it.
//! Existing F8/F9/F10 values remain compatible with settings v3.
use crate::core::AppResult;

pub const DEFAULTS: [u32; 4] = [0x77, 0x78, 0x79, 0x2c];
pub fn encode(key: u32, modifiers: u32) -> Option<u32> {
    if !(8..=254).contains(&key)
        || [
            0x10, 0x11, 0x12, 0x1b, 0x5b, 0x5c, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5,
        ]
        .contains(&key)
    {
        return None;
    }
    Some(key | ((modifiers & 15) << 8))
}
pub fn valid(value: u32) -> bool {
    encode(value & 255, value >> 8) == Some(value)
}
pub fn validate(keys: &[u32; 4]) -> AppResult<()> {
    for (i, key) in keys.iter().enumerate() {
        if !valid(*key) || keys[..i].contains(key) {
            return Err(crate::i18n::tr(
                "Atalhos duplicados ou inválidos. Pressione outra tecla ou combinação.",
            )
            .into());
        }
    }
    Ok(())
}
pub fn label(value: u32) -> String {
    let mut parts = vec![];
    for (mask, name) in [(2, "Ctrl"), (1, "Alt"), (4, "Shift"), (8, "Win")] {
        if value >> 8 & mask != 0 {
            parts.push(name.to_owned());
        }
    }
    parts.push(crate::platform::windows::files::key_name(value & 255));
    parts.join("+")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_defaults_arbitrary_keys_and_conflicts() {
        validate(&DEFAULTS).unwrap();
        assert_eq!(encode(0x41, 6), Some(0x641));
        assert!(valid(0x641));
        for value in [0, 0x1b, 0x10, 0x10000, 0x1111] {
            assert!(!valid(value));
        }
        assert!(validate(&[0x41, 0x241, 0x441, 0x2c]).is_ok());
        assert!(validate(&[0x77, 0x77, 0x79, 0x2c]).is_err());
    }
}
