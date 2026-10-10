// Aliasi generiranih tipova API-ja; ručno se ništa ne opisuje.
import type { components } from "@hotelixsim/api-client";

type S = components["schemas"];

export type HotelBody = S["HotelBody"];
export type PublicHotelBody = S["PublicHotelBody"];
export type PrivateHotelBody = S["PrivateHotelBody"];
export type LastMonthBody = S["LastMonthBody"];
export type MonthScoreBody = S["MonthScoreBody"];
export type GameBody = S["GameBody"];
export type RoundBody = S["RoundBody"];
export type SituationsBody = S["SituationsBody"];
export type SituationBody = S["SituationBody"];
export type ConversationBody = S["ConversationBody"];
export type ResponseBody = S["ResponseBody"];
export type CriterionBody = S["CriterionBody"];
export type PlanBody = S["PlanBody"];
export type PlanLimitsBody = S["PlanLimitsBody"];
export type FeedItemBody = S["FeedItemBody"];
export type IndexEntry = S["IndexEntry"];
export type IndexCode = S["IndexCode"];
export type MonthCode = S["MonthCode"];
export type CauseCode = S["CauseCode"];
export type VarCode = S["VarCode"];
export type UnitCode = S["UnitCode"];
export type StrategyCode = S["StrategyCode"];
