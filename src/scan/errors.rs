#[derive(Debug)]
pub enum ScanError {
    UnrecognisedCharacter,
    UnknownVariableType,
    MissingStringDelimiter,
    InvalidNumber,
}
