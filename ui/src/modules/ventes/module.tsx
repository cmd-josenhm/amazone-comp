// Module « Ventes » — Propriétaire : A4. Voir PROMPT_A4.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>Ventes</h1>
      <p className="texte-secondaire">Clients, devis, factures normalisées, avoirs.</p>
      <p className="texte-secondaire">Module en construction (agent A4).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "ventes",
  libelle: "Ventes",
  chemin: "/ventes",
  ordre: 10,
  Page,
};
