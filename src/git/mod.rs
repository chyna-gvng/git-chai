pub mod status;
pub mod commit;
pub mod grouping;
pub mod operations;



pub use status::get_changed_files;
pub use commit::{create_commit_for_file, create_commit_for_directory, push_changes};
pub use grouping::{group_changes_by_directory, ChangeGroup};
pub use operations::{stage_file, stage_directory};