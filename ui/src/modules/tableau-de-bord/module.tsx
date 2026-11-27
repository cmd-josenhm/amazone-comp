// Module « Tableau de bord » — Propriétaire : A9. Voir PROMPT_A9.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>Tableau de bord</h1>
      <p className="texte-secondaire">Indicateurs clés, échéances fiscales, trésorerie disponible.</p>
      <p className="texte-secondaire">Module en construction (agent A9).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "tableau-de-bord",
  libelle: "Tableau de bord",
  chemin: "/",
  ordre: 0,
  Page,
};
