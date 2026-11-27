// Module « Fiscalité » — Propriétaire : A8. Voir PROMPT_A8.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>Fiscalité</h1>
      <p className="texte-secondaire">TVA, AIB, IS/IBA, TPS, calendrier, déclarations à déposer sur e-services.</p>
      <p className="texte-secondaire">Module en construction (agent A8).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "fiscal",
  libelle: "Fiscalité",
  chemin: "/fiscal",
  ordre: 50,
  Page,
};
