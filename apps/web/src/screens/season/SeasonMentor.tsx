// Sezonski mentor (AI ili rezerva iz brojki, KS-2): tekst ima četiri podnaslova u svojim retcima; ekran ih samo
// pokazuje kao podnaslove, tekst ne mijenja. Dok mentor piše, „Mentor piše…“; ako se ne javi, to se kaže.
import { t } from "@hotelixsim/i18n";
import { useEffect, useState } from "react";
import { poll } from "../../api/poll";
import { Avatar } from "../../avatar/Avatar";
import { MENTOR_LOOK } from "../../avatar/look";
import { Card } from "../../ui/primitives";

const PARTS: readonly string[] = [
  t("season.mentor.part1"),
  t("season.mentor.part2"),
  t("season.mentor.part3"),
  t("season.mentor.part4"),
];

function MentorText({ text }: { text: string }) {
  const lines = text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
  return (
    <div className="space-y-1">
      {lines.map((l, i) =>
        PARTS.includes(l) ? (
          // biome-ignore lint/suspicious/noArrayIndexKey: retci stalnog teksta, redoslijed se ne mijenja
          <h3 key={i} className="pt-2 font-bold first:pt-0">
            {l}
          </h3>
        ) : (
          // biome-ignore lint/suspicious/noArrayIndexKey: retci stalnog teksta, redoslijed se ne mijenja
          <p key={i}>{l}</p>
        ),
      )}
    </div>
  );
}

export function SeasonMentor({ text }: { text: string | null }) {
  // Isti odgovor poslužitelja ne pokreće ponovni prikaz, pa „nije se javio“ ima vlastiti sat (kao mentor kruga).
  const [gaveUp, setGaveUp] = useState(false);
  useEffect(() => {
    const id = setTimeout(() => setGaveUp(true), poll.maxFeedPolls * poll.feedMs);
    return () => clearTimeout(id);
  }, []);
  return (
    <Card tone="mint" className="flex items-start gap-3">
      <Avatar look={MENTOR_LOOK} size={56} />
      <div className="min-w-0 flex-1 space-y-1">
        <h2 className="font-bold text-accent-ink">{t("season.mentor")}</h2>
        {text !== null ? (
          <MentorText text={text} />
        ) : (
          <p role="status">{gaveUp ? t("month.mentorMissing") : t("month.mentorWaiting")}</p>
        )}
      </div>
    </Card>
  );
}
