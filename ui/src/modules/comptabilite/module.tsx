// Module « Comptabilité » — Propriétaire : A1. Voir PROMPT_A1.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>Comptabilité</h1>
      <p className="texte-secondaire">Plan de comptes, journaux, saisie d'opérations diverses, lettrage, clôture.</p>
      <p className="texte-secondaire">Module en construction (agent A1).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "comptabilite",
  libelle: "Comptabilité",
  chemin: "/comptabilite",
  ordre: 60,
  Page,
};
