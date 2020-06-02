use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Behaviour {
    Honest,
    Malicious,
}

impl fmt::Display for Behaviour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Behaviour::Honest => "honest",
                Behaviour::Malicious => "malicious",
            }
        )
    }
}
