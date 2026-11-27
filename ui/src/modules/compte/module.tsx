// Module « Compte et licence » — Propriétaire : A10. Voir PROMPT_A10.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>Compte et licence</h1>
      <p className="texte-secondaire">Compte AMAZONE COMP, licence, sauvegarde en ligne.</p>
      <p className="texte-secondaire">Module en construction (agent A10).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "compte",
  libelle: "Compte et licence",
  chemin: "/compte",
  ordre: 90,
  Page,
};
