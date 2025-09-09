use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitStatus {
    AddedStaged,      // A: file added to index
    ModifiedStaged,   // M: file modified in index
    DeletedStaged,    // D: file deleted from index
    AddedUnstaged,    // A: file added in working tree
    ModifiedUnstaged, // M: file modified in working tree
    DeletedUnstaged,  // D: file deleted in working tree
    Untracked,        // ??: untracked file
    Renamed,          // R: renamed
    Copied,           // C: copied
    Unmerged,         // U: unmerged
    Ignored,          // !: ignored
    Unknown(String),  // Unknown status code
}

impl FromStr for GitStatus {
    type Err = crate::error::GitChaiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A " => Ok(GitStatus::AddedStaged),
            "M " => Ok(GitStatus::ModifiedStaged),
            "D " => Ok(GitStatus::DeletedStaged),
            " A" => Ok(GitStatus::AddedUnstaged),
            " M" => Ok(GitStatus::ModifiedUnstaged),
            " D" => Ok(GitStatus::DeletedUnstaged),
            "??" => Ok(GitStatus::Untracked),
            "R " => Ok(GitStatus::Renamed),
            "C " => Ok(GitStatus::Copied),
            "U " => Ok(GitStatus::Unmerged),
            "! " => Ok(GitStatus::Ignored),
            _ => Ok(GitStatus::Unknown(s.to_string())),
        }
    }
}

impl fmt::Display for GitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitStatus::AddedStaged => write!(f, "A "),
            GitStatus::ModifiedStaged => write!(f, "M "),
            GitStatus::DeletedStaged => write!(f, "D "),
            GitStatus::AddedUnstaged => write!(f, " A"),
            GitStatus::ModifiedUnstaged => write!(f, " M"),
            GitStatus::DeletedUnstaged => write!(f, " D"),
            GitStatus::Untracked => write!(f, "??"),
            GitStatus::Renamed => write!(f, "R "),
            GitStatus::Copied => write!(f, "C "),
            GitStatus::Unmerged => write!(f, "U "),
            GitStatus::Ignored => write!(f, "! "),
            GitStatus::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Add,
    Modify,
    Delete,
    Rename,
    Copy,
}

impl From<GitStatus> for ChangeType {
    fn from(status: GitStatus) -> Self {
        match status {
            GitStatus::AddedStaged | GitStatus::AddedUnstaged | GitStatus::Untracked => {
                ChangeType::Add
            }
            GitStatus::ModifiedStaged | GitStatus::ModifiedUnstaged => ChangeType::Modify,
            GitStatus::DeletedStaged | GitStatus::DeletedUnstaged => ChangeType::Delete,
            GitStatus::Renamed => ChangeType::Rename,
            GitStatus::Copied => ChangeType::Copy,
            GitStatus::Unknown(_) | GitStatus::Unmerged | GitStatus::Ignored => ChangeType::Modify, // Default fallback
        }
    }
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeType::Add => write!(f, "add"),
            ChangeType::Modify => write!(f, "mod"),
            ChangeType::Delete => write!(f, "del"),
            ChangeType::Rename => write!(f, "rename"),
            ChangeType::Copy => write!(f, "copy"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_parsing() {
        assert_eq!(GitStatus::from_str("A ").unwrap(), GitStatus::AddedStaged);
        assert_eq!(
            GitStatus::from_str("M ").unwrap(),
            GitStatus::ModifiedStaged
        );
        assert_eq!(GitStatus::from_str("D ").unwrap(), GitStatus::DeletedStaged);
        assert_eq!(GitStatus::from_str(" A").unwrap(), GitStatus::AddedUnstaged);
        assert_eq!(
            GitStatus::from_str(" M").unwrap(),
            GitStatus::ModifiedUnstaged
        );
        assert_eq!(
            GitStatus::from_str(" D").unwrap(),
            GitStatus::DeletedUnstaged
        );
        assert_eq!(GitStatus::from_str("??").unwrap(), GitStatus::Untracked);
        assert_eq!(GitStatus::from_str("R ").unwrap(), GitStatus::Renamed);
        assert_eq!(GitStatus::from_str("C ").unwrap(), GitStatus::Copied);
        assert_eq!(GitStatus::from_str("U ").unwrap(), GitStatus::Unmerged);
        assert_eq!(GitStatus::from_str("! ").unwrap(), GitStatus::Ignored);

        // Test unknown status
        let unknown = GitStatus::from_str("X ").unwrap();
        match unknown {
            GitStatus::Unknown(s) => assert_eq!(s, "X "),
            _ => panic!("Expected Unknown variant"),
        }
    }

    #[test]
    fn test_git_status_display() {
        assert_eq!(GitStatus::AddedStaged.to_string(), "A ");
        assert_eq!(GitStatus::ModifiedStaged.to_string(), "M ");
        assert_eq!(GitStatus::DeletedStaged.to_string(), "D ");
        assert_eq!(GitStatus::AddedUnstaged.to_string(), " A");
        assert_eq!(GitStatus::ModifiedUnstaged.to_string(), " M");
        assert_eq!(GitStatus::DeletedUnstaged.to_string(), " D");
        assert_eq!(GitStatus::Untracked.to_string(), "??");
        assert_eq!(GitStatus::Renamed.to_string(), "R ");
        assert_eq!(GitStatus::Copied.to_string(), "C ");
        assert_eq!(GitStatus::Unmerged.to_string(), "U ");
        assert_eq!(GitStatus::Ignored.to_string(), "! ");
        assert_eq!(GitStatus::Unknown("X ".to_string()).to_string(), "X ");
    }

    #[test]
    fn test_change_type_from_git_status() {
        assert_eq!(ChangeType::from(GitStatus::AddedStaged), ChangeType::Add);
        assert_eq!(ChangeType::from(GitStatus::AddedUnstaged), ChangeType::Add);
        assert_eq!(ChangeType::from(GitStatus::Untracked), ChangeType::Add);
        assert_eq!(
            ChangeType::from(GitStatus::ModifiedStaged),
            ChangeType::Modify
        );
        assert_eq!(
            ChangeType::from(GitStatus::ModifiedUnstaged),
            ChangeType::Modify
        );
        assert_eq!(
            ChangeType::from(GitStatus::DeletedStaged),
            ChangeType::Delete
        );
        assert_eq!(
            ChangeType::from(GitStatus::DeletedUnstaged),
            ChangeType::Delete
        );
        assert_eq!(ChangeType::from(GitStatus::Renamed), ChangeType::Rename);
        assert_eq!(ChangeType::from(GitStatus::Copied), ChangeType::Copy);

        // Test fallbacks
        assert_eq!(ChangeType::from(GitStatus::Unmerged), ChangeType::Modify);
        assert_eq!(ChangeType::from(GitStatus::Ignored), ChangeType::Modify);
        assert_eq!(
            ChangeType::from(GitStatus::Unknown("".to_string())),
            ChangeType::Modify
        );
    }

    #[test]
    fn test_change_type_display() {
        assert_eq!(ChangeType::Add.to_string(), "add");
        assert_eq!(ChangeType::Modify.to_string(), "mod");
        assert_eq!(ChangeType::Delete.to_string(), "del");
        assert_eq!(ChangeType::Rename.to_string(), "rename");
        assert_eq!(ChangeType::Copy.to_string(), "copy");
    }
}
