pub type Error = Box<dyn std::error::Error>;
pub type Result = std::result::Result<(), Error>;

mod expand_path;
mod parse;
