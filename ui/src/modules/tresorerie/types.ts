// Types du module « Trésorerie » — Propriétaire : A6. Champs en snake_case, identiques à Rust (C8).
export type TypeCompte = "BANQUE" | "CAISSE" | "MOBILE_MONEY";

export interface TresorerieCompte {
  id: number;
  type_compte: TypeCompte;
  libelle: string;
  banque_operateur?: string | null;
  /** Numéro de compte ou de téléphone masqué par Rust (4 derniers chiffres). */
  numero?: string | null;
  compte_comptable: string;
  journal: string;
  solde_ouverture: number;
  actif: boolean;
  date_ouverture: string;
}

export interface TresorerieSolde {
  compte: string;
  type_compte: TypeCompte;
  solde: number;
}

export interface TresorerieOperateur {
  id: number;
  nom: string;
  libelle?: string | null;
  actif: boolean;
}

export type CreerCompteParams = {
  type_compte: TypeCompte;
  libelle: string;
  banque_operateur?: string | null;
  numero?: string | null;
  solde_ouverture: number;
}
