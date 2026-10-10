import { t } from "@hotelixsim/i18n";
import { ChevronLeft } from "lucide-react";
import { useEffect, useRef } from "react";
import { problemMessage } from "../api/errors";
import { useSituations } from "../api/queries";
import type { HotelBody } from "../api/types";
import { Link } from "../router";
import { ErrorText, StatusText } from "../ui/primitives";
import { Conversation } from "./situation/Conversation";
import { FreeResponse } from "./situation/FreeResponse";
import { ChosenAnswer, OfferedAnswers } from "./situation/OfferedAnswers";
import { Verdict } from "./situation/Verdict";

export function SituationScreen({ me, code }: { me: HotelBody; code: string }) {
  const hotelId = me.hotel.hotel_id;
  const sits = useSituations(hotelId);
  // Je li forma za odgovor bila prikazana: tek tada novi sadržaj (izbor, presuda) prima fokus.
  const answeredHere = useRef(false);
  const current = sits.data?.items.find((i) => i.code === code);
  const formShown =
    sits.data !== undefined &&
    current !== undefined &&
    sits.data.open &&
    current.chosen === null &&
    current.response === null;
  useEffect(() => {
    if (formShown) answeredHere.current = true;
  }, [formShown]);
  const back = (
    <Link to="/situacije" className="inline-flex min-h-11 items-center gap-1 font-semibold text-accent-ink">
      <ChevronLeft size={18} aria-hidden="true" />
      {t("situation.back")}
    </Link>
  );
  if (sits.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (sits.isError && !sits.data) return <ErrorText>{problemMessage(sits.error)}</ErrorText>;
  const s = sits.data.items.find((i) => i.code === code);
  if (!s)
    return (
      <>
        {back}
        <p>{t("situation.missing")}</p>
      </>
    );
  const open = sits.data.open;
  const decided = s.chosen !== null || s.response !== null;
  const answerArea = (
    <div className="space-y-3">
      {s.response && <Verdict response={s.response} focusOnMount={answeredHere.current} />}
      {open && !decided && s.free_text && <FreeResponse hotelId={hotelId} s={s} />}
      <ChosenAnswer s={s} focusOnMount={answeredHere.current} />
      {open && !decided && (
        <OfferedAnswers
          hotelId={hotelId}
          s={s}
          heading={s.free_text ? t("answers.orTitle") : t("answers.title")}
        />
      )}
      {!open && !decided && <p className="font-semibold">{t("answers.missed")}</p>}
    </div>
  );
  const title = <h1 className="text-3xl leading-tight font-extrabold tracking-tight">{s.title}</h1>;
  // Bez lika je ekran jedan stupac za čitanje: povratak, naslov i odgovori u istom omotaču (UI_PRAVILA §4.3).
  if (!s.character)
    return (
      <div data-layout="reading" className="mx-auto max-w-2xl space-y-3">
        {back}
        {title}
        {answerArea}
      </div>
    );
  return (
    <>
      {back}
      {title}
      <div className="space-y-3 lg:grid lg:grid-cols-2 lg:items-start lg:gap-6 lg:space-y-0">
        <div>
          <Conversation hotelId={hotelId} code={s.code} canWrite={open && !decided} />
        </div>
        {answerArea}
      </div>
    </>
  );
}
