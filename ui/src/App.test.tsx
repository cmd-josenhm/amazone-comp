import { fireEvent, render, screen } from "@testing-library/react";
import { App } from "./App";
import { formaterMontant } from "./design/composants";
import { modules } from "./modules/registre";

describe("Coque", () => {
  it("affiche tous les modules dans le menu", () => {
    render(<App />);
    for (const m of modules) {
      expect(screen.getByRole("link", { name: m.libelle })).toBeInTheDocument();
    }
    expect(screen.getByRole("link", { name: "À propos" })).toBeInTheDocument();
  });

  it("la page À propos affiche les champs du créateur à compléter et le formulaire d'avis", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("link", { name: "À propos" }));
    expect(screen.getAllByText("A COMPLETER").length).toBeGreaterThanOrEqual(5);
    expect(screen.getByRole("button", { name: "Envoyer mon avis" })).toBeInTheDocument();
  });

  it("refuse un avis vide", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("link", { name: "À propos" }));
    fireEvent.click(screen.getByRole("button", { name: "Envoyer mon avis" }));
    expect(screen.getByText(/quelques mots/)).toBeInTheDocument();
  });
});

describe("formaterMontant", () => {
  it("formate en français sans décimales", () => {
    expect(formaterMontant(1234567).replace(/\s/g, " ")).toBe("1 234 567 FCFA");
    expect(formaterMontant(0)).toBe("0 FCFA");
    expect(formaterMontant(1.5)).toBe("montant invalide");
  });
});
