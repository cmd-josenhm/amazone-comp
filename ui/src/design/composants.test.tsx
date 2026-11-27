import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  Assistant,
  Bandeau,
  Bouton,
  Carte,
  CaseACocher,
  Champ,
  ChampDate,
  ChampMontant,
  Chargement,
  Dialogue,
  EtatVide,
  MessageErreur,
  Notification,
  Onglets,
  Selecteur,
  Tableau,
  type ColonneTableau,
} from "./composants";

type Ligne = { id: number; nom: string; montant: number };

const colonnes: readonly ColonneTableau<Ligne>[] = [
  { id: "nom", entete: "Nom", cle: "nom" },
  { id: "montant", entete: "Montant", cle: "montant", format: "montant" },
];

const lignes: readonly Ligne[] = [{ id: 1, nom: "Awa", montant: 1250000 }];

describe("composants de base", () => {
  it("rend un bouton, un champ, une carte et un bandeau avec leurs noms accessibles", () => {
    const surChangement = vi.fn();
    render(
      <>
        <Bouton variante="principal">Enregistrer</Bouton>
        <Champ libelle="Nom" valeur="Awa" surChangement={surChangement} aide="Aide du champ" />
        <Carte titre="Résumé">Contenu</Carte>
        <Bandeau type="alerte">Attention</Bandeau>
      </>,
    );

    expect(screen.getByRole("button", { name: "Enregistrer" })).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Nom" })).toHaveValue("Awa");
    expect(screen.getByRole("heading", { name: "Résumé" })).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("Attention");
  });
});

