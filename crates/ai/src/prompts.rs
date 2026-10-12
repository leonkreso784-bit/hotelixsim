//! Upute ugrađene pri prevođenju; verzija = ime datoteke bez `.md` (zapisuje se uz svaki AI poziv).

pub struct Prompt {
    pub version: &'static str,
    pub text: &'static str,
}

pub const JUDGE: Prompt = Prompt {
    version: "presuda.v3",
    text: include_str!("../../../prompts/presuda.v3.md"),
};
pub const CHARACTER: Prompt = Prompt {
    version: "lik.v2",
    text: include_str!("../../../prompts/lik.v2.md"),
};
pub const MENTOR: Prompt = Prompt {
    version: "mentor.v4",
    text: include_str!("../../../prompts/mentor.v4.md"),
};
pub const MENTOR_SEASON: Prompt = Prompt {
    version: "mentor-sezona.v1",
    text: include_str!("../../../prompts/mentor-sezona.v1.md"),
};
