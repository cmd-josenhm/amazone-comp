// Module « États financiers » — Propriétaire : A9. Voir PROMPT_A9.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>États financiers</h1>
      <p className="texte-secondaire">Balance, grand livre, bilan, compte de résultat, TFT, DSF, reprise de données.</p>
      <p className="texte-secondaire">Module en construction (agent A9).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "etats",
  libelle: "États financiers",
  chemin: "/etats",
  ordre: 70,
  Page,
};
