// Pont vers le cœur Rust — Propriétaire : A3.
// Tous les appels passent par `appeler` : jamais d'import direct de @tauri-apps/api dans les modules.
import { invoke } from "@tauri-apps/api/core";

/** Erreur renvoyée par une commande Rust (voir src-tauri/src/commandes/mod.rs). */
export type ErreurCommande = { code: string; message: string };

export class ErreurAppel extends Error {
  readonly code: string;
  constructor(e: ErreurCommande) {
    super(e.message);
    this.code = e.code;
  }
}

function estErreurCommande(e: unknown): e is ErreurCommande {
  return typeof e === "object" && e !== null && "code" in e && "message" in e;
}

/** Appelle une commande `module_action`. Les arguments sont convertis en JSON par Tauri. */
export async function appeler<T>(commande: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(commande, args);
  } catch (e) {
    if (estErreurCommande(e)) {
      throw new ErreurAppel(e);
    }
    throw new ErreurAppel({
      code: "INCONNUE",
      message: "Une erreur inattendue est survenue. Réessayez, puis contactez le support si elle persiste.",
    });
  }
}

/** Vrai si l'interface tourne dans l'application (et non dans un navigateur de développement). */
export function dansTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
