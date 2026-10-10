import { t } from "@hotelixsim/i18n";
import { Avatar } from "../avatar/Avatar";
import { HAIR_STYLES, KNOWN_LOOKS, lookFromSeed, MENTOR_LOOK, PLAYER_LOOK } from "../avatar/look";
import { Card } from "../ui/primitives";

const GUESTS = [
  "Ana Kovač",
  "Ivan Perić",
  "Maja Jurić",
  "Luka Novak",
  "Iva Babić",
  "Tomislav Marić",
  "Lana Vuković",
  "Karlo Šimić",
  "Ema Knežević",
];
const ITEM = "flex flex-col items-center gap-1";

export function AvatarGallery() {
  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">{t("avatars.title")}</h1>
      <p className="text-muted">{t("avatars.lead")}</p>
      <Card>
        <h2 className="pb-2 font-bold">{t("avatars.known")}</h2>
        <ul className="grid grid-cols-3 gap-3 text-center text-sm">
          <li className={ITEM}>
            <Avatar look={PLAYER_LOOK} size={72} />
            {t("avatars.player")}
          </li>
          <li className={ITEM}>
            <Avatar look={MENTOR_LOOK} size={72} />
            {t("avatars.mentor")}
          </li>
          {Object.entries(KNOWN_LOOKS).map(([name, look]) => (
            <li key={name} className={ITEM}>
              <Avatar look={look} size={72} />
              {name}
            </li>
          ))}
        </ul>
      </Card>
      <Card>
        <h2 className="pb-2 font-bold">{t("avatars.random")}</h2>
        <ul className="grid grid-cols-3 gap-3 text-center text-xs">
          {GUESTS.map((name) => (
            <li key={name} className={ITEM}>
              <Avatar look={lookFromSeed(name)} size={64} />
              {name}
            </li>
          ))}
        </ul>
      </Card>
      <Card>
        <h2 className="pb-2 font-bold">{t("avatars.hair")}</h2>
        <ul className="grid grid-cols-4 gap-2 text-center text-xs">
          {HAIR_STYLES.map((hair) => (
            <li key={hair} className={ITEM}>
              <Avatar look={{ ...PLAYER_LOOK, hair }} size={56} />
              {hair}
            </li>
          ))}
        </ul>
      </Card>
    </>
  );
}
