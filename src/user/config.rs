pub struct Config<'r> {
    pub db_path: &'r std::path::PathBuf,
    pub meta_path: &'r std::path::PathBuf,
}
