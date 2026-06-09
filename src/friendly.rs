const ADJECTIVES: &[&str] = &[
    "bold", "calm", "clever", "daring", "eager", "fierce", "gentle", "happy",
    "jolly", "keen", "lively", "merry", "nimble", "proud", "quick", "ready",
    "swift", "tough", "vivid", "witty", "zany", "able", "bright", "cool",
    "deft", "elite", "fine", "grand", "holy", "ideal", "jovial", "kind",
    "loyal", "magic", "noble", "optimal", "prime", "quiet", "robust", "solid",
    "true", "unique", "valid", "wise", "xenial", "youthful", "zealous",
];

const NOUNS: &[&str] = &[
    "fox", "owl", "eagle", "wolf", "bear", "lion", "tiger", "hawk",
    "shark", "whale", "dolphin", "otter", "panda", "koala", "penguin", "seal",
    "moose", "deer", "horse", "zebra", "giraffe", "rhino", "hippo", "croc",
    "python", "cobra", "viper", "mamba", "gecko", "iguana", "chameleon", "tortoise",
    "falcon", "raven", "swan", "heron", "crane", "stork", "ibis", "flamingo",
];

fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

pub fn to_friendly(session_id: &str) -> String {
    let hash = simple_hash(session_id);
    let adj_idx = (hash as usize) % ADJECTIVES.len();
    let noun_idx = ((hash >> 32) as usize) % NOUNS.len();
    format!("{}-{}", ADJECTIVES[adj_idx], NOUNS[noun_idx])
}

pub fn is_friendly(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return false;
    }
    parts[0].chars().all(|c| c.is_ascii_lowercase())
        && parts[1].chars().all(|c| c.is_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        assert_eq!(to_friendly("deepwiki-session-0"), to_friendly("deepwiki-session-0"));
        assert_eq!(to_friendly("test"), to_friendly("test"));
    }

    #[test]
    fn test_different_inputs_different_outputs() {
        let a = to_friendly("session-1");
        let b = to_friendly("session-2");
        assert_ne!(a, b);
    }

    #[test]
    fn test_format() {
        let name = to_friendly("anything");
        assert!(name.contains('-'));
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert!(ADJECTIVES.contains(&parts[0]));
        assert!(NOUNS.contains(&parts[1]));
    }

    #[test]
    fn test_is_friendly_valid() {
        assert!(is_friendly("bold-fox"));
        assert!(is_friendly("clever-owl"));
        assert!(is_friendly("zany-zebra"));
    }

    #[test]
    fn test_is_friendly_invalid() {
        assert!(!is_friendly(""));
        assert!(!is_friendly("boldfox"));
        assert!(!is_friendly("Bold-fox"));
        assert!(!is_friendly("bold-Fox"));
        assert!(!is_friendly("bold-fox-extra"));
        assert!(!is_friendly("123-abc"));
    }
}