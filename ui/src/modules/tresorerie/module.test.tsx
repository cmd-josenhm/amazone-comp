import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, vi } from "vitest";
import { appeler } from "../../lib/ipc";
import type { TresorerieCompte, TresorerieSolde } from "./types";
import { declaration } from "./module";

vi.mock("../../lib/ipc", () => ({
  appeler: vi.fn(),
  ErreurAppel: class extends Error {},
}));

const compte: TresorerieCompte = {
  id: 1,
  type_compte: "BANQUE",
  libelle: "Banque OBA",
  banque_operateur: "OBA",
  numero: "••••1010",
  compte_comptable: "5211",
  journal: "BQ1",
  solde_ouverture: 50_000,
  actif: true,
  date_ouverture: "2026-01-01T00:00:00Z",
};

const solde: TresorerieSolde = {
  compte: "5211",
  type_compte: "BANQUE",
  solde: 50_000,
};

const Page = declaration.Page;
let comptesServeur: TresorerieCompte[];
let mobileMoneyActif: boolean;

beforeEach(() => {
  comptesServeur = [];
  mobileMoneyActif = true;
  vi.mocked(appeler).mockReset();
  vi.mocked(appeler).mockImplementation(async (commande) => {
    if (commande === "tresorerie_comptes") return comptesServeur;
    if (commande === "tresorerie_soldes") return comptesServeur.length > 0 ? [solde] : [];
    if (commande === "tresorerie_operateurs") {
      return [
        { id: 1, nom: "MTN MoMo", libelle: "MTN Mobile Money", actif: true },
        { id: 2, nom: "Moov Money", libelle: "Moov Money", actif: true },
      ];
    }
    if (commande === "tresorerie_configuration") {
      return { mobile_money_actif: mobileMoneyActif };
    }
    if (commande === "tresorerie_mobile_money_configurer") {
      mobileMoneyActif = false;
      return { mobile_money_actif: mobileMoneyActif };
    }
    if (commande === "tresorerie_compte_creer") {
      comptesServeur = [compte];
      return compte;
    }
    return undefined;
  });
});

describe("Module Trésorerie", () => {
  it("crée un compte avec une ouverture justifiée et rafraîchit les soldes", async () => {
    render(<Page />);

    fireEvent.change(screen.getByLabelText(/Libellé du compte/), {
      target: { value: "Banque OBA" },
    });
    fireEvent.change(screen.getByLabelText(/^Banque/), { target: { value: "OBA" } });
    fireEvent.change(screen.getByLabelText(/Numéro de compte bancaire/), {
      target: { value: "0000001010" },
    });
    fireEvent.change(screen.getByLabelText(/Solde d'ouverture/), {
      target: { value: "50000" },
    });
    fireEvent.change(screen.getByLabelText(/Référence de la pièce justificative/), {
      target: { value: "Relevé initial" },
    });
    fireEvent.change(screen.getByLabelText(/Compte de contrepartie/), {
      target: { value: "101" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Créer le compte" }));

    await waitFor(() => {
      expect(appeler).toHaveBeenCalledWith("tresorerie_compte_creer", {
        params: {
          type_compte: "BANQUE",
          libelle: "Banque OBA",
          banque_operateur: "OBA",
          numero: "0000001010",
          solde_ouverture: 50_000,
          piece_ouverture: "Relevé initial",
          compte_contrepartie_ouverture: "101",
        },
      });
    });
    expect(await screen.findByText("••••1010")).toBeInTheDocument();
    expect(screen.getByText(/50[\s\u202f]000 FCFA/)).toBeInTheDocument();
  });

  it("permet d'activer ou désactiver Mobile Money pour le dossier", async () => {
    render(<Page />);
    const bouton = await screen.findByRole("button", { name: "Désactiver Mobile Money" });
    fireEvent.click(bouton);

    await waitFor(() => {
      expect(appeler).toHaveBeenCalledWith("tresorerie_mobile_money_configurer", { actif: false });
    });
    expect(await screen.findByText("Module Mobile Money : désactivé.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Activer Mobile Money" })).toBeInTheDocument();
  });
});
