// Odore službi vanjskih likova (kartica IZGLED-c §5): ravne boje, tanki obrubi, bez teksta. Mijenjaju samo odjeću, kapu
// i ovratnik — lice ostaje isto. Nikad crveni križ (zaštićen znak): hitna nosi zvijezdu života, liječnik stetoskop.
import type { Hat, Outfit } from "./look";

const GOLD = "#E0A82E";
const SILVER = "#C9CED6";
const SHIRT_BLUE = "#BFD9F2";
const NAVY_DARK = "#121C30";
const STAR_BLUE = "#1565C0";
const GREY = "#6B7280";

/** Vrhovi zvijezde s `n` krakova (vanjski polumjer `R`, unutarnji `r`), počinje prema gore. */
function star(cx: number, cy: number, R: number, r: number, n: number): string {
  return Array.from({ length: n * 2 }, (_, i) => {
    const a = (Math.PI * i) / n - Math.PI / 2;
    const d = i % 2 === 0 ? R : r;
    return `${(cx + d * Math.cos(a)).toFixed(2)},${(cy + d * Math.sin(a)).toFixed(2)}`;
  }).join(" ");
}

export type ServiceOutfitKind = Extract<Outfit, "police" | "firefighter" | "paramedic" | "coat">;

export function ServiceOutfit({ outfit }: { outfit: ServiceOutfitKind }) {
  switch (outfit) {
    case "police":
      return (
        <g data-part="outfit-police">
          <path d="M27.3 45.2 L32 51 L36.7 45.2 Z" fill={SHIRT_BLUE} />
          <path d="M26.8 45 L29.6 50.4 L31.6 48.2 Z M37.2 45 L34.4 50.4 L32.4 48.2 Z" fill="#DCEBFA" />
          <path
            d="M12.5 52.5 L19.5 47.6 L20.6 49.6 L13.6 54.6 Z M51.5 52.5 L44.5 47.6 L43.4 49.6 L50.4 54.6 Z"
            fill={NAVY_DARK}
          />
          <polygon points={star(41, 54.5, 2.6, 1.1, 5)} fill={GOLD} />
        </g>
      );
    case "firefighter":
      return (
        <g data-part="outfit-firefighter">
          <rect x={11} y={53} width={42} height={2.4} rx={0.6} fill="#F4D03F" />
          <rect x={10.8} y={55.4} width={42.4} height={1.2} fill={SILVER} />
          <rect x={10.4} y={56.6} width={43.2} height={2.4} rx={0.6} fill="#F4D03F" />
        </g>
      );
    case "paramedic":
      return (
        <g data-part="outfit-paramedic">
          <rect x={8} y={57.5} width={48} height={3} fill={SILVER} />
          <circle cx={41} cy={51.5} r={3.6} fill="#fff" />
          <g fill={STAR_BLUE}>
            {[0, 60, 120].map((deg) => (
              <rect
                key={deg}
                x={40.2}
                y={48.6}
                width={1.6}
                height={5.8}
                transform={`rotate(${deg} 41 51.5)`}
              />
            ))}
          </g>
        </g>
      );
    case "coat":
      return (
        <g data-part="outfit-coat">
          <path d="M27.3 45.2 L32 55 L36.7 45.2 Z" fill={SHIRT_BLUE} />
          <path
            d="M23.5 46.6 L32 57 L40.5 46.6 M32 57 L32 64"
            stroke="#9AA5B1"
            strokeWidth={1.2}
            fill="none"
          />
          <rect
            x={38}
            y={54}
            width={6}
            height={4.6}
            rx={0.6}
            fill="none"
            stroke="#9AA5B1"
            strokeWidth={0.8}
          />
          <path d="M40 52.6 L40 55.6" stroke={STAR_BLUE} strokeWidth={1.2} strokeLinecap="round" />
        </g>
      );
  }
}

/** Stetoskop oko vrata: siva cijev u obliku U i srebrni kružić na prsima. */
export function Stethoscope() {
  return (
    <g data-part="neckwear-stethoscope">
      <path d="M26.5 45 Q25 56 32 57 Q39 56 37.5 45" stroke={GREY} strokeWidth={1.3} fill="none" />
      <path d="M32 57 L32 58" stroke={GREY} strokeWidth={1.3} />
      <circle cx={32} cy={59.6} r={2} fill={SILVER} stroke={GREY} strokeWidth={0.8} />
    </g>
  );
}

/** Šapka policije ili kaciga vatrogasca; obje pokrivaju kosu iznad čela. */
export function ServiceHat({ hat }: { hat: Extract<Hat, "police" | "fire"> }) {
  if (hat === "police")
    return (
      <g data-part="hat-police">
        <path d="M15 16 C15 7.5 49 7.5 49 16 L45 21 L19 21 Z" fill="#1F2F4D" />
        <rect x={19} y={18.4} width={26} height={3.2} fill={NAVY_DARK} />
        <path d="M19 21.4 Q32 27 45 21.4 Q32 23.6 19 21.4 Z" fill="#111" />
        <circle cx={32} cy={15.4} r={1.8} fill={GOLD} />
      </g>
    );
  return (
    <g data-part="hat-fire">
      <path d="M16 22 C15 6 49 6 48 22 Z" fill="#C62828" />
      <path d="M16.9 17 L47.1 17 L47.8 19.6 L16.2 19.6 Z" fill={SILVER} />
      <path d="M21 12.5 Q32 6.5 43 12.5 L41 15 Q32 10.5 23 15 Z" fill="#263238" opacity={0.85} />
      <path d="M12 22.4 Q32 19 52 22.4 L52 24 Q32 21.4 12 24 Z" fill="#9E1B1B" />
    </g>
  );
}
