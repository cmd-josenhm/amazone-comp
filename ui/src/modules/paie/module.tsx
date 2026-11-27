// Module « Paie » — Propriétaire : A7. Voir PROMPT_A7.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>Paie</h1>
      <p className="texte-secondaire">Salariés, bulletins, ITS, VPS, CNSS.</p>
      <p className="texte-secondaire">Module en construction (agent A7).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "paie",
  libelle: "Paie",
  chemin: "/paie",
  ordre: 40,
  Page,
};
