use json_derive::Serialize;
use json::serialize;
use json::error::SerializeError;
use std::io::Write;

#[derive(Serialize)]
struct Test {
    name: String,
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let t = Test{
            id: "quetal".to_string(),
            name: "holla".to_string(),
        };
    }
}
