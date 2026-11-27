// Module « Trésorerie » — Propriétaire : A6. Voir PROMPT_A6.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>Trésorerie</h1>
      <p className="texte-secondaire">Banques, caisse, Mobile Money, rapprochement, import de relevés.</p>
      <p className="texte-secondaire">Module en construction (agent A6).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "tresorerie",
  libelle: "Trésorerie",
  chemin: "/tresorerie",
  ordre: 30,
  Page,
};
