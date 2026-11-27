// Registre des modules — Propriétaire : A3. Un module = une ligne. Chaque agent ne modifie que son dossier.
import { declaration as m_tableau_de_bord } from "./tableau-de-bord/module";
import { declaration as m_ventes } from "./ventes/module";
import { declaration as m_achats } from "./achats/module";
import { declaration as m_tresorerie } from "./tresorerie/module";
import { declaration as m_paie } from "./paie/module";
import { declaration as m_fiscal } from "./fiscal/module";
import { declaration as m_comptabilite } from "./comptabilite/module";
import { declaration as m_etats } from "./etats/module";
import { declaration as m_compte } from "./compte/module";
import { declaration as m_assistance } from "./assistance/module";
import type { DeclarationModule } from "./types";

export const modules: readonly DeclarationModule[] = [
  m_tableau_de_bord,
  m_ventes,
  m_achats,
  m_tresorerie,
  m_paie,
  m_fiscal,
  m_comptabilite,
  m_etats,
  m_compte,
  m_assistance,
].sort((a, b) => a.ordre - b.ordre);
