// Coordonnées du propriétaire affichées dans « À propos ».
// ⚠️ Seul le chef de projet modifie ce fichier, avec les informations fournies par le propriétaire.
// Photo : déposer le fichier dans ui/src/pages/a-propos/photo.jpg puis renseigner `photo`.

export type Proprietaire = {
  nom: string;
  photo: string | null;
  contact: string;
  linkedin: string;
  github: string;
  autres: string;
};

export const A_COMPLETER = "A COMPLETER";

export const proprietaire: Proprietaire = {
  nom: A_COMPLETER,
  photo: null,
  contact: A_COMPLETER,
  linkedin: A_COMPLETER,
  github: A_COMPLETER,
  autres: A_COMPLETER,
};
