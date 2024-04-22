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


#[derive(Debug, Clone)]
pub(crate) struct ArgParseError(pub String);
impl std::fmt::Display for ArgParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed to parse argument: {}", self.0)
    }
}
impl std::error::Error for ArgParseError {}
macro_rules! ebargs {
    ($s:expr) => {
        Err(Box::new(ArgParseError($s.to_string())))
    }
}
macro_rules! ebargsnb {
    ($s:expr) => {
        Err(ArgParseError($s.to_string()))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NoDefaultInterface;
impl std::fmt::Display for NoDefaultInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "OS has no default routing interface")
    }
}
impl std::error::Error for NoDefaultInterface {}
macro_rules! enodei {
    () => {
        Err(Box::new(NoDefaultInterface))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NonexistentInterface(pub String);
impl std::fmt::Display for NonexistentInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Interface does not exist: {}", &self.0)
    }
}
impl std::error::Error for NonexistentInterface {}
macro_rules! enonei {
    ($s:expr) => {
        Err(Box::new(NonexistentInterface($s.to_string())))
    }
}
