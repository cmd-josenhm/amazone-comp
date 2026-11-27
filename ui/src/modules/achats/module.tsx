// Module « Achats et stocks » — Propriétaire : A5. Voir PROMPT_A5.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>Achats et stocks</h1>
      <p className="texte-secondaire">Fournisseurs, commandes, réceptions, articles, inventaires.</p>
      <p className="texte-secondaire">Module en construction (agent A5).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "achats",
  libelle: "Achats et stocks",
  chemin: "/achats",
  ordre: 20,
  Page,
};
