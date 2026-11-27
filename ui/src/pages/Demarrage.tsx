// Assistant de premier lancement — Propriétaire : A3 (écrans), A10 (compte/licence), A2 (dossier).
import { Carte } from "../design/composants";

export const etapesDemarrage = [
  "Conditions d'utilisation : lire et accepter",
  "Compte AMAZONE COMP : créer un compte ou se connecter (Internet nécessaire une seule fois)",
  "Entreprise : raison sociale, IFU, forme juridique, régime fiscal (réel normal, réel simplifié, TPS)",
  "Administrateur : identifiant, mot de passe, impression de la clé de secours",
  "Comptabilité : exercice, plan de comptes SYSCOHADA, journaux, comptes de banque, caisse, Mobile Money",
  "Reprise : importer une balance d'ouverture, des clients, des fournisseurs, des articles (facultatif)",
  "Terminé : visite guidée du tableau de bord",
] as const;

export function Demarrage() {
  return (
    <div>
      <h1>Bienvenue dans AMAZONE COMP</h1>
      <Carte titre="Mise en route en 7 étapes">
        <ol>
          {etapesDemarrage.map((e) => (
            <li key={e}>{e}</li>
          ))}
        </ol>
        <p className="texte-secondaire">Assistant en construction (A3).</p>
      </Carte>
    </div>
  );
}
