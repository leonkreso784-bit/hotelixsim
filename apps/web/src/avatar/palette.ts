// Boje LIKA su podatak (kao u BitLifeu: koža, kosa, odjeća), ne tema sučelja — zato su ovdje, a ne u styles.css.
// Indeksi u ove nizove spremaju se u AvatarLook; novi element ide NA KRAJ niza (stari izgledi se ne mijenjaju).
export const SKIN = [
  "#FCE3D3",
  "#F5CBA7",
  "#E0AC69",
  "#C68642",
  "#8D5524",
  "#5C3A21",
  "#F1D3BE",
  "#3F2A1D",
] as const;
/** crna, tamnosmeđa, smeđa, plava, riđa, sijeda, svijetloplava, kestenjasta */
export const HAIR = [
  "#1F1A17",
  "#4A2F1D",
  "#8B5A2B",
  "#D9A441",
  "#B5552B",
  "#A7ADB4",
  "#E8CFA0",
  "#6B2E1F",
] as const;
/** plava, zelena, oker, ljubičasta, crvena, tamna, bijela (kuhar), tirkizna, malina, kava */
export const CLOTHES = [
  "#246BFD",
  "#15924A",
  "#E0A82E",
  "#7C4DFF",
  "#DC2F2F",
  "#2C3E50",
  "#F7F7F7",
  "#00897B",
  "#C2185B",
  "#8D6E63",
] as const;
/** pijesak, menta, svijetloplava — boje koncepta v2 */
export const BACKDROP = ["#F4E8D5", "#D8F0EE", "#EAF1FF"] as const;
