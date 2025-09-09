pub mod commit;
pub mod grouping;
pub mod operations;
pub mod status;

pub use commit::{create_commit_for_directory, create_commit_for_file, push_changes};
pub use grouping::{ChangeGroup, group_changes_by_directory};
pub use operations::{stage_directory, stage_file};
pub use status::get_changed_files;
