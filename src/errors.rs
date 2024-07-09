#[derive(Debug)]
pub struct MyError {
    pub info: String,
}


impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MyError: {}", self.info)
    }
}

impl std::error::Error for MyError {
    fn description(&self) -> &str {
        &self.info
    }
}
