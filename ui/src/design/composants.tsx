// Composants de base du design system — Propriétaire : A3.
// Les modules métier utilisent UNIQUEMENT ces composants (pas de bibliothèque UI externe).
import { useId, type ButtonHTMLAttributes, type ReactNode } from "react";

type ProprietesBouton = ButtonHTMLAttributes<HTMLButtonElement> & {
  variante?: "principal" | "secondaire";
};

export function Bouton({ variante = "secondaire", className, ...reste }: ProprietesBouton) {
  const classes = ["bouton", variante === "principal" ? "bouton--principal" : "", className ?? ""]
    .filter(Boolean)
    .join(" ");
  return <button type="button" className={classes} {...reste} />;
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
}: ProprietesChamp) {
  const id = useId();
  const idAide = `${id}-aide`;
  const commun = {
    id,
    value: valeur,
    required: obligatoire,
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
  type?: "info" | "alerte";
}) {
  return (
    <div className={type === "alerte" ? "bandeau bandeau--alerte" : "bandeau"} role="status">
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
