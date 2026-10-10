import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { AvatarGallery } from "../screens/AvatarGallery";
import { Avatar } from "./Avatar";
import {
  HAIR_STYLES,
  isValidLook,
  KNOWN_LOOKS,
  lookFor,
  lookFromSeed,
  MENTOR_LOOK,
  PLAYER_LOOK,
} from "./look";

afterEach(cleanup);

describe("izgled lika", () => {
  it("isto ime uvijek daje isti izgled", () => {
    expect(lookFromSeed("Ana Kovač")).toEqual(lookFromSeed("Ana Kovač"));
  });
  it("izgled iz imena je zamrznut: promjena palete ga ne smije pomaknuti", () => {
    expect(lookFromSeed("Ana Kovač")).toEqual({
      skin: 1,
      hair: "buzz",
      hairColor: 2,
      eyes: "happy",
      mouth: "worried",
      facialHair: "none",
      glasses: "none",
      hat: "none",
      clothes: 2,
      backdrop: 2,
    });
  });
  it("ime koje je svojstvo Objecta pada na izgled iz imena", () => {
    expect(lookFor("constructor")).toEqual(lookFromSeed("constructor"));
  });
  it("izgled iz imena je uvijek valjan", () => {
    for (let i = 0; i < 300; i++) expect(isValidLook(lookFromSeed(`lik-${i}`))).toBe(true);
  });
  it("različita imena daju raznolike likove", () => {
    const names = ["Ana", "Ivan", "Maja", "Luka", "Petar", "Iva", "Tomislav", "Lana", "Karlo", "Ema"];
    const distinct = new Set(names.map((n) => JSON.stringify(lookFromSeed(n))));
    expect(distinct.size).toBeGreaterThanOrEqual(8);
    const hairs = new Set(names.map((n) => lookFromSeed(n).hair));
    expect(hairs.size).toBeGreaterThanOrEqual(4);
  });
  it("poznati likovi imaju ručno izabran izgled, nepoznati izgled iz imena", () => {
    expect(lookFor("Petra Horvat")).toEqual(KNOWN_LOOKS["Petra Horvat"]);
    expect(lookFor("Marko Babić").hat).toBe("chef");
    expect(lookFor("Netko Novi")).toEqual(lookFromSeed("Netko Novi"));
  });
  it("igrač, mentor i poznati likovi su valjani", () => {
    for (const l of [PLAYER_LOOK, MENTOR_LOOK, ...Object.values(KNOWN_LOOKS)])
      expect(isValidLook(l)).toBe(true);
  });
  it("isValidLook odbija indeks izvan palete i nepoznatu opciju", () => {
    expect(isValidLook({ ...PLAYER_LOOK, skin: 99 })).toBe(false);
    expect(isValidLook({ ...PLAYER_LOOK, hair: "mohawk" as never })).toBe(false);
  });
});

describe("Avatar", () => {
  it("crta slojeve koje izgled traži, i samo njih", () => {
    const { container } = render(
      <Avatar look={{ ...PLAYER_LOOK, hair: "bun", glasses: "round", hat: "none" }} />,
    );
    expect(container.querySelector('[data-part="hair-bun"]')).not.toBeNull();
    expect(container.querySelector('[data-part="glasses-round"]')).not.toBeNull();
    expect(container.querySelector('[data-part^="hat-"]')).toBeNull();
  });
  it("svaka frizura ima svoj sloj (osim „bez kose“)", () => {
    for (const hair of HAIR_STYLES) {
      const { container, unmount } = render(<Avatar look={{ ...PLAYER_LOOK, hair }} />);
      const part = container.querySelector(`[data-part="hair-${hair}"]`);
      expect(hair === "none" ? part === null : part !== null).toBe(true);
      unmount();
    }
  });
  it("bez naslova je skriven čitaču, s naslovom je slika s imenom", () => {
    const { container } = render(<Avatar look={PLAYER_LOOK} />);
    expect(container.querySelector("svg")?.getAttribute("aria-hidden")).toBe("true");
    render(<Avatar look={MENTOR_LOOK} title="Mentor" />);
    expect(screen.getByRole("img", { name: "Mentor" })).toBeTruthy();
  });
  it("dva lika na istom ekranu ne dijele id izreza", () => {
    const { container } = render(
      <>
        <Avatar look={PLAYER_LOOK} />
        <Avatar look={MENTOR_LOOK} />
      </>,
    );
    const ids = [...container.querySelectorAll("clipPath")].map((c) => c.id);
    expect(new Set(ids).size).toBe(2);
  });
});

describe("AvatarGallery", () => {
  it("pokazuje poznate likove i sve frizure", () => {
    const { container } = render(<AvatarGallery />);
    expect(screen.getByText("Petra Horvat")).toBeTruthy();
    expect(screen.getByText("Marko Babić")).toBeTruthy();
    expect(container.querySelectorAll("svg").length).toBeGreaterThanOrEqual(2 + 2 + 9 + 8);
  });
});
