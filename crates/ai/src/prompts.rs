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
    version: "lik.v1",
    text: include_str!("../../../prompts/lik.v1.md"),
};
pub const MENTOR: Prompt = Prompt {
    version: "mentor.v2",
    text: include_str!("../../../prompts/mentor.v2.md"),
};
