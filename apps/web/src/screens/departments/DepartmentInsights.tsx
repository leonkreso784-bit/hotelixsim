// „Što mu fali“ i „U čemu je dobar“: rečenice iz zadnjeg obračunatog kruga (pravila u `insights.ts`). Prije prvog
// obračuna jedna rečenica kada će se vidjeti.
import { t } from "@hotelixsim/i18n";
import type { DepartmentBody } from "../../api/types";
import { tw } from "../../format";
import { Card } from "../../ui/primitives";
import { type Insight, insights } from "./insights";

function List({ items, empty }: { items: Insight[]; empty: string }) {
  if (items.length === 0) return <p className="text-muted">{empty}</p>;
  return (
    <ul className="space-y-2">
      {items.map((i) => (
        <li key={i.text}>
          <p>{i.text}</p>
          {i.suggestion && <p className="text-sm font-semibold">{i.suggestion}</p>}
        </li>
      ))}
    </ul>
  );
}

export function DepartmentInsights({ dept, weekly }: { dept: DepartmentBody; weekly: boolean }) {
  const r = dept.last;
  if (!r)
    return (
      <Card className="space-y-2">
        <p>{tw(weekly, "dept.noResult")}</p>
        <p>{tw(weekly, "dept.insights.empty")}</p>
      </Card>
    );
  const x = insights(dept, r, weekly);
  return (
    <div className="space-y-3 xl:grid xl:grid-cols-2 xl:items-start xl:gap-3 xl:space-y-0">
      <Card className="space-y-2">
        <h2 className="text-lg font-bold">{t("dept.weak")}</h2>
        <List items={x.weak} empty={tw(weekly, "dept.weak.none")} />
      </Card>
      <Card className="space-y-2">
        <h2 className="text-lg font-bold">{t("dept.strong")}</h2>
        <List items={x.strong} empty={tw(weekly, "dept.strong.none")} />
      </Card>
    </div>
  );
}
