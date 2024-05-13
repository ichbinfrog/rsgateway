

#[derive(Debug)]
pub enum Authorization {
    Basic { user: String, password: String },
}

// impl FromStr for Authorization {
//     type Err = ParseError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.split_once(' ') {
//             Some((scheme, rest)) => {
//                 match scheme.to_lowercase().as_str() {
//                     "basic" => {
//                         let decoded = base64:: rest.trim()
//                     }
//                     _ => {}
//                 }
//             }
//             _ => {}
//         }

//         Ok(Authorization::Basic { user: "".to_string(), password: "".to_string() })
//     }
// }
