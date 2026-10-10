// App.tsx — ulaz: gost bez igre vidi početak, inače okvir s ekranom po ruti.
import { t } from "@hotelixsim/i18n";
import { isNoGame, problemMessage } from "./api/errors";
import { useGame, useMe } from "./api/queries";
import type { HotelBody } from "./api/types";
import { type Route, useRoute } from "./router";
import { AvatarGallery } from "./screens/AvatarGallery";
import { HotelScreen } from "./screens/HotelScreen";
import { NotFound } from "./screens/NotFound";
import { StartScreen } from "./screens/StartScreen";
import { chipText, Layout } from "./ui/Layout";
import { Button, ErrorText, StatusText } from "./ui/primitives";

function Screen({ route, me }: { route: Route; me: HotelBody }) {
  switch (route.name) {
    case "hotel":
      return <HotelScreen me={me} />;
    case "avatars":
      return <AvatarGallery />;
    default:
      return <NotFound />;
  }
}

function GameShell({ me }: { me: HotelBody }) {
  const route = useRoute();
  const game = useGame(me.game_id);
  return (
    <Layout chip={game.data ? chipText(game.data) : null}>
      <Screen route={route} me={me} />
    </Layout>
  );
}

export function App() {
  const me = useMe();
  if (me.isPending)
    return (
      <div className="mx-auto max-w-md p-6">
        <StatusText>{t("status.loading")}</StatusText>
      </div>
    );
  if (me.isError) {
    if (isNoGame(me.error)) return <StartScreen />;
    return (
      <div className="mx-auto max-w-md space-y-3 p-6">
        <ErrorText>{problemMessage(me.error)}</ErrorText>
        <Button variant="secondary" onClick={() => void me.refetch()}>
          {t("error.retry")}
        </Button>
      </div>
    );
  }
  return <GameShell me={me.data} />;
}
