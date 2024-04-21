#[derive(Debug, Clone)]
pub(crate) struct NoConfigFile;
impl std::fmt::Display for NoConfigFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No config file detected!")
    }
}
impl std::error::Error for NoConfigFile {}

macro_rules! enocnf {
    () => {
        Err(Box::new(NoConfigFile))
    };
}
