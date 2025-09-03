/// This enum represents whether a diffed result
/// has a significant, less significant, minor or no change
pub enum DiffType {
    MAJOR,
    MINOR,
    PATCH,
    IDENTICAL,
}
