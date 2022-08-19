use nom::{
    IResult,
    sequence::delimited,
    character::complete::char,
    bytes::complete::is_not,
};

#[allow(dead_code)]
fn parens(input: &str) -> IResult<&str, &str> {
    delimited(char('('),
              is_not(")"),
              char(')'))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parens() {
        let (x1, x2) = parens("(hello, world)").unwrap();

        println!("x1: {}, x2: {}", x1, x2)
    }
}