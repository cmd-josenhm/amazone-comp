// Tests de l'écran « Clients » — Propriétaire : A4. Exemples chiffrés écrits à la main.
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { appeler, ErreurAppel } from "../../lib/ipc";
import { Clients } from "./clients";
import type { Client } from "./types";

vi.mock("../../lib/ipc", async (importOriginal) => {
  const original = await importOriginal<typeof import("../../lib/ipc")>();
  return { ...original, appeler: vi.fn() };
});

const appelerMock = vi.mocked(appeler);

const comptant: Client = {
  id: 1,
  code: "CLI-000001",
  nom: "Client comptant",
  type_client: "PARTICULIER",
  ifu: null,
  rccm: null,
  adresse: null,
  telephone: null,
  courriel: null,
  retient_tva: false,
  exonere_aib: false,
  motif_exoneration_aib: null,
  actif: true,
  cree_le: "2026-11-27T10:00:00Z",
  modifie_le: null,
};

const etoile: Client = {
  id: 2,
  code: "CLI-000002",
  nom: "Boutique Étoile",
  type_client: "ENTREPRISE",
  ifu: "3201912345678",
  rccm: "RB/COT/23 B 12345",
  adresse: "Ganhi, Cotonou",
  telephone: "+2290197123456",
  courriel: "contact@etoile.bj",
  retient_tva: true,
  exonere_aib: false,
  motif_exoneration_aib: null,
  actif: true,
  cree_le: "2026-11-27T10:05:00Z",
  modifie_le: null,
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe("écran Clients", () => {
  it("affiche les clients au chargement et protège le client Comptant", async () => {
    appelerMock.mockResolvedValue([comptant, etoile]);
    render(<Clients />);
    expect(await screen.findByText("Boutique Étoile")).toBeInTheDocument();
    expect(screen.getByText("CLI-000002")).toBeInTheDocument();
    expect(screen.getByText("Client comptant")).toBeInTheDocument();
    // Le client Comptant n'a ni « Désactiver » ni « Modifier ».
    expect(screen.getAllByRole("button", { name: "Désactiver" })).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "Modifier" })).toHaveLength(1);
    expect(screen.getByText("Obligatoire")).toBeInTheDocument();
    expect(appelerMock).toHaveBeenCalledWith("ventes_clients_rechercher", {
      recherche: { texte: "", inclure_inactifs: false, limite: 100 },
    });
  });

  it("recherche un client par texte", async () => {
    appelerMock.mockResolvedValue([comptant, etoile]);
    render(<Clients />);
    await screen.findByText("Boutique Étoile");
    fireEvent.change(screen.getByLabelText("Rechercher (nom, IFU, téléphone)"), {
      target: { value: "345678" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Rechercher" }));
    await waitFor(() =>
      expect(appelerMock).toHaveBeenCalledWith("ventes_clients_rechercher", {
        recherche: { texte: "345678", inclure_inactifs: false, limite: 100 },
      }),
    );
  });

  it("crée un client et renvoie à la liste", async () => {
    appelerMock.mockImplementation((commande) => {
      if (commande === "ventes_client_creer") {
        return Promise.resolve(etoile);
      }
      return Promise.resolve([comptant, etoile]);
    });
    render(<Clients />);
    await screen.findByText("Boutique Étoile");
    fireEvent.click(screen.getByRole("button", { name: "Nouveau client" }));
    fireEvent.change(screen.getByLabelText(/Nom ou raison sociale/), {
      target: { value: "Boutique Étoile" },
    });
    fireEvent.change(screen.getByLabelText(/Type de client/), { target: { value: "ENTREPRISE" } });
    fireEvent.change(screen.getByLabelText(/IFU/), { target: { value: "3201912345678" } });
    fireEvent.change(screen.getByLabelText(/Téléphone/), {
      target: { value: "01 97 12 34 56" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Enregistrer" }));
    await waitFor(() =>
      expect(appelerMock).toHaveBeenCalledWith("ventes_client_creer", {
        informations: {
          nom: "Boutique Étoile",
          type_client: "ENTREPRISE",
          ifu: "3201912345678",
          rccm: null,
          adresse: null,
          telephone: "01 97 12 34 56",
          courriel: null,
          retient_tva: false,
          exonere_aib: false,
          motif_exoneration_aib: null,
        },
      }),
    );
    // Retour à la liste, rechargée.
    await waitFor(() => expect(screen.getByRole("button", { name: "Nouveau client" })).toBeInTheDocument());
  });

  it("affiche l'erreur du cœur en cas de refus", async () => {
    appelerMock.mockImplementation((commande) => {
      if (commande === "ventes_client_creer") {
        return Promise.reject(
          new ErreurAppel({
            code: "IFU_DEJA_UTILISE",
            message: "un autre client porte déjà l'IFU 3201912345678",
          }),
        );
      }
      return Promise.resolve([comptant]);
    });
    render(<Clients />);
    await screen.findByText("Client comptant");
    fireEvent.click(screen.getByRole("button", { name: "Nouveau client" }));
    fireEvent.change(screen.getByLabelText(/Nom ou raison sociale/), {
      target: { value: "Client bis" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Enregistrer" }));
    expect(
      await screen.findByText("un autre client porte déjà l'IFU 3201912345678"),
    ).toBeInTheDocument();
  });

  it("désactive un client puis recharge la liste", async () => {
    appelerMock.mockImplementation((commande) => {
      if (commande === "ventes_client_desactiver") {
        return Promise.resolve(undefined);
      }
      return Promise.resolve([comptant, etoile]);
    });
    render(<Clients />);
    await screen.findByText("Boutique Étoile");
    fireEvent.click(screen.getByRole("button", { name: "Désactiver" }));
    await waitFor(() =>
      expect(appelerMock).toHaveBeenCalledWith("ventes_client_desactiver", { id: 2 }),
    );
  });

  it("modifie un client à partir de la liste", async () => {
    appelerMock.mockImplementation((commande) => {
      if (commande === "ventes_client_modifier") {
        return Promise.resolve({ ...etoile, nom: "Boutique Étoile SARL" });
      }
      return Promise.resolve([comptant, etoile]);
    });
    render(<Clients />);
    await screen.findByText("Boutique Étoile");
    fireEvent.click(screen.getByRole("button", { name: "Modifier" }));
    const champNom = screen.getByLabelText(/Nom ou raison sociale/);
    expect(champNom).toHaveValue("Boutique Étoile");
    fireEvent.change(champNom, { target: { value: "Boutique Étoile SARL" } });
    fireEvent.click(screen.getByRole("button", { name: "Enregistrer" }));
    await waitFor(() =>
      expect(appelerMock).toHaveBeenCalledWith("ventes_client_modifier", {
        id: 2,
        informations: expect.objectContaining({ nom: "Boutique Étoile SARL" }),
      }),
    );
  });
});
