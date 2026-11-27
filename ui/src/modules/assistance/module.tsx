// Module « Assistance » — Propriétaire : A10. Voir PROMPT_A10.md et docs/adr/0010-console-admin-et-assistance.md.
import type { DeclarationModule } from "../types";

function Page() {
  return (
    <div>
      <h1>Assistance</h1>
      <p className="texte-secondaire">
        Posez vos questions à l&apos;équipe AMAZONE COMP et suivez les réponses ici.
      </p>
      <p className="texte-secondaire">
        Ne communiquez jamais votre mot de passe ni votre clé de secours.
      </p>
      <p className="texte-secondaire">Module en construction (agent A10).</p>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "assistance",
  libelle: "Assistance",
  chemin: "/assistance",
  ordre: 95,
  Page,
};
