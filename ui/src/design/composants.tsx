// Composants du design system — Propriétaire : A3.
// Les modules métier utilisent uniquement ces composants (pas de bibliothèque UI externe).
import {
  useEffect,
  useId,
  useRef,
  useState,
  type ButtonHTMLAttributes,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
} from "react";

type ProprietesBouton = ButtonHTMLAttributes<HTMLButtonElement> & {
  variante?: "principal" | "secondaire";
};

export function Bouton({ variante = "secondaire", className, type = "button", ...reste }: ProprietesBouton) {
  const classes = ["bouton", variante === "principal" ? "bouton--principal" : "", className ?? ""]
    .filter(Boolean)
    .join(" ");
  return <button type={type} className={classes} {...reste} />;
}

type ProprietesChamp = {
  libelle: string;
  valeur: string;
  surChangement: (valeur: string) => void;
  aide?: string;
  erreur?: string;
  multiligne?: boolean;
  type?: "text" | "email" | "tel" | "password";
  obligatoire?: boolean;
  desactive?: boolean;
};

export function Champ({
  libelle,
  valeur,
  surChangement,
  aide,
  erreur,
  multiligne = false,
  type = "text",
  obligatoire = false,
  desactive = false,
}: ProprietesChamp) {
  const id = useId();
  const idAide = `${id}-aide`;
  const commun = {
    id,
    value: valeur,
    required: obligatoire,
    disabled: desactive,
    "aria-describedby": aide || erreur ? idAide : undefined,
    "aria-invalid": erreur ? true : undefined,
  };
  return (
    <div className="champ">
      <label htmlFor={id}>
        {libelle}
        {obligatoire ? " *" : ""}
      </label>
      {multiligne ? (
        <textarea {...commun} rows={5} onChange={(e) => surChangement(e.target.value)} />
      ) : (
        <input {...commun} type={type} onChange={(e) => surChangement(e.target.value)} />
      )}
      {erreur ? (
        <span id={idAide} className="champ__erreur" role="alert">
          {erreur}
        </span>
      ) : aide ? (
        <span id={idAide} className="champ__aide">
          {aide}
        </span>
      ) : null}
    </div>
  );
}

export function Carte({ titre, children }: { titre?: string; children: ReactNode }) {
  return (
    <section className="carte">
      {titre ? <h2>{titre}</h2> : null}
      {children}
    </section>
  );
}

export function Bandeau({
  children,
  type = "info",
}: {
  children: ReactNode;
  type?: "info" | "alerte" | "succes" | "erreur";
}) {
  const classe = type === "info" ? "" : `bandeau--${type}`;
  const role = type === "erreur" || type === "alerte" ? "alert" : "status";
  const nomAccessible = typeof children === "string" ? children : undefined;
  return (
    <div className={["bandeau", classe].filter(Boolean).join(" ")} role={role} aria-label={nomAccessible}>
      {children}
    </div>
  );
}

/** Affiche un montant FCFA (entier) au format français : 1 234 567 FCFA. */
export function formaterMontant(francs: number): string {
  if (!Number.isSafeInteger(francs)) {
    return "montant invalide";
  }
  return `${new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 0 }).format(francs)} FCFA`;
}

export function Montant({ francs }: { francs: number }) {
  return <span className="montant">{formaterMontant(francs)}</span>;
}

export type DirectionTri = "asc" | "desc";

export type TriTableau = {
  colonne: string;
  direction: DirectionTri;
};

export type ColonneTableau<Ligne> = {
  id: string;
  entete: string;
  cle?: keyof Ligne;
  contenu?: (ligne: Ligne) => ReactNode;
  format?: "montant";
  montant?: boolean;
  alignement?: "gauche" | "centre" | "droite";
  triable?: boolean;
};

export type ProprietesTableau<Ligne> = {
  colonnes: readonly ColonneTableau<Ligne>[];
  lignes: readonly Ligne[];
  /** Numéro de la page affichée, à partir de 1. */
  page: number;
  /** Nombre total de lignes, fourni par Rust pour la pagination. */
  total: number;
  /** Nombre de lignes par page, fourni par Rust. */
  surPage: number;
  tri?: TriTableau;
  libelle?: string;
  cleLigne?: (ligne: Ligne, index: number) => string | number;
  libelleLigne?: (ligne: Ligne) => string;
  surChangementPage?: (page: number) => void;
  /** Alias conservé pour les écrans qui préfèrent le nom court. */
  surPageChange?: (page: number) => void;
  surChangementTri?: (tri: TriTableau) => void;
  /** Alias conservé pour les écrans qui préfèrent le nom court. */
  surTriChange?: (tri: TriTableau) => void;
  surLigne?: (ligne: Ligne) => void;
  surLigneClick?: (ligne: Ligne) => void;
};

