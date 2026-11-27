// Contrat d'un module d'interface — Propriétaire : A3. Ne pas modifier sans ADR.
import type { ComponentType } from "react";

export type DeclarationModule = {
  /** Identifiant stable (ex. "ventes"). */
  id: string;
  /** Libellé du menu, en français. */
  libelle: string;
  /** Chemin de la route racine du module (ex. "/ventes"). Les sous-routes sont gérées par le module. */
  chemin: string;
  /** Ordre d'affichage dans le menu (croissant). */
  ordre: number;
  /** Écran principal (le module gère ses sous-pages avec <Routes> relatives). */
  Page: ComponentType;
};
