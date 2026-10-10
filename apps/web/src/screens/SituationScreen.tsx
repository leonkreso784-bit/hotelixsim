import { t } from "@hotelixsim/i18n";
import { ChevronLeft } from "lucide-react";
import { problemMessage } from "../api/errors";
import { useSituations } from "../api/queries";
import type { HotelBody } from "../api/types";
import { Link } from "../router";
import { ErrorText, StatusText } from "../ui/primitives";
import { Conversation } from "./situation/Conversation";
import { ChosenAnswer, OfferedAnswers } from "./situation/OfferedAnswers";

export function SituationScreen({ me, code }: { me: HotelBody; code: string }) {
  const hotelId = me.hotel.hotel_id;
  const sits = useSituations(hotelId);
  const back = (
    <Link to="/situacije" className="inline-flex min-h-11 items-center gap-1 font-semibold text-accent">
      <ChevronLeft size={18} aria-hidden="true" />
      {t("situation.back")}
    </Link>
  );
  if (sits.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (sits.isError) return <ErrorText>{problemMessage(sits.error)}</ErrorText>;
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
  return (
    <>
      {back}
      <h1 className="text-3xl leading-tight font-extrabold tracking-tight">{s.title}</h1>
      {s.character && <Conversation hotelId={hotelId} code={s.code} canWrite={open && !decided} />}
      {/* Task 7: slobodan odgovor i presuda */}
      <ChosenAnswer s={s} />
      {open && !decided && (
        <OfferedAnswers
          hotelId={hotelId}
          s={s}
          heading={s.free_text ? t("answers.orTitle") : t("answers.title")}
        />
      )}
      {!open && !decided && <p className="font-semibold">{t("answers.missed")}</p>}
    </>
  );
}
