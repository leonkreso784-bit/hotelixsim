//! Tijela ekrana situacija: popis situacija kruga s odjelom, vrstom, važnošću, opisom, ulogama, obveznima i
//! prepuštenima (A-1), te slobodan odgovor i njegova presuda, i razgovor sa likom. Izdvojeno iz `schema.rs` (granica veličine datoteke);
//! `schema` ih ponovno izvozi, pa su putanje i imena u OpenAPI-ju ista.

use serde::Serialize;
use utoipa::ToSchema;

use crate::problem::Problem;
use crate::schema::{MonthCode, calendar_month};

/// Ponuđeni odgovor je samo oznaka; kvaliteta i mehanika nikad ne izlaze.
#[derive(Serialize, ToSchema)]
pub struct SituationBody {
    pub code: String,
    pub title: String,
    /// Oznake ponuđenih odgovora; indeks u popisu je broj odgovora.
    pub answers: Vec<String>,
    #[schema(required = true)]
    pub chosen: Option<u8>,
    /// Student smije odgovoriti svojim riječima i razgovarati s likom.
    pub free_text: bool,
    #[schema(required = true)]
    pub character: Option<CharacterBody>,
    #[schema(required = true)]
    pub response: Option<ResponseBody>,
    /// Kod odjela izvučenog primjerka (`restaurant`); `null` za situaciju cijelog hotela i sidra S1–S4.
    #[schema(required = true)]
    pub department: Option<String>,
    /// Vrsta primjerka; `null` za sidra S1–S4.
    #[schema(required = true)]
    pub kind: Option<SituationKindBody>,
    /// Važnost primjerka 1–5 (popis je po važnosti silazno, sidra prva); `null` za sidra.
    #[schema(required = true)]
    pub weight: Option<u8>,
    /// Opis situacije; `null` za sidra.
    #[schema(required = true)]
    pub description: Option<String>,
    /// Uloge primjerka; lik je među njima.
    pub roles: Vec<RoleBody>,
    /// Student je mora riješiti sam (prve tri situacije kruga).
    pub mandatory: bool,
    /// Prepuštena voditelju odjela.
    pub delegated: bool,
    /// Kod odjela voditelja kojem se situacija smije prepustiti; `null` kad ga nema (stari model).
    #[schema(required = true)]
    pub leader: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CharacterBody {
    pub name: String,
    pub role: String,
    /// Rod, dob i podrijetlo za izgled lika; `null` za sidra S1–S4 i kad ime nije u popisu imena.
    #[schema(required = true)]
    pub look: Option<CharacterLookBody>,
}

/// Izgled lika primjerka: web po njemu crta lice (rod: brada, frizura, naušnice; dob; raspon boje kože).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
pub struct CharacterLookBody {
    pub gender: GenderBody,
    /// `null` kad dob nije poznata (web je bira iz imena).
    #[schema(required = true)]
    pub age_group: Option<AgeGroupBody>,
    pub origin: OriginBody,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GenderBody {
    F,
    M,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgeGroupBody {
    Young,
    Adult,
    Senior,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OriginBody {
    Regional,
    Foreign,
}

impl From<app::CharacterLook> for CharacterLookBody {
    fn from(l: app::CharacterLook) -> Self {
        CharacterLookBody {
            gender: match l.gender {
                app::LookGender::F => GenderBody::F,
                app::LookGender::M => GenderBody::M,
            },
            age_group: l.age_group.map(|a| match a {
                app::AgeGroup::Young => AgeGroupBody::Young,
                app::AgeGroup::Adult => AgeGroupBody::Adult,
                app::AgeGroup::Senior => AgeGroupBody::Senior,
            }),
            origin: match l.origin {
                app::Origin::Regional => OriginBody::Regional,
                app::Origin::Foreign => OriginBody::Foreign,
            },
        }
    }
}

/// Vrsta situacije iz predloška.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SituationKindBody {
    Complaint,
    Staff,
    Opportunity,
    Breakdown,
    Crisis,
    Request,
}

impl From<game::SituationKind> for SituationKindBody {
    fn from(k: game::SituationKind) -> Self {
        use game::SituationKind as K;
        match k {
            K::Complaint => Self::Complaint,
            K::Staff => Self::Staff,
            K::Opportunity => Self::Opportunity,
            K::Breakdown => Self::Breakdown,
            K::Crisis => Self::Crisis,
            K::Request => Self::Request,
        }
    }
}

/// Uloga u situaciji: ključ iz predloška, ime osobe i rečenica tko je (ako je poznata).
#[derive(Serialize, ToSchema)]
pub struct RoleBody {
    pub key: String,
    pub name: String,
    #[schema(required = true)]
    pub role: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatusBody {
    Pending,
    Judged,
    Neutral,
}

impl From<app::ResponseStatus> for ResponseStatusBody {
    fn from(s: app::ResponseStatus) -> Self {
        match s {
            app::ResponseStatus::Pending => Self::Pending,
            app::ResponseStatus::Judged => Self::Judged,
            app::ResponseStatus::Neutral => Self::Neutral,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct CriterionBody {
    pub n: u8,
    pub text: String,
    pub met: bool,
    pub reason: String,
}

/// Slobodan odgovor studenta i njegova presuda (ponuđeni odgovori nemaju ovo).
#[derive(Serialize, ToSchema)]
pub struct ResponseBody {
    pub text: String,
    pub status: ResponseStatusBody,
    #[schema(required = true)]
    pub quality: Option<u8>,
    pub criteria: Vec<CriterionBody>,
    #[schema(required = true)]
    pub rationale: Option<String>,
}

impl From<&app::ResponseView> for ResponseBody {
    fn from(r: &app::ResponseView) -> Self {
        ResponseBody {
            text: r.text.clone(),
            status: r.status.into(),
            quality: r.quality,
            criteria: r
                .criteria
                .iter()
                .map(|c| CriterionBody {
                    n: c.n,
                    text: c.text.clone(),
                    met: c.met,
                    reason: c.reason.clone(),
                })
                .collect(),
            rationale: r.rationale.clone(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct SituationsBody {
    #[schema(required = true)]
    pub month: Option<MonthCode>,
    #[schema(required = true)]
    pub round: Option<String>,
    /// Može li se još odgovarati.
    pub open: bool,
    /// Koliko ih je bez odgovora (0 kad runda nije otvorena); ekran upozori prije zatvaranja mjeseca.
    pub unanswered: u8,
    pub items: Vec<SituationBody>,
}

impl TryFrom<&app::SituationsView> for SituationsBody {
    type Error = Problem;

    fn try_from(v: &app::SituationsView) -> Result<Self, Problem> {
        Ok(SituationsBody {
            month: v.round.map(calendar_month).transpose()?,
            round: v.round.map(|r| r.code()),
            open: v.open,
            unanswered: v.unanswered,
            items: v
                .items
                .iter()
                .map(|i| SituationBody {
                    code: i.code.clone(),
                    title: i.title.clone(),
                    answers: i.answers.clone(),
                    chosen: i.chosen,
                    free_text: i.free_text,
                    character: i.character.as_ref().map(|c| CharacterBody {
                        name: c.name.clone(),
                        role: c.role.clone(),
                        look: c.look.map(CharacterLookBody::from),
                    }),
                    response: i.response.as_ref().map(ResponseBody::from),
                    department: i.department.clone(),
                    kind: i.kind.map(SituationKindBody::from),
                    weight: i.weight,
                    description: i.description.clone(),
                    roles: i
                        .roles
                        .iter()
                        .map(|r| RoleBody {
                            key: r.key.clone(),
                            name: r.name.clone(),
                            role: r.role.clone(),
                        })
                        .collect(),
                    mandatory: i.mandatory,
                    delegated: i.delegated,
                    leader: i.leader.clone(),
                })
                .collect(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerBody {
    Student,
    Character,
}

#[derive(Serialize, ToSchema)]
pub struct ChatLineBody {
    pub from: SpeakerBody,
    pub text: String,
}

#[derive(Serialize, ToSchema)]
pub struct ConversationBody {
    pub situation: String,
    pub character_name: String,
    pub role: String,
    /// Isti izgled kao u popisu situacija.
    #[schema(required = true)]
    pub character_look: Option<CharacterLookBody>,
    pub opening: String,
    pub lines: Vec<ChatLineBody>,
    /// Koliko poruka student još smije poslati.
    pub remaining: u8,
    /// Lik još nije odgovorio na zadnju poruku.
    pub awaiting_reply: bool,
    /// Runda je otvorena; inače je razgovor samo za čitanje.
    pub open: bool,
}

impl From<&app::ConversationView> for ConversationBody {
    fn from(v: &app::ConversationView) -> Self {
        ConversationBody {
            situation: v.situation.clone(),
            character_name: v.character_name.clone(),
            role: v.role.clone(),
            character_look: v.character_look.map(CharacterLookBody::from),
            opening: v.opening.clone(),
            lines: v
                .lines
                .iter()
                .map(|l| ChatLineBody {
                    from: match l.from {
                        game::Speaker::Student => SpeakerBody::Student,
                        game::Speaker::Character => SpeakerBody::Character,
                    },
                    text: l.text.clone(),
                })
                .collect(),
            remaining: v.remaining,
            awaiting_reply: v.awaiting_reply,
            open: v.open,
        }
    }
}
