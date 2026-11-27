// Types IPC du module Ventes — Propriétaire : A4. Champs snake_case identiques au Rust (C8).
// Montants transmis en entiers FCFA uniquement (règle R1).

/** Type de client (identique à `ac_ventes::clients::TypeClient`). */
export type TypeClient = "PARTICULIER" | "ENTREPRISE" | "ADMINISTRATION";

/** Informations saisies sur un client (`ac_ventes::clients::InformationsClient`). */
export type InformationsClient = {
  nom: string;
  type_client: TypeClient;
  ifu: string | null;
  rccm: string | null;
  adresse: string | null;
  telephone: string | null;
  courriel: string | null;
  retient_tva: boolean;
  exonere_aib: boolean;
  motif_exoneration_aib: string | null;
};

/** Client enregistré (`ac_ventes::clients::Client`). */
export type Client = {
  id: number;
  code: string;
  nom: string;
  type_client: TypeClient;
  ifu: string | null;
  rccm: string | null;
  adresse: string | null;
  telephone: string | null;
  courriel: string | null;
  retient_tva: boolean;
  exonere_aib: boolean;
  motif_exoneration_aib: string | null;
  actif: boolean;
  cree_le: string;
  modifie_le: string | null;
};