describe("Tableau", () => {
  it("affiche les montants, déclenche le tri, la pagination et le clic clavier d'une ligne", () => {
    const surTri = vi.fn();
    const surPage = vi.fn();
    const surLigne = vi.fn();
    render(
      <Tableau
        colonnes={colonnes}
        lignes={lignes}
        page={1}
        total={20}
        surPage={5}
        tri={{ colonne: "nom", direction: "asc" }}
        surChangementTri={surTri}
        surChangementPage={surPage}
        surLigne={surLigne}
        libelleLigne={(ligne) => ligne.nom}
      />,
    );

    expect(screen.getByRole("table", { name: "Tableau" })).toBeInTheDocument();
    expect(screen.getByText(/1[\u00a0 ]250[\u00a0 ]000 FCFA/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Trier par Nom" }));
    expect(surTri).toHaveBeenCalledWith({ colonne: "nom", direction: "desc" });
    fireEvent.click(screen.getByRole("button", { name: "Page suivante" }));
    expect(surPage).toHaveBeenCalledWith(2);
    const ligne = screen.getByRole("row", { name: "Awa" });
    fireEvent.keyDown(ligne, { key: "Enter" });
    expect(surLigne).toHaveBeenCalledWith(lignes[0]);
  });

  it("gère une page vide, une colonne non triable et les boutons de pagination", () => {
    const surPage = vi.fn();
    const colonnesAvecContenu: readonly ColonneTableau<Ligne>[] = [
      { id: "nom", entete: "Nom affiché", contenu: (ligne) => ligne.nom, triable: false, alignement: "centre" },
      { id: "montant", entete: "Montant", cle: "montant", montant: true },
    ];
    const { rerender } = render(
      <Tableau
        colonnes={colonnesAvecContenu}
        lignes={[]}
        page={0}
        total={0}
        surPage={0}
        surChangementPage={surPage}
      />,
    );
    expect(screen.getByRole("status", { name: "Aucun résultat" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Trier par Nom affiché" })).not.toBeInTheDocument();
    rerender(
      <Tableau
        colonnes={colonnesAvecContenu}
        lignes={lignes}
        page={2}
        total={20}
        surPage={5}
        surPageChange={surPage}
        surTriChange={() => undefined}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Première page" }));
    fireEvent.click(screen.getByRole("button", { name: "Page précédente" }));
    fireEvent.click(screen.getByRole("button", { name: "Dernière page" }));
    expect(surPage).toHaveBeenCalledTimes(3);
    expect(surPage).toHaveBeenNthCalledWith(1, 1);
    expect(surPage).toHaveBeenNthCalledWith(2, 1);
    expect(surPage).toHaveBeenNthCalledWith(3, 4);
  });
});

describe("ChampMontant", () => {
  it("transmet 1 250 000 comme entier et refuse une décimale", () => {
    const surChangement = vi.fn();
    render(<ChampMontant libelle="Montant" surChangement={surChangement} />);
    const champ = screen.getByRole("textbox", { name: "Montant" });

    fireEvent.change(champ, { target: { value: "1 250 000" } });
    expect(surChangement).toHaveBeenLastCalledWith(1250000);
    fireEvent.change(champ, { target: { value: "12,5" } });
    expect(surChangement).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("alert")).toHaveTextContent("nombre entier");
  });

  it("refuse un montant négatif par défaut et l'accepte avec negatifAutorise", () => {
    const interdit = vi.fn();
    const autorise = vi.fn();
    const { rerender } = render(<ChampMontant libelle="Montant" surChangement={interdit} />);
    fireEvent.change(screen.getByRole("textbox", { name: "Montant" }), { target: { value: "-500" } });
    expect(interdit).not.toHaveBeenCalled();
    rerender(<ChampMontant libelle="Montant" surChangement={autorise} negatifAutorise />);
    fireEvent.change(screen.getByRole("textbox", { name: "Montant" }), { target: { value: "-500 " } });
    expect(autorise).toHaveBeenCalledWith(-500);
  });

  it("refuse les lettres et les entiers trop grands, et accepte la saisie vide", () => {
    const surChangement = vi.fn();
    render(<ChampMontant libelle="Montant" surChangement={surChangement} />);
    const champ = screen.getByRole("textbox", { name: "Montant" });
    fireEvent.change(champ, { target: { value: "abc" } });
    expect(screen.getByRole("alert")).toHaveTextContent("sans lettre");
    fireEvent.change(champ, { target: { value: "999999999999999999999" } });
    expect(screen.getByRole("alert")).toHaveTextContent("trop grand");
    fireEvent.change(champ, { target: { value: "" } });
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});

describe("ChampDate et Selecteur", () => {
  it("ouvre le calendrier, le ferme avec Échap et choisit une date", () => {
    const surChangement = vi.fn();
    render(<ChampDate libelle="Date" valeur="27/11/2026" surChangement={surChangement} />);
    fireEvent.click(screen.getByRole("button", { name: "Ouvrir le calendrier" }));
    expect(screen.getByRole("dialog", { name: "Calendrier" })).toBeInTheDocument();
    fireEvent.keyDown(screen.getByRole("dialog", { name: "Calendrier" }), { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Calendrier" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Ouvrir le calendrier" }));
    fireEvent.click(screen.getByRole("gridcell", { name: "28 novembre 2026" }));
    expect(surChangement).toHaveBeenCalledWith("28/11/2026");
  });

  it("navigue entre les mois et affiche une erreur de format", () => {
    const surChangement = vi.fn();
    render(<ChampDate libelle="Date" valeur="" surChangement={surChangement} />);
    const champ = screen.getByRole("textbox", { name: "Date" });
    fireEvent.change(champ, { target: { value: "27-11" } });
    expect(screen.getByRole("alert")).toHaveTextContent("JJ/MM/AAAA");
    fireEvent.click(screen.getByRole("button", { name: "Ouvrir le calendrier" }));
    fireEvent.click(screen.getByRole("button", { name: "Mois suivant" }));
    fireEvent.click(screen.getByRole("button", { name: "Mois précédent" }));
    expect(screen.getByRole("dialog", { name: "Calendrier" })).toBeInTheDocument();
  });

  it("recherche une option avec les flèches et Entrée", () => {
    const surChangement = vi.fn();
    render(
      <Selecteur
        libelle="Secteur"
        options={[
          { valeur: "commerce", libelle: "Commerce" },
          { valeur: "services", libelle: "Services" },
        ]}
        surChangement={surChangement}
      />,
    );
    const champ = screen.getByRole("combobox", { name: "Secteur" });
    fireEvent.focus(champ);
    fireEvent.change(champ, { target: { value: "serv" } });
    fireEvent.keyDown(champ, { key: "ArrowDown" });
    fireEvent.keyDown(champ, { key: "Enter" });
    expect(surChangement).toHaveBeenCalledWith("services");
  });

  it("annule une recherche et annonce l'absence de choix", () => {
    render(<Selecteur libelle="Secteur" options={[]} surChangement={() => undefined} />);
    const champ = screen.getByRole("combobox", { name: "Secteur" });
    fireEvent.focus(champ);
    expect(screen.getByText("Aucun choix trouvé")).toBeInTheDocument();
    fireEvent.keyDown(champ, { key: "ArrowUp" });
    fireEvent.keyDown(champ, { key: "Escape" });
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  });
});

describe("CaseACocher, Dialogue et Assistant", () => {
  it("gère la case à cocher et son état d'erreur", () => {
    const surChangement = vi.fn();
    render(<CaseACocher libelle="J'accepte" cochee={false} surChangement={surChangement} erreur="Obligatoire" />);
    const caseACocher = screen.getByRole("checkbox", { name: "J'accepte" });
    fireEvent.click(caseACocher);
    expect(surChangement).toHaveBeenCalledWith(true);
    expect(screen.getByRole("alert")).toHaveTextContent("Obligatoire");
  });

  it("ferme un dialogue avec Échap et garde le rôle modal", () => {
    const surFermer = vi.fn();
    render(
      <>
        <Bouton>Ouvrir</Bouton>
        <Dialogue ouvert titre="Confirmer" surFermer={surFermer} surConfirmer={() => undefined}>
          <p>Êtes-vous sûr ?</p>
        </Dialogue>
      </>,
    );
    const dialogue = screen.getByRole("dialog", { name: "Confirmer" });
    expect(dialogue).toHaveAttribute("aria-modal", "true");
    fireEvent.keyDown(document, { key: "Escape" });
    expect(surFermer).toHaveBeenCalledTimes(1);
  });

  it("n'avance pas si le bouton est bloqué et affiche le résumé à la dernière étape", () => {
    const surValider = vi.fn();
    const { rerender } = render(
      <Assistant
        etapes={[
          { titre: "Première étape", contenu: <p>Départ</p> },
          { titre: "Dernière étape", contenu: <p>Fin</p> },
        ]}
        suivantDesactive
        resumeFinal={<p>Tout est relu.</p>}
        surValider={surValider}
      />,
    );
    expect(screen.getByRole("button", { name: "Suivant" })).toBeDisabled();
    rerender(
      <Assistant
        etapes={[
          { titre: "Première étape", contenu: <p>Départ</p> },
          { titre: "Dernière étape", contenu: <p>Fin</p> },
        ]}
        resumeFinal={<p>Tout est relu.</p>}
        surValider={surValider}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Première étape" }));
    expect(screen.getByText("Départ")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Suivant" }));
    expect(screen.getByText("Fin")).toBeInTheDocument();
    expect(screen.getByText("Résumé avant validation")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Valider" }));
    expect(surValider).toHaveBeenCalledTimes(1);
  });

  it("affiche un assistant vide", () => {
    render(<Assistant etapes={[]} />);
    expect(screen.getByRole("status", { name: "Aucune étape" })).toBeInTheDocument();
  });
});

describe("Onglets et notifications", () => {
  it("change d'onglet avec la flèche droite", () => {
    render(
      <Onglets
        onglets={[
          { id: "un", libelle: "Un", contenu: <p>Contenu un</p> },
          { id: "deux", libelle: "Deux", contenu: <p>Contenu deux</p> },
        ]}
      />,
    );
    const premier = screen.getByRole("tab", { name: "Un" });
    fireEvent.keyDown(premier, { key: "ArrowRight" });
    expect(screen.getByRole("tab", { name: "Deux" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tabpanel")).toHaveTextContent("Contenu deux");
  });

  it("gère Début, Fin et une liste d'onglets vide", () => {
    const { rerender } = render(
      <Onglets
        onglets={[
          { id: "un", libelle: "Un", contenu: <p>Contenu un</p> },
          { id: "deux", libelle: "Deux", contenu: <p>Contenu deux</p> },
        ]}
      />,
    );
    fireEvent.keyDown(screen.getByRole("tab", { name: "Un" }), { key: "End" });
    expect(screen.getByRole("tab", { name: "Deux" })).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(screen.getByRole("tab", { name: "Deux" }), { key: "Home" });
    expect(screen.getByRole("tab", { name: "Un" })).toHaveAttribute("aria-selected", "true");
    rerender(<Onglets onglets={[]} />);
    expect(screen.queryByRole("tablist")).not.toBeInTheDocument();
  });

  it("affiche puis ferme une notification au clavier et rend les états", () => {
    const surFermer = vi.fn();
    render(
      <>
        <Notification type="erreur" surFermer={surFermer}>
          Erreur lisible
        </Notification>
        <EtatVide titre="Aucun élément" description="Rien à afficher." />
        <Chargement />
        <MessageErreur erreur={{ message: "Échec de la commande" }} surRéessayer={() => undefined} />
      </>,
    );
    expect(screen.getByText("Erreur lisible")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Fermer la notification" }));
    expect(surFermer).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("status", { name: "Aucun élément" })).toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Chargement…" })).toBeInTheDocument();
    const alertes = screen.getAllByRole("alert");
    expect(alertes.some((alerte) => alerte.textContent?.includes("Échec de la commande"))).toBe(true);
    expect(within(alertes.find((alerte) => alerte.textContent?.includes("Échec de la commande")) as HTMLElement).getByRole("button", { name: "Réessayer" })).toBeInTheDocument();
  });

  it("peut rester invisible et se fermer après une durée", () => {
    vi.useFakeTimers();
    const surFermer = vi.fn();
    const { rerender } = render(<Notification visible={false} message="Invisible" surFermer={surFermer} />);
    expect(screen.queryByText("Invisible")).not.toBeInTheDocument();
    rerender(<Notification visible message="Visible" surFermer={surFermer} duree={1000} />);
    vi.advanceTimersByTime(1000);
    expect(surFermer).toHaveBeenCalledTimes(1);
    vi.useRealTimers();
  });
});
