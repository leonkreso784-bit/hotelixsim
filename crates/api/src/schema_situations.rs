//! Tijela ekrana situacija: popis situacija kruga s odjelom, vrstom, važnošću, opisom, ulogama, obveznima i
//! prepuštenima (A-1), te slobodan odgovor i njegova presuda. Izdvojeno iz `schema.rs` (granica veličine datoteke);
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
