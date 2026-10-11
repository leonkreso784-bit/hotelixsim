// App.tsx — ulaz: gost bez igre vidi početak, inače okvir s ekranom po ruti.
import { t } from "@hotelixsim/i18n";
import type { ReactNode } from "react";
import { isNoGame, problemMessage } from "./api/errors";
import { useGame, useMe } from "./api/queries";
import type { HotelBody } from "./api/types";
import { isV2 } from "./game";
import { type Route, useRoute } from "./router";
import { AvatarGallery } from "./screens/AvatarGallery";
import { DepartmentsScreen } from "./screens/DepartmentsScreen";
import { FeedScreen } from "./screens/FeedScreen";
import { HotelScreen } from "./screens/HotelScreen";
import { IndicatorScreen } from "./screens/IndicatorScreen";
import { KpiScreen } from "./screens/KpiScreen";
import { MonthScreen } from "./screens/MonthScreen";
import { NotFound } from "./screens/NotFound";
import { PlanScreen } from "./screens/PlanScreen";
import { PlScreen } from "./screens/PlScreen";
import { SituationScreen } from "./screens/SituationScreen";
import { SituationsScreen } from "./screens/SituationsScreen";
import { StartScreen } from "./screens/StartScreen";
import { chipText, Layout } from "./ui/Layout";
import { Button, ErrorText, StatusText } from "./ui/primitives";

/** Ekrani s jednim stupcem dobivaju ograničenu širinu čitanja; hotel, krug i situacija sami slažu dva stupca,
 * a pokazatelji na računalu trebaju širinu za tablicu. */
function Reading({ children }: { children: ReactNode }) {
  return <div className="mx-auto max-w-2xl space-y-3">{children}</div>;
}

function Screen({ route, me }: { route: Route; me: HotelBody }) {
  switch (route.name) {
    case "hotel":
      return <HotelScreen me={me} />;
    case "situations":
      return (
        <Reading>
          <SituationsScreen me={me} />
        </Reading>
      );
    case "situation":
      return <SituationScreen key={route.code} me={me} code={route.code} />;
    case "plan":
      return (
        <Reading>
          <PlanScreen me={me} />
        </Reading>
      );
    case "round":
      return <MonthScreen key={route.code} me={me} code={route.code} />;
    case "indicator":
      return (
        <Reading>
          <IndicatorScreen key={route.code} me={me} code={route.code} highlight={route.highlight} />
        </Reading>
      );
    case "kpi":
      return <KpiScreen me={me} />;
    case "feed":
      return (
        <Reading>
          <FeedScreen me={me} />
        </Reading>
      );
    case "departments":
      return <DepartmentsScreen me={me} code={null} />;
    case "department":
      return <DepartmentsScreen me={me} code={route.code} />;
    case "pl":
      return <PlScreen me={me} code={route.code} />;
    case "avatars":
      return <AvatarGallery />;
    case "new":
      return <StartScreen inGame />;
    default:
      return (
        <Reading>
          <NotFound />
        </Reading>
      );
  }
}

function GameShell({ me }: { me: HotelBody }) {
  const route = useRoute();
  const game = useGame(me.game_id);
  return (
    <Layout
      chip={game.data ? chipText(game.data) : null}
      withDepartments={game.data ? isV2(game.data) : false}
    >
      <Screen route={route} me={me} />
    </Layout>
  );
}

export function App() {
  const me = useMe();
  if (me.isPending)
    return (
      <div className="mx-auto max-w-md p-6 md:max-w-xl">
        <StatusText>{t("status.loading")}</StatusText>
      </div>
    );
  if (me.isError) {
    if (isNoGame(me.error)) return <StartScreen />;
    return (
      <div className="mx-auto max-w-md space-y-3 p-6 md:max-w-xl">
        <ErrorText>{problemMessage(me.error)}</ErrorText>
        <Button variant="secondary" onClick={() => void me.refetch()}>
          {t("error.retry")}
        </Button>
      </div>
    );
  }
  return <GameShell me={me.data} />;
}
