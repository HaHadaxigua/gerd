#[derive(Debug, PartialEq)]
pub enum RoxyErr {
    CharNotFound,
    CrossBorder,
    UnexpectedCharacter,
    LoadSubString,

    Utf8Error(std::str::Utf8Error),
}