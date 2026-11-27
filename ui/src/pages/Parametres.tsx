// Paramètres — Propriétaire : A3 (affichage). Logique des mises à jour : A10.
import { useState } from "react";
import { Bandeau, Carte } from "../design/composants";

type ModeMiseAJour = "automatique" | "manuel";

export function Parametres() {
  const [mode, setMode] = useState<ModeMiseAJour>("automatique");
  return (
    <div>
      <h1>Paramètres</h1>
      <Carte titre="Mises à jour">
        <fieldset style={{ border: "none", padding: 0, margin: 0 }}>
          <legend className="texte-secondaire">
            Une licence valide est nécessaire pour recevoir les mises à jour.
          </legend>
          <label style={{ display: "block", margin: "8px 0" }}>
            <input
              type="radio"
              name="mode-maj"
              checked={mode === "automatique"}
              onChange={() => setMode("automatique")}
            />{" "}
            Automatiques : télécharger et proposer l'installation dès qu'une version est disponible
          </label>
          <label style={{ display: "block", margin: "8px 0" }}>
            <input
              type="radio"
              name="mode-maj"
              checked={mode === "manuel"}
              onChange={() => setMode("manuel")}
            />{" "}
            Manuelles : je vérifie moi-même depuis cette page
          </label>
        </fieldset>
        <Bandeau>Enregistrement du choix et recherche de mise à jour : en construction (A10).</Bandeau>
      </Carte>
      <Carte titre="Dossier de l'entreprise">
        <p className="texte-secondaire">Raison sociale, IFU, régime fiscal, exercice : en construction (A2 / A3).</p>
      </Carte>
      <Carte titre="Utilisateurs et rôles">
        <p className="texte-secondaire">
          Administrateur, Comptable, Saisie, Paie, Lecture : en construction (A2 / A3).
        </p>
      </Carte>
    </div>
  );
}
