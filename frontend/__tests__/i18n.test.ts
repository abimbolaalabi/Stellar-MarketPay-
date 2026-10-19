import fs from "fs";
import path from "path";
import { createElement } from "react";
import { cleanup, render, screen } from "@testing-library/react";

import i18n from "@/lib/i18n";
import Custom404 from "@/pages/404";

const LOCALES_DIR = path.join(__dirname, "../public/locales");
const REQUIRED_LOCALES = ["es", "fr", "pt"];

function flattenObject(obj: Record<string, unknown>, prefix = ""): Record<string, unknown> {
  return Object.entries(obj).reduce((result: Record<string, unknown>, [key, value]) => {
    const keyPath = prefix ? `${prefix}.${key}` : key;
    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      Object.assign(result, flattenObject(value as Record<string, unknown>, keyPath));
    } else {
      result[keyPath] = value;
    }
    return result;
  }, {});
}

describe("i18n translations completeness", () => {
  const english = flattenObject(
    JSON.parse(fs.readFileSync(path.join(LOCALES_DIR, "en/common.json"), "utf-8")),
  );

  it.each(REQUIRED_LOCALES)("has every English key translated in %s", (locale) => {
    const translated = flattenObject(
      JSON.parse(fs.readFileSync(path.join(LOCALES_DIR, locale, "common.json"), "utf-8")),
    );

    const missing = Object.keys(english).filter((key) => !(key in translated));
    const placeholders = Object.entries(translated)
      .filter(([, value]) => typeof value === "string" && value.includes("[TRANSLATE]"))
      .map(([key]) => key);

    expect(missing).toEqual([]);
    expect(placeholders).toEqual([]);
  });
});

describe("404 page translations", () => {
  const cases: Array<[locale: string, title: string, homeLabel: string]> = [
    ["en", "Page Not Found", "Go Home"],
    ["es", "Página no encontrada", "Ir al inicio"],
    ["fr", "Page introuvable", "Retour à l'accueil"],
    ["pt", "Página não encontrada", "Ir para o início"],
  ];

  afterEach(async () => {
    cleanup();
    await i18n.changeLanguage("en");
  });

  it.each(cases)("renders translated 404 content in %s", async (locale, title, homeLabel) => {
    await i18n.changeLanguage(locale);
    render(createElement(Custom404));

    expect(screen.getByRole("heading", { name: title })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: homeLabel })).toHaveAttribute("href", "/");
  });
});
