import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Layout } from "./Layout";

afterEach(cleanup);

describe("Layout", () => {
  it("ima jedan glavni izbornik, a aktivna stavka ima aria-current", () => {
    window.history.pushState({}, "", "/situacije");
    render(
      <Layout chip="JUN 1/4">
        <p>sadržaj</p>
      </Layout>,
    );
    expect(screen.getAllByRole("navigation", { name: "Glavni izbornik" })).toHaveLength(1);
    expect(screen.getByRole("link", { name: "Situacije" }).getAttribute("aria-current")).toBe("page");
    expect(screen.getByRole("link", { name: "Hotel" }).getAttribute("aria-current")).toBeNull();
    expect(screen.getByText("JUN 1/4")).toBeTruthy();
  });

  it("stavka „Odjeli“ postoji samo za v2 igru", () => {
    window.history.pushState({}, "", "/odjeli");
    const { rerender } = render(
      <Layout chip={null} withDepartments>
        <p>sadržaj</p>
      </Layout>,
    );
    expect(screen.getByRole("link", { name: "Odjeli" }).getAttribute("aria-current")).toBe("page");
    rerender(
      <Layout chip={null}>
        <p>sadržaj</p>
      </Layout>,
    );
    expect(screen.queryByRole("link", { name: "Odjeli" })).toBeNull();
  });
});
