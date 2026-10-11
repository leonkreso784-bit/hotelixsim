import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Link, matchRoute, useRoute } from "./router";

afterEach(cleanup);

describe("matchRoute", () => {
  it("prepoznaje sve rute igrača", () => {
    expect(matchRoute("/")).toEqual({ name: "hotel" });
    expect(matchRoute("/demo")).toEqual({ name: "hotel" });
    expect(matchRoute("/situacije")).toEqual({ name: "situations" });
    expect(matchRoute("/situacije/S1")).toEqual({ name: "situation", code: "S1" });
    expect(matchRoute("/plan")).toEqual({ name: "plan" });
    expect(matchRoute("/mjesec/jun")).toEqual({ name: "round", code: "jun" });
    expect(matchRoute("/tijek/")).toEqual({ name: "feed" });
    expect(matchRoute("/odjeli")).toEqual({ name: "departments" });
    expect(matchRoute("/odjeli/restaurant")).toEqual({ name: "department", code: "restaurant" });
    expect(matchRoute("/odjeli/a/b")).toEqual({ name: "notFound" });
    expect(matchRoute("/likovi")).toEqual({ name: "avatars" });
    expect(matchRoute("/pokazatelji")).toEqual({ name: "kpi" });
    expect(matchRoute("/pokazatelj/Z")).toEqual({ name: "indicator", code: "Z", highlight: null });
    expect(matchRoute("/pokazatelj/F/revenue")).toEqual({
      name: "indicator",
      code: "F",
      highlight: "revenue",
    });
  });
  it("sve ostalo je notFound", () => {
    expect(matchRoute("/mjesec/xyz")).toEqual({ name: "notFound" });
    expect(matchRoute("/situacije/S1/x")).toEqual({ name: "notFound" });
    expect(matchRoute("/pokazatelj")).toEqual({ name: "notFound" });
    expect(matchRoute("/pokazatelji/jun")).toEqual({ name: "notFound" });
    expect(matchRoute("/pokazatelj/X")).toEqual({ name: "notFound" });
    expect(matchRoute("/pokazatelj/Z/nema")).toEqual({ name: "notFound" });
    expect(matchRoute("/pokazatelj/Z/staff/x")).toEqual({ name: "notFound" });
    expect(matchRoute("/nesto")).toEqual({ name: "notFound" });
    expect(matchRoute("/situacije/%")).toEqual({ name: "notFound" });
  });
});

function Where() {
  const r = useRoute();
  return (
    <div>
      <p>ruta:{r.name}</p>
      <Link to="/tijek">tijek</Link>
    </div>
  );
}

describe("Link i useRoute", () => {
  it("klik na Link mijenja putanju bez punog učitavanja i obavijesti useRoute", () => {
    window.history.replaceState(null, "", "/");
    render(<Where />);
    expect(screen.getByText("ruta:hotel")).toBeTruthy();
    fireEvent.click(screen.getByRole("link", { name: "tijek" }));
    expect(window.location.pathname).toBe("/tijek");
    expect(screen.getByText("ruta:feed")).toBeTruthy();
  });
  it("povratak (popstate) također osvježi rutu", () => {
    window.history.replaceState(null, "", "/plan");
    render(<Where />);
    window.history.replaceState(null, "", "/situacije");
    fireEvent(window, new PopStateEvent("popstate"));
    expect(screen.getByText("ruta:situations")).toBeTruthy();
  });
});

describe("krugovi", () => {
  it("krug po kodu i stari put mjeseca", () => {
    expect(matchRoute("/krug/t05")).toEqual({ name: "round", code: "t05" });
    expect(matchRoute("/mjesec/jun")).toEqual({ name: "round", code: "jun" });
    expect(matchRoute("/krug/t18")).toEqual({ name: "notFound" });
  });
});
