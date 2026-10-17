import { type MessageKey, t } from "@hotelixsim/i18n";
import { Avatar } from "../avatar/Avatar";
import {
  AGES,
  type AvatarLook,
  BROWS,
  EARRINGS,
  HAIR_STYLES,
  KNOWN_LOOKS,
  type LookHint,
  lookFor,
  lookFromSeed,
  MENTOR_LOOK,
  NECKWEAR,
  NOSES,
  OUTFITS,
  PLAYER_LOOK,
} from "../avatar/look";
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
/** Red „Službe“: odora iz podatka `uniform` (kao iz API-ja), lice iz imena; imena su izmišljena za galeriju. */
const SERVICES: readonly { label: MessageKey; name: string; hint: LookHint }[] = [
  {
    label: "avatars.service.police_m",
    name: "Marin Jelić",
    hint: { gender: "m", age_group: "adult", origin: "regional", uniform: "police" },
  },
  {
    label: "avatars.service.police_f",
    name: "Ivana Kos",
    hint: { gender: "f", age_group: "adult", origin: "regional", uniform: "police" },
  },
  {
    label: "avatars.service.fire",
    name: "Dario Rukavina",
    hint: { gender: "m", age_group: "adult", origin: "regional", uniform: "fire" },
  },
  {
    label: "avatars.service.ems",
    name: "Sanja Matić",
    hint: { gender: "f", age_group: "adult", origin: "regional", uniform: "ems" },
  },
  {
    label: "avatars.service.doctor_m",
    name: "Goran Lukić",
    hint: { gender: "m", age_group: "senior", origin: "regional", uniform: "doctor" },
  },
  {
    label: "avatars.service.doctor_f",
    name: "Petra Vidić",
    hint: { gender: "f", age_group: "adult", origin: "regional", uniform: "doctor" },
  },
];
const TRAIT_KEYS = ["brows", "nose", "age", "earrings", "neckwear", "outfit"] as const;
const ITEM = "flex flex-col items-center gap-1";

/** Podloga redova: bez kravate u redu odjeće i s običnom košuljom u redu vezova, da se vidi samo osobina u redu. */
const BASE: Record<(typeof TRAIT_KEYS)[number], AvatarLook> = {
  brows: PLAYER_LOOK,
  nose: PLAYER_LOOK,
  age: PLAYER_LOOK,
  earrings: PLAYER_LOOK,
  neckwear: { ...PLAYER_LOOK, outfit: "shirt" },
  outfit: { ...PLAYER_LOOK, neckwear: "none" },
};

/** Redovi novih osobina; naslov je sirovi id osobine (kao opcije frizura), bez prijevoda do uređivača. */
const TRAIT_ROWS: readonly {
  key: "brows" | "nose" | "age" | "earrings" | "neckwear" | "outfit";
  values: readonly string[];
}[] = [
  { key: "brows", values: BROWS },
  { key: "nose", values: NOSES },
  { key: "age", values: AGES },
  { key: "earrings", values: EARRINGS },
  { key: "neckwear", values: NECKWEAR },
  { key: "outfit", values: OUTFITS },
];

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
      <Card>
        <h2 className="pb-2 font-bold">{t("avatars.services")}</h2>
        <ul className="grid grid-cols-3 gap-3 text-center text-xs">
          {SERVICES.map(({ label, name, hint }) => (
            <li key={label} className={ITEM}>
              <span className="flex items-end gap-1">
                <Avatar look={lookFor(name, hint)} size={56} />
                <Avatar look={lookFor(name, hint)} size={36} />
              </span>
              {t(label)}
            </li>
          ))}
        </ul>
      </Card>
      {TRAIT_ROWS.map(({ key, values }) => (
        <Card key={key}>
          <h2 className="pb-2 font-bold">{key}</h2>
          <ul className="grid grid-cols-4 gap-2 text-center text-xs">
            {values.map((value) => (
              <li key={value} className={ITEM}>
                <Avatar look={{ ...BASE[key], [key]: value } as AvatarLook} size={56} />
                {value}
              </li>
            ))}
          </ul>
        </Card>
      ))}
    </>
  );
}
