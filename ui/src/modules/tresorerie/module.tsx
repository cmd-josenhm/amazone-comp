// Module « Trésorerie » — Propriétaire : A6. Voir PROMPT_A6.md.
import { useCallback, useEffect, useState } from "react";
import type { FormEvent } from "react";
import { Bouton, Bandeau, Carte, Champ, formaterMontant } from "../../design/composants";
import { appeler, ErreurAppel } from "../../lib/ipc";
import type { DeclarationModule } from "../types";
import type { CreerCompteParams, TresorerieCompte, TresorerieSolde, TypeCompte } from "./types";

function libelleTypeCompte(t: TypeCompte): string {
  if (t === "BANQUE") return "Banque";
  if (t === "CAISSE") return "Caisse";
  return "Mobile Money";
}

function Page() {
  const [comptes, setComptes] = useState<TresorerieCompte[]>([]);
  const [soldes, setSoldes] = useState<TresorerieSolde[]>([]);
  const [chargement, setChargement] = useState(true);
  const [erreur, setErreur] = useState("");
  const [type, setType] = useState<TypeCompte>("BANQUE");
  const [libelle, setLibelle] = useState("");
  const [banqueOperateur, setBanqueOperateur] = useState("");
  const [numero, setNumero] = useState("");
  const [soldeOuverture, setSoldeOuverture] = useState("");

  const charger = useCallback(
    () =>
      Promise.all([
        appeler<TresorerieCompte[]>("tresorerie_comptes"),
        appeler<TresorerieSolde[]>("tresorerie_soldes"),
      ]),
    [],
  );

  useEffect(() => {
    let annule = false;
    void charger()
      .then(([reçus, lus]) => {
        if (!annule) {
          setComptes(reçus);
          setSoldes(lus);
        }
      })
      .catch((e: unknown) => {
        if (e instanceof ErreurAppel && !annule) setErreur(e.message);
      })
      .finally(() => {
        if (!annule) setChargement(false);
      });
    return () => {
      annule = true;
    };
  }, [charger]);

  const creer = (evenement: FormEvent<HTMLFormElement>) => {
    evenement.preventDefault();
    const nombre = Number(soldeOuverture.trim());
    if (!Number.isInteger(nombre)) {
      setErreur("Le solde d'ouverture doit être un montant entier en FCFA.");
      return;
    }
    const params: CreerCompteParams = {
      type_compte: type,
      libelle,
      banque_operateur: banqueOperateur.trim() || null,
      numero: numero.trim() || null,
      solde_ouverture: nombre,
    };
    appeler<TresorerieCompte>("tresorerie_compte_creer", params)
      .then(() => {
        setLibelle("");
        setBanqueOperateur("");
        setNumero("");
        setSoldeOuverture("");
        void charger()
          .then(([reçus, lus]) => {
            setComptes(reçus);
            setSoldes(lus);
          })
          .catch((e: unknown) => {
            if (e instanceof ErreurAppel) setErreur(e.message);
          });
      })
      .catch((e: unknown) => {
        if (e instanceof ErreurAppel) setErreur(e.message);
      });
  };

  return (
    <div>
      <h1>Trésorerie</h1>
      <p className="texte-secondaire">
        Comptes bancaires, caisses et portefeuilles Mobile Money (MTN MoMo, Moov Money, Celtic Cash).
      </p>
      {erreur !== "" ? <Bandeau type="alerte">{erreur}</Bandeau> : null}

      <Carte titre="Nouveau compte de trésorerie">
        <form onSubmit={creer}>
          <label htmlFor="tresorerie-type">Type de compte</label>
          <select id="tresorerie-type" value={type} onChange={(e) => setType(e.target.value as TypeCompte)}>
            <option value="BANQUE">Banque</option>
            <option value="CAISSE">Caisse</option>
            <option value="MOBILE_MONEY">Mobile Money</option>
          </select>
          <Champ
            libelle="Libellé du compte"
            valeur={libelle}
            surChangement={setLibelle}
            obligatoire
          />
          <Champ
            libelle="Banque ou opérateur"
            valeur={banqueOperateur}
            surChangement={setBanqueOperateur}
            aide="Obligatoire pour Mobile Money (table tresorerie_operateur)."
          />
          <Champ
            libelle="Numéro de compte ou de téléphone"
            valeur={numero}
            surChangement={setNumero}
            type="tel"
          />
          <Champ
            libelle="Solde d'ouverture (FCFA)"
            valeur={soldeOuverture}
            surChangement={setSoldeOuverture}
            aide="Positif ou nul ; impossible pour une caisse."
          />
          <Bouton variante="principal" type="submit">
            Créer le compte
          </Bouton>
        </form>
      </Carte>

      <Carte titre="Comptes de trésorerie">
        {chargement ? <p className="texte-secondaire">Chargement…</p> : null}
        <table>
          <thead>
            <tr>
              <th>Type</th>
              <th>Libellé</th>
              <th>Numéro</th>
              <th>Compte comptable</th>
              <th>Journal</th>
              <th>Solde</th>
            </tr>
          </thead>
          <tbody>
            {comptes.map((c) => {
              const s = soldes.find((x) => x.compte === c.compte_comptable);
              return (
                <tr key={c.id}>
                  <td>{libelleTypeCompte(c.type_compte)}</td>
                  <td>{c.libelle}</td>
                  <td>{c.numero ?? ""}</td>
                  <td>{c.compte_comptable}</td>
                  <td>{c.journal}</td>
                  <td>{s ? formaterMontant(s.solde) : "0 FCFA"}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {comptes.length === 0 ? <p className="texte-secondaire">Aucun compte de trésorerie.</p> : null}
      </Carte>
    </div>
  );
}

export const declaration: DeclarationModule = {
  id: "tresorerie",
  libelle: "Trésorerie",
  chemin: "/tresorerie",
  ordre: 30,
  Page,
};
