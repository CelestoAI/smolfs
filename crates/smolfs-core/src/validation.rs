use crate::error::{Result, SmolFsError};

pub fn validate_volume_name(name: &str) -> Result<()> {
    let valid = !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));

    if valid {
        Ok(())
    } else {
        Err(SmolFsError::InvalidVolumeName {
            name: name.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::validate_volume_name;

    #[test]
    fn accepts_safe_names() {
        for name in ["demo", "agent_1", "agent.1", "agent-1", "A19"] {
            validate_volume_name(name).unwrap();
        }
    }

    #[test]
    fn rejects_unsafe_names() {
        for name in ["", "../x", "with space", "a/b"] {
            assert!(validate_volume_name(name).is_err());
        }
    }
}
