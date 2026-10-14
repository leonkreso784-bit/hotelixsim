// Kartica odjela na popisu: ikona i ime (poveznica na cijelu karticu), ocjena odjela kao jedna traka (zatvoren odjel
// je nema, uz njega stoji značka „Zatvoren“), rezultat zadnjeg kruga, broj ljudi (najam bez ljudi ga nema) i obnova
// odlučena za ovaj mjesec.
import { t } from "@hotelixsim/i18n";
import type { DepartmentBody } from "../../api/types";
import { tw } from "../../format";
import { Link } from "../../router";
import { Badge } from "../../ui/primitives";
import { lockedPhrase } from "../renewal/renewal";
import { FOCUS, leaseWithoutPeople, peopleText, resultText } from "./helpers";
import { DeptIcon } from "./icons";
import { Meter } from "./Meter";
import { deptScore, scoreLevel } from "./score";

export function DepartmentCard({
  dept,
  weekly,
  selected,
  level = 2,
}: {
  dept: DepartmentBody;
  weekly: boolean;
  selected: boolean;
  /** Razina naslova kartice: 3 kad je popis ispod naslova razine 2. */
  level?: 2 | 3;
}) {
  const Heading = level === 2 ? "h2" : "h3";
  const score = deptScore(dept);
  return (
    <article
      className={`relative space-y-3 rounded-card border border-line bg-surface p-4 has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-accent ${selected ? "ring-2 ring-accent" : ""}`}
    >
      <div className="flex items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-2">
          <DeptIcon code={dept.code} />
          <Heading className="min-w-0 text-lg font-bold">
            {/* after:inset-0 razvlači poveznicu preko cijele kartice: jedna meta, jedan Link. */}
            <Link
              to={`/odjeli/${dept.code}`}
              aria-current={selected ? "page" : undefined}
              className={`flex min-h-11 items-center rounded-card after:absolute after:inset-0 after:z-10 after:rounded-card ${FOCUS}`}
            >
              {dept.name}
            </Link>
          </Heading>
        </div>
        {!dept.open && <Badge tone="muted">{t("dept.closed")}</Badge>}
      </div>
      {score && <Meter label={t("dept.score")} points={score.points} level={scoreLevel(score.points)} />}
      {dept.last && (
        <p className="flex items-baseline justify-between gap-3 text-sm">
          <span className="min-w-0">{tw(weekly, "dept.result")}</span>
          <strong className="whitespace-nowrap tabular-nums">{resultText(dept.last.profit_cents)}</strong>
        </p>
      )}
      {!leaseWithoutPeople(dept) && (
        <p className="text-sm text-muted">{peopleText(dept.permanent, dept.seasonal)}</p>
      )}
      {dept.renewal && (
        <p className="text-sm">{t("dept.card.renewal", { phrase: lockedPhrase(dept.renewal, weekly) })}</p>
      )}
    </article>
  );
}