function valeurAffichable(valeur: unknown): ReactNode {
  if (valeur === null || valeur === undefined || typeof valeur === "boolean") {
    return null;
  }
  if (typeof valeur === "string" || typeof valeur === "number") {
    return String(valeur);
  }
  return null;
}

function nombreDePages(total: number, surPage: number): number {
  const lignesParPage = Math.max(1, surPage);
  return Math.max(1, Math.ceil(Math.max(0, total) / lignesParPage));
}

/**
 * Tableau générique : les données et l'ordre sont contrôlés par l'écran appelant.
 * Les boutons de pagination et de tri demandent une nouvelle page à Rust au moyen des callbacks.
 */
export function Tableau<Ligne>({
  colonnes,
  lignes,
  page,
  total,
  surPage,
  tri,
  libelle = "Tableau",
  cleLigne,
  libelleLigne,
  surChangementPage,
  surPageChange,
  surChangementTri,
  surTriChange,
  surLigne,
  surLigneClick,
}: ProprietesTableau<Ligne>) {
  const pages = nombreDePages(total, surPage);
  const pageActuelle = Math.min(pages, Math.max(1, page));
  const changerPage = surChangementPage ?? surPageChange;
  const changerTri = surChangementTri ?? surTriChange;
  const choisirLigne = surLigne ?? surLigneClick;

  const demanderTri = (colonne: ColonneTableau<Ligne>) => {
    if (!changerTri) {
      return;
    }
    const direction: DirectionTri = tri?.colonne === colonne.id && tri.direction === "asc" ? "desc" : "asc";
    changerTri({ colonne: colonne.id, direction });
  };

  const afficherCellule = (colonne: ColonneTableau<Ligne>, ligne: Ligne): ReactNode => {
    if (colonne.contenu) {
      return colonne.contenu(ligne);
    }
    const valeur = colonne.cle === undefined ? undefined : ligne[colonne.cle];
    const estMontant = colonne.format === "montant" || colonne.montant === true;
    if (estMontant && typeof valeur === "number" && Number.isSafeInteger(valeur)) {
      return <Montant francs={valeur} />;
    }
    return valeurAffichable(valeur);
  };

  return (
    <div className="tableau">
      <div className="tableau__conteneur" tabIndex={0}>
        <table aria-label={libelle}>
          <thead>
            <tr>
              {colonnes.map((colonne) => {
                const triActif = tri?.colonne === colonne.id;
                const ariaSort: "ascending" | "descending" | "none" = triActif
                  ? tri.direction === "asc"
                    ? "ascending"
                    : "descending"
                  : "none";
                const alignement = colonne.alignement ?? (colonne.format === "montant" || colonne.montant ? "droite" : "gauche");
                return (
                  <th key={colonne.id} scope="col" aria-sort={ariaSort} className={`tableau__cellule--${alignement}`}>
                    {colonne.triable === false ? (
                      colonne.entete
                    ) : (
                      <button
                        type="button"
                        className="tableau__tri"
                        aria-label={`Trier par ${colonne.entete}`}
                        onClick={() => demanderTri(colonne)}
                      >
                        <span>{colonne.entete}</span>
                        <span className="tableau__indicateur-tri" aria-hidden="true">
                          {triActif ? (tri.direction === "asc" ? "↑" : "↓") : "↕"}
                        </span>
                      </button>
                    )}
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody>
            {lignes.length === 0 ? (
              <tr>
                <td colSpan={Math.max(1, colonnes.length)}>
                  <EtatVide titre="Aucun résultat" description="Aucune ligne ne correspond à cette page." />
                </td>
              </tr>
            ) : (
              lignes.map((ligne, index) => {
                const cle = cleLigne ? cleLigne(ligne, index) : index;
                const alignements = colonnes.map((colonne) => {
                  const alignement = colonne.alignement ?? (colonne.format === "montant" || colonne.montant ? "droite" : "gauche");
                  return `tableau__cellule--${alignement}`;
                });
                return (
                  <tr
                    key={cle}
                    className={choisirLigne ? "tableau__ligne--cliquable" : undefined}
                    tabIndex={choisirLigne ? 0 : undefined}
                    aria-label={libelleLigne ? libelleLigne(ligne) : undefined}
                    onClick={choisirLigne ? () => choisirLigne(ligne) : undefined}
                    onKeyDown={
                      choisirLigne
                        ? (event) => {
                            if (event.key === "Enter" || event.key === " ") {
                              event.preventDefault();
                              choisirLigne(ligne);
                            }
                          }
                        : undefined
                    }
                  >
                    {colonnes.map((colonne, colonneIndex) => (
                      <td key={colonne.id} className={alignements[colonneIndex]}>
                        {afficherCellule(colonne, ligne)}
                      </td>
                    ))}
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
      <nav className="tableau__pagination" aria-label="Pagination du tableau">
        <span aria-live="polite">
          Page {pageActuelle} sur {pages}
        </span>
        <div className="tableau__pagination-actions">
          <Bouton
            aria-label="Première page"
            disabled={pageActuelle <= 1 || !changerPage}
            onClick={() => changerPage?.(1)}
          >
            Première
          </Bouton>
          <Bouton
            aria-label="Page précédente"
            disabled={pageActuelle <= 1 || !changerPage}
            onClick={() => changerPage?.(pageActuelle - 1)}
          >
            Précédente
          </Bouton>
          <Bouton
            aria-label="Page suivante"
            disabled={pageActuelle >= pages || !changerPage}
            onClick={() => changerPage?.(pageActuelle + 1)}
          >
            Suivante
          </Bouton>
          <Bouton
            aria-label="Dernière page"
            disabled={pageActuelle >= pages || !changerPage}
            onClick={() => changerPage?.(pages)}
          >
            Dernière
          </Bouton>
        </div>
      </nav>
    </div>
  );
}

function formaterSaisieMontant(valeur: number | undefined): string {
  if (valeur === undefined || !Number.isSafeInteger(valeur)) {
    return "";
  }
  return new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 0 }).format(valeur);
}

type ResultatLectureMontant = number | null | "invalide" | "negatif-interdit" | "trop-grand";

function lireMontant(texte: string, negatifAutorise: boolean): ResultatLectureMontant {
  const compact = texte.trim().replace(/\s/g, "");
  if (compact === "") {
    return null;
  }
  if (!/^-?\d+$/.test(compact)) {
    return "invalide";
  }
  if (compact.startsWith("-") && !negatifAutorise) {
    return "negatif-interdit";
  }
  const nombre = Number(compact);
  return Number.isSafeInteger(nombre) ? nombre : "trop-grand";
}

type ProprietesChampMontant = {
  libelle: string;
  valeur?: number;
  surChangement: (valeur: number) => void;
  negatifAutorise?: boolean;
  aide?: string;
  erreur?: string;
  obligatoire?: boolean;
  desactive?: boolean;
};

/** Champ de saisie d'un montant entier en FCFA, sans décimale ni lettre. */
export function ChampMontant({
  libelle,
  valeur,
  surChangement,
  negatifAutorise = false,
  aide,
  erreur,
  obligatoire = false,
  desactive = false,
}: ProprietesChampMontant) {
  const id = useId();
  const idAide = `${id}-aide`;
  const [texteSaisi, setTexteSaisi] = useState<string>();
  const [erreurLocale, setErreurLocale] = useState<string>();
  const messageErreur = erreurLocale ?? erreur;
  const texte = erreurLocale ? texteSaisi ?? formaterSaisieMontant(valeur) : formaterSaisieMontant(valeur);

  const traiterChangement = (nouveauTexte: string) => {
    setTexteSaisi(nouveauTexte);
    const texteSaisi = nouveauTexte;
    const resultat = lireMontant(texteSaisi, negatifAutorise);
    if (resultat === null) {
      setErreurLocale(undefined);
      return;
    }
    if (typeof resultat === "string") {
      const messages: Record<Exclude<ResultatLectureMontant, number | null>, string> = {
        invalide: "Saisissez uniquement un nombre entier, sans lettre ni décimale.",
        "negatif-interdit": "Un montant négatif n'est pas autorisé ici.",
        "trop-grand": "Le montant est trop grand pour être enregistré.",
      };
      setErreurLocale(messages[resultat]);
      return;
    }
    setErreurLocale(undefined);
    setTexteSaisi(undefined);
    surChangement(resultat);
  };

  return (
    <div className="champ champ-montant">
      <label htmlFor={id}>
        {libelle}
        {obligatoire ? " *" : ""}
      </label>
      <div className="champ-montant__controle">
        <input
          id={id}
          type="text"
          inputMode="numeric"
          value={texte}
          required={obligatoire}
          disabled={desactive}
          aria-describedby={aide || messageErreur ? idAide : undefined}
          aria-invalid={messageErreur ? true : undefined}
          onChange={(event) => traiterChangement(event.currentTarget.value)}
        />
        <span aria-hidden="true">FCFA</span>
      </div>
      {messageErreur ? (
        <span id={idAide} className="champ__erreur" role="alert">
          {messageErreur}
        </span>
      ) : aide ? (
        <span id={idAide} className="champ__aide">
          {aide}
        </span>
      ) : null}
    </div>
  );
}

const mois = [
  "janvier",
  "février",
  "mars",
  "avril",
  "mai",
  "juin",
  "juillet",
  "août",
  "septembre",
  "octobre",
  "novembre",
  "décembre",
] as const;
const joursCourts = ["L", "M", "M", "J", "V", "S", "D"] as const;

function dateDepuisTexte(texte: string): Date | undefined {
  const morceaux = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec(texte);
  if (!morceaux) {
    return undefined;
  }
  const jour = Number(morceaux[1]);
  const moisIndex = Number(morceaux[2]) - 1;
  const annee = Number(morceaux[3]);
  const date = new Date(annee, moisIndex, jour);
  return date.getFullYear() === annee && date.getMonth() === moisIndex && date.getDate() === jour ? date : undefined;
}

function texteDepuisDate(date: Date): string {
  const jour = String(date.getDate()).padStart(2, "0");
  const moisIndex = String(date.getMonth() + 1).padStart(2, "0");
  return `${jour}/${moisIndex}/${date.getFullYear()}`;
}

type ProprietesChampDate = {
  libelle: string;
  valeur?: string;
  surChangement: (valeur: string) => void;
  aide?: string;
  erreur?: string;
  obligatoire?: boolean;
  desactive?: boolean;
};

/** Champ de date au format JJ/MM/AAAA avec un calendrier utilisable au clavier. */
export function ChampDate({
  libelle,
  valeur = "",
  surChangement,
  aide,
  erreur,
  obligatoire = false,
  desactive = false,
}: ProprietesChampDate) {
  const id = useId();
  const idAide = `${id}-aide`;
  const idCalendrier = `${id}-calendrier`;
  const dateValeur = dateDepuisTexte(valeur);
  const [texteSaisi, setTexteSaisi] = useState<string>();
  const texte = texteSaisi ?? valeur;
  const [erreurLocale, setErreurLocale] = useState<string>();
  const [ouvert, setOuvert] = useState(false);
  const [moisAffiche, setMoisAffiche] = useState<Date>(() => {
    const aujourdHui = new Date();
    const dateInitiale = dateValeur ?? aujourdHui;
    return new Date(dateInitiale.getFullYear(), dateInitiale.getMonth(), 1);
  });
  const messageErreur = erreurLocale ?? erreur;

  const traiterTexte = (nouveauTexte: string) => {
    setTexteSaisi(nouveauTexte);
    if (!/^[0-9/]*$/.test(nouveauTexte)) {
      setErreurLocale("Utilisez le format JJ/MM/AAAA.");
      return;
    }
    setErreurLocale(undefined);
    surChangement(nouveauTexte);
  };

  const choisirDate = (jour: number) => {
    const date = new Date(moisAffiche.getFullYear(), moisAffiche.getMonth(), jour);
    const nouvelleValeur = texteDepuisDate(date);
    setTexteSaisi(undefined);
    setErreurLocale(undefined);
    surChangement(nouvelleValeur);
    setOuvert(false);
  };

  const annee = moisAffiche.getFullYear();
  const moisIndex = moisAffiche.getMonth();
  const decalage = (new Date(annee, moisIndex, 1).getDay() + 6) % 7;
  const nombreJours = new Date(annee, moisIndex + 1, 0).getDate();
  const cellules: Array<number | null> = [];
  for (let index = 0; index < decalage; index += 1) {
    cellules.push(null);
  }
  for (let jour = 1; jour <= nombreJours; jour += 1) {
    cellules.push(jour);
  }

  return (
    <div className="champ champ-date">
      <label htmlFor={id}>
        {libelle}
        {obligatoire ? " *" : ""}
      </label>
      <div className="champ-date__controle">
        <input
          id={id}
          type="text"
          inputMode="numeric"
          placeholder="JJ/MM/AAAA"
          autoComplete="off"
          value={texte}
          required={obligatoire}
          disabled={desactive}
          aria-describedby={aide || messageErreur ? idAide : undefined}
          aria-invalid={messageErreur ? true : undefined}
          onChange={(event) => traiterTexte(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              setOuvert(false);
            }
          }}
        />
        <Bouton
          className="champ-date__ouvrir"
          aria-label="Ouvrir le calendrier"
          aria-expanded={ouvert}
          aria-controls={idCalendrier}
          disabled={desactive}
          onClick={() => setOuvert((etat) => !etat)}
        >
          Calendrier
        </Bouton>
      </div>
      {ouvert ? (
        <div
          id={idCalendrier}
          className="champ-date__calendrier"
          role="dialog"
          aria-label="Calendrier"
          onKeyDown={(event: ReactKeyboardEvent<HTMLDivElement>) => {
            if (event.key === "Escape") {
              setOuvert(false);
            }
          }}
        >
          <div className="champ-date__entete">
            <Bouton
              aria-label="Mois précédent"
              onClick={() => setMoisAffiche(new Date(annee, moisIndex - 1, 1))}
            >
              ←
            </Bouton>
            <strong>
              {mois[moisIndex]} {annee}
            </strong>
            <Bouton aria-label="Mois suivant" onClick={() => setMoisAffiche(new Date(annee, moisIndex + 1, 1))}>
              →
            </Bouton>
          </div>
          <div className="champ-date__jours" role="grid" aria-label={`Calendrier ${mois[moisIndex]} ${annee}`}>
            {joursCourts.map((jour, index) => (
              <span key={`${jour}-${index}`} role="columnheader" aria-label={jour}>
                {jour}
              </span>
            ))}
            {cellules.map((jour, index) =>
              jour === null ? (
                <span key={`vide-${index}`} aria-hidden="true" />
              ) : (
                <button
                  key={jour}
                  type="button"
                  role="gridcell"
                  aria-label={`${jour} ${mois[moisIndex]} ${annee}`}
                  aria-current={dateValeur?.getFullYear() === annee && dateValeur.getMonth() === moisIndex && dateValeur.getDate() === jour ? "date" : undefined}
                  onClick={() => choisirDate(jour)}
                >
                  {jour}
                </button>
              ),
            )}
          </div>
        </div>
      ) : null}
      {messageErreur ? (
        <span id={idAide} className="champ__erreur" role="alert">
          {messageErreur}
        </span>
      ) : aide ? (
        <span id={idAide} className="champ__aide">
          {aide}
        </span>
      ) : null}
    </div>
  );
}

export type OptionSelecteur = {
  valeur: string;
  libelle: string;
};

type ProprietesSelecteur = {
  libelle: string;
  options: readonly OptionSelecteur[];
  valeur?: string;
  surChangement: (valeur: string) => void;
  aide?: string;
  erreur?: string;
  obligatoire?: boolean;
  desactive?: boolean;
  placeholder?: string;
};

/** Selecteur avec champ de recherche et navigation par flèches. */
export function Selecteur({
  libelle,
  options,
  valeur = "",
  surChangement,
  aide,
  erreur,
  obligatoire = false,
  desactive = false,
  placeholder = "Rechercher ou choisir",
}: ProprietesSelecteur) {
  const id = useId();
  const idAide = `${id}-aide`;
  const idListe = `${id}-liste`;
  const [rechercheLocale, setRechercheLocale] = useState<string>();
  const [ouvert, setOuvert] = useState(false);
  const [indexSurvole, setIndexSurvole] = useState(-1);
  const optionChoisie = options.find((option) => option.valeur === valeur);
  const libelleChoisi = optionChoisie?.libelle ?? "";
  const recherche = rechercheLocale ?? libelleChoisi;
  const optionsFiltrees = options.filter((option) => option.libelle.toLocaleLowerCase("fr-FR").includes(recherche.trim().toLocaleLowerCase("fr-FR")));

  const choisir = (option: OptionSelecteur) => {
    setRechercheLocale(option.libelle);
    setOuvert(false);
    setIndexSurvole(-1);
    surChangement(option.valeur);
  };

  const traiterTouche = (event: ReactKeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Escape") {
      event.preventDefault();
      setRechercheLocale(undefined);
      setOuvert(false);
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setOuvert(true);
      setIndexSurvole((index) => Math.min(optionsFiltrees.length - 1, index + 1));
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setIndexSurvole((index) => Math.max(0, index - 1));
      return;
    }
    if (event.key === "Enter" && ouvert) {
      event.preventDefault();
      const option = optionsFiltrees[indexSurvole] ?? optionsFiltrees[0];
      if (option) {
        choisir(option);
      }
    }
  };

  return (
    <div className="champ selecteur">
      <label htmlFor={id}>
        {libelle}
        {obligatoire ? " *" : ""}
      </label>
      <input
        id={id}
        type="text"
        role="combobox"
        value={recherche}
        placeholder={placeholder}
        autoComplete="off"
        disabled={desactive}
        required={obligatoire}
        aria-autocomplete="list"
        aria-expanded={ouvert}
        aria-controls={idListe}
        aria-activedescendant={indexSurvole >= 0 ? `${id}-option-${indexSurvole}` : undefined}
        aria-describedby={aide || erreur ? idAide : undefined}
        aria-invalid={erreur ? true : undefined}
        onFocus={() => {
          setOuvert(true);
          setRechercheLocale(recherche || libelleChoisi);
        }}
        onChange={(event) => {
          setRechercheLocale(event.currentTarget.value);
          setOuvert(true);
          setIndexSurvole(0);
        }}
        onKeyDown={traiterTouche}
        onBlur={() => setOuvert(false)}
      />
      {ouvert ? (
        <ul id={idListe} className="selecteur__liste" role="listbox" aria-label={`Choix pour ${libelle}`}>
          {optionsFiltrees.length === 0 ? (
            <li className="selecteur__vide">Aucun choix trouvé</li>
          ) : (
            optionsFiltrees.map((option, index) => (
              <li
                id={`${id}-option-${index}`}
                key={option.valeur}
                role="option"
                aria-selected={option.valeur === valeur}
                className={index === indexSurvole ? "selecteur__option--active" : undefined}
                onMouseDown={(event) => {
                  event.preventDefault();
                  choisir(option);
                }}
              >
                {option.libelle}
              </li>
            ))
          )}
        </ul>
      ) : null}
      {erreur ? (
        <span id={idAide} className="champ__erreur" role="alert">
          {erreur}
        </span>
      ) : aide ? (
        <span id={idAide} className="champ__aide">
          {aide}
        </span>
      ) : null}
    </div>
  );
}

type ProprietesCaseACocher = {
  libelle: string;
  cochee: boolean;
  surChangement: (cochee: boolean) => void;
  aide?: string;
  erreur?: string;
  obligatoire?: boolean;
  desactive?: boolean;
};

export function CaseACocher({
  libelle,
  cochee,
  surChangement,
  aide,
  erreur,
  obligatoire = false,
  desactive = false,
}: ProprietesCaseACocher) {
  const id = useId();
  const idAide = `${id}-aide`;
  return (
    <div className="case-a-cocher">
      <div className="case-a-cocher__ligne">
        <input
          id={id}
          type="checkbox"
          checked={cochee}
          required={obligatoire}
          disabled={desactive}
          aria-describedby={aide || erreur ? idAide : undefined}
          aria-invalid={erreur ? true : undefined}
          onChange={(event) => surChangement(event.currentTarget.checked)}
        />
        <label htmlFor={id}>
          {libelle}
          {obligatoire ? " *" : ""}
        </label>
      </div>
      {erreur ? (
        <span id={idAide} className="champ__erreur" role="alert">
          {erreur}
        </span>
      ) : aide ? (
        <span id={idAide} className="champ__aide">
          {aide}
        </span>
      ) : null}
    </div>
  );
}

type ProprietesDialogue = {
  ouvert: boolean;
  titre: string;
  children: ReactNode;
  surFermer: () => void;
  surConfirmer?: () => void;
  libelleConfirmation?: string;
  libelleAnnulation?: string;
};

/** Dialogue modal avec retour du focus, piège de tabulation et fermeture par Échap. */
export function Dialogue({
  ouvert,
  titre,
  children,
  surFermer,
  surConfirmer,
  libelleConfirmation = "Confirmer",
  libelleAnnulation = "Annuler",
}: ProprietesDialogue) {
  const id = useId();
  const conteneurRef = useRef<HTMLDivElement>(null);
  const focusAvantRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!ouvert) {
      return undefined;
    }
    const elementActif = document.activeElement;
    focusAvantRef.current = elementActif instanceof HTMLElement ? elementActif : null;
    const conteneur = conteneurRef.current;
    const selecteurFocusable =
      "button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled]), [href], [tabindex]:not([tabindex=\"-1\"])";
    const premier = conteneur?.querySelector<HTMLElement>(selecteurFocusable);
    (premier ?? conteneur)?.focus();

    const gererTouche = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        surFermer();
        return;
      }
      if (event.key !== "Tab" || !conteneur) {
        return;
      }
      const elements = Array.from(conteneur.querySelectorAll<HTMLElement>(selecteurFocusable));
      if (elements.length === 0) {
        event.preventDefault();
        conteneur.focus();
        return;
      }
      const premierElement = elements[0];
      const dernierElement = elements[elements.length - 1];
      if (event.shiftKey && document.activeElement === premierElement) {
        event.preventDefault();
        dernierElement?.focus();
      } else if (!event.shiftKey && document.activeElement === dernierElement) {
        event.preventDefault();
        premierElement?.focus();
      }
    };

    document.addEventListener("keydown", gererTouche);
    return () => {
      document.removeEventListener("keydown", gererTouche);
      focusAvantRef.current?.focus();
      focusAvantRef.current = null;
    };
  }, [ouvert, surFermer]);

  if (!ouvert) {
    return null;
  }

  return (
    <div className="dialogue__fond" role="presentation">
      <div
        ref={conteneurRef}
        className="dialogue"
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${id}-titre`}
        tabIndex={-1}
      >
        <div className="dialogue__entete">
          <h2 id={`${id}-titre`}>{titre}</h2>
          <Bouton aria-label="Fermer" onClick={surFermer}>
            ×
          </Bouton>
        </div>
        <div className="dialogue__contenu">{children}</div>
        <div className="dialogue__actions">
          <Bouton onClick={surFermer}>{libelleAnnulation}</Bouton>
          {surConfirmer ? (
            <Bouton variante="principal" onClick={surConfirmer}>
              {libelleConfirmation}
            </Bouton>
          ) : null}
        </div>
      </div>
    </div>
  );
}

export type EtapeAssistant = {
  id?: string;
  titre: string;
  contenu: ReactNode;
  resume?: ReactNode;
};

type ProprietesAssistant = {
  etapes: readonly EtapeAssistant[];
  resumeFinal?: ReactNode;
  etape?: number;
  etapeInitiale?: number;
  surChangementEtape?: (etape: number) => void;
  surValider?: () => void;
  suivantDesactive?: boolean;
};

/** Assistant pas à pas : les étapes précédentes restent accessibles et la dernière peut présenter un résumé. */
export function Assistant({
  etapes,
  resumeFinal,
  etape,
  etapeInitiale = 0,
  surChangementEtape,
  surValider,
  suivantDesactive = false,
}: ProprietesAssistant) {
  const [etapeInterne, setEtapeInterne] = useState(etapeInitiale);
  if (etapes.length === 0) {
    return <EtatVide titre="Aucune étape" description="Cet assistant ne contient pas encore d'étape." />;
  }

  const indexEtape = Math.min(etapes.length - 1, Math.max(0, etape ?? etapeInterne));
  const etapeCourante = etapes[indexEtape];
  const estDerniere = indexEtape === etapes.length - 1;
  const resume = etapeCourante?.resume ?? (estDerniere ? resumeFinal : undefined);

  const allerA = (nouvelIndex: number) => {
    const index = Math.min(etapes.length - 1, Math.max(0, nouvelIndex));
    setEtapeInterne(index);
    surChangementEtape?.(index);
  };

  return (
    <section className="assistant" aria-label="Assistant">
      <ol className="assistant__etapes" aria-label="Étapes de l'assistant">
        {etapes.map((etapeItem, index) => (
          <li key={etapeItem.id ?? `${etapeItem.titre}-${index}`}>
            <button
              type="button"
              className={index === indexEtape ? "assistant__etape--active" : undefined}
              aria-current={index === indexEtape ? "step" : undefined}
              disabled={index > indexEtape}
              onClick={() => allerA(index)}
              onKeyDown={(event) => {
                if (event.key === "ArrowLeft" && index > 0) {
                  event.preventDefault();
                  allerA(index - 1);
                } else if (event.key === "ArrowRight" && index < etapes.length - 1) {
                  event.preventDefault();
                  allerA(index + 1);
                }
              }}
            >
              <span aria-hidden="true">{index + 1}</span>
              {etapeItem.titre}
            </button>
          </li>
        ))}
      </ol>
      <div className="assistant__corps" aria-live="polite">
        <h2>{etapeCourante?.titre}</h2>
        <div className="assistant__contenu">{etapeCourante?.contenu}</div>
        {resume ? (
          <section className="assistant__resume" aria-labelledby={`${etapeCourante?.id ?? "assistant"}-resume`}>
            <h3 id={`${etapeCourante?.id ?? "assistant"}-resume`}>Résumé avant validation</h3>
            {resume}
          </section>
        ) : null}
      </div>
      <div className="assistant__actions">
        <Bouton disabled={indexEtape === 0} onClick={() => allerA(indexEtape - 1)}>
          Précédent
        </Bouton>
        {estDerniere ? (
          <Bouton variante="principal" disabled={!surValider} onClick={surValider}>
            Valider
          </Bouton>
        ) : (
          <Bouton variante="principal" disabled={suivantDesactive} onClick={() => allerA(indexEtape + 1)}>
            Suivant
          </Bouton>
        )}
      </div>
    </section>
  );
}

export type Onglet = {
  id: string;
  libelle: string;
  contenu: ReactNode;
};

type ProprietesOnglets = {
  onglets: readonly Onglet[];
  valeur?: string;
  surChangement?: (id: string) => void;
};

/** Onglets WAI-ARIA avec navigation gauche/droite, Début et Fin. */
export function Onglets({ onglets, valeur, surChangement }: ProprietesOnglets) {
  const id = useId();
  const [actifInterne, setActifInterne] = useState(onglets[0]?.id ?? "");
  const boutons = useRef<Array<HTMLButtonElement | null>>([]);
  const idDemande = valeur ?? actifInterne;
  const actif = onglets.some((onglet) => onglet.id === idDemande) ? idDemande : onglets[0]?.id ?? "";
  const indexActif = onglets.findIndex((onglet) => onglet.id === actif);
  const ongletActif = onglets[indexActif];

  const selectionner = (nouvelId: string) => {
    setActifInterne(nouvelId);
    surChangement?.(nouvelId);
  };

  const deplacer = (index: number) => {
    const onglet = onglets[index];
    if (!onglet) {
      return;
    }
    selectionner(onglet.id);
    boutons.current[index]?.focus();
  };

  if (onglets.length === 0) {
    return null;
  }

  return (
    <div className="onglets">
      <div className="onglets__liste" role="tablist" aria-label="Sections">
        {onglets.map((onglet, index) => (
          <button
            key={onglet.id}
            ref={(element) => {
              boutons.current[index] = element;
            }}
            id={`${id}-onglet-${onglet.id}`}
            type="button"
            role="tab"
            aria-selected={onglet.id === actif}
            aria-controls={`${id}-panneau-${onglet.id}`}
            tabIndex={onglet.id === actif ? 0 : -1}
            onClick={() => selectionner(onglet.id)}
            onKeyDown={(event) => {
              if (event.key === "ArrowRight") {
                event.preventDefault();
                deplacer((index + 1) % onglets.length);
              } else if (event.key === "ArrowLeft") {
                event.preventDefault();
                deplacer((index - 1 + onglets.length) % onglets.length);
              } else if (event.key === "Home") {
                event.preventDefault();
                deplacer(0);
              } else if (event.key === "End") {
                event.preventDefault();
                deplacer(onglets.length - 1);
              }
            }}
          >
            {onglet.libelle}
          </button>
        ))}
      </div>
      {ongletActif ? (
        <div
          id={`${id}-panneau-${ongletActif.id}`}
          className="onglets__panneau"
          role="tabpanel"
          tabIndex={0}
          aria-labelledby={`${id}-onglet-${ongletActif.id}`}
        >
          {ongletActif.contenu}
        </div>
      ) : null}
    </div>
  );
}

type TypeNotification = "info" | "succes" | "alerte" | "erreur";

type ProprietesNotification = {
  message?: ReactNode;
  children?: ReactNode;
  type?: TypeNotification;
  visible?: boolean;
  surFermer?: () => void;
  duree?: number;
};

/** Notification brève : elle reste accessible avec un rôle d'état ou d'alerte et peut se fermer au clavier. */
export function Notification({
  message,
  children,
  type = "info",
  visible = true,
  surFermer,
  duree = 0,
}: ProprietesNotification) {
  useEffect(() => {
    if (!visible || !surFermer || duree <= 0) {
      return undefined;
    }
    const minuteur = window.setTimeout(surFermer, duree);
    return () => window.clearTimeout(minuteur);
  }, [duree, surFermer, visible]);

  if (!visible) {
    return null;
  }

  const contenu = message === undefined ? children : message;
  const nomAccessible = typeof contenu === "string" ? contenu : undefined;
  return (
    <div
      className={`notification notification--${type}`}
      role={type === "erreur" || type === "alerte" ? "alert" : "status"}
      aria-label={nomAccessible}
      aria-live={type === "erreur" || type === "alerte" ? "assertive" : "polite"}
    >
      <div className="notification__message">{contenu}</div>
      {surFermer ? (
        <Bouton aria-label="Fermer la notification" onClick={surFermer}>
          Fermer
        </Bouton>
      ) : null}
    </div>
  );
}

type ProprietesEtatVide = {
  titre: string;
  description?: string;
  action?: ReactNode;
};

export function EtatVide({ titre, description, action }: ProprietesEtatVide) {
  return (
    <section className="etat-vide" aria-label={titre} role="status">
      <h2>{titre}</h2>
      {description ? <p>{description}</p> : null}
      {action ? <div className="etat-vide__action">{action}</div> : null}
    </section>
  );
}

export function Chargement({ texte = "Chargement…" }: { texte?: string }) {
  return (
    <div className="chargement" role="status" aria-label={texte} aria-live="polite" aria-busy="true">
      <span className="chargement__indicateur" aria-hidden="true" />
      <span>{texte}</span>
    </div>
  );
}

type ProprietesMessageErreur = {
  erreur: string | { readonly message: string };
  actionProposee?: string;
  surRéessayer?: () => void;
  action?: ReactNode;
};

/** Erreur lisible pour l'utilisateur, avec l'action proposée par la commande Rust. */
export function MessageErreur({ erreur, actionProposee, surRéessayer, action }: ProprietesMessageErreur) {
  const id = useId();
  const message = typeof erreur === "string" ? erreur : erreur.message;
  return (
    <div className="message-erreur" role="alert" aria-labelledby={`${id}-titre`}>
      <h2 id={`${id}-titre`}>Impossible de terminer cette action</h2>
      <p>{message}</p>
      <p className="texte-secondaire">
        {actionProposee ?? "Réessayez. Si le problème persiste, contactez le support."}
      </p>
      {action ? <div className="message-erreur__action">{action}</div> : null}
      {surRéessayer ? (
        <Bouton variante="secondaire" onClick={surRéessayer}>
          Réessayer
        </Bouton>
      ) : null}
    </div>
  );
}
