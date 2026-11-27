// Module « Trésorerie » — Propriétaire : A6. Voir PROMPT_A6.md.
import { useCallback, useEffect, useState } from "react";
import type { FormEvent } from "react";
import { Bandeau, Bouton, Carte, Champ, formaterMontant } from "../../design/composants";
import { appeler, ErreurAppel } from "../../lib/ipc";
import type { DeclarationModule } from "../types";
import type {
  CreerCompteParams,
  CreerOperateurParams,
  TresorerieCompte,
  TresorerieConfiguration,
  TresorerieOperateur,
  TresorerieSolde,
  TypeCompte,
} from "./types";

const MONTANT_MAX = 1_000_000_000_000_000;

function libelleTypeCompte(type: TypeCompte): string {
  if (type === "BANQUE") return "Banque";
  if (type === "CAISSE") return "Caisse";
  return "Mobile Money";
}

function Page() {
  const [comptes, setComptes] = useState<TresorerieCompte[]>([]);
  const [soldes, setSoldes] = useState<TresorerieSolde[]>([]);
  const [operateurs, setOperateurs] = useState<TresorerieOperateur[]>([]);
  const [configuration, setConfiguration] = useState<TresorerieConfiguration | null>(null);
  const [chargement, setChargement] = useState(true);
  const [erreur, setErreur] = useState("");
  const [type, setType] = useState<TypeCompte>("BANQUE");
  const [libelle, setLibelle] = useState("");
  const [banqueOperateur, setBanqueOperateur] = useState("");
  const [numero, setNumero] = useState("");
  const [soldeOuverture, setSoldeOuverture] = useState("");
  const [pieceOuverture, setPieceOuverture] = useState("");
  const [compteContrepartie, setCompteContrepartie] = useState("");
  const [nouvelOperateur, setNouvelOperateur] = useState("");

  const charger = useCallback(
    () =>
      Promise.all([
        appeler<TresorerieCompte[]>("tresorerie_comptes"),
        appeler<TresorerieSolde[]>("tresorerie_soldes"),
        appeler<TresorerieOperateur[]>("tresorerie_operateurs"),
        appeler<TresorerieConfiguration>("tresorerie_configuration"),
      ]),
    [],
  );

  const afficherErreur = useCallback((cause: unknown) => {
    if (cause instanceof ErreurAppel) setErreur(cause.message);
  }, []);

  useEffect(() => {
    let annule = false;
    void charger()
      .then(([comptesRecus, soldesLus, operateursLus, configurationLue]) => {
        if (!annule) {
          setComptes(comptesRecus);
          setSoldes(soldesLus);
          setOperateurs(operateursLus);
          setConfiguration(configurationLue);
        }
      })
      .catch((cause: unknown) => {
        if (!annule) afficherErreur(cause);
      })
      .finally(() => {
        if (!annule) setChargement(false);
      });
    return () => {
      annule = true;
    };
  }, [afficherErreur, charger]);

  const rafraichir = () =>
    charger()
      .then(([comptesRecus, soldesLus, operateursLus, configurationLue]) => {
        setComptes(comptesRecus);
        setSoldes(soldesLus);
        setOperateurs(operateursLus);
        setConfiguration(configurationLue);
      })
      .catch(afficherErreur);

  const creer = (evenement: FormEvent<HTMLFormElement>) => {
    evenement.preventDefault();
    setErreur("");
    const texteMontant = soldeOuverture.trim();
    const montant = texteMontant === "" ? 0 : Number(texteMontant);
    if (!Number.isSafeInteger(montant) || montant < -MONTANT_MAX || montant > MONTANT_MAX) {
      setErreur("Le solde d'ouverture doit être un montant entier valide en FCFA.");
      return;
    }
    if (type === "CAISSE" && montant < 0) {
      setErreur("Une caisse ne peut pas avoir un solde d'ouverture négatif.");
      return;
    }
    if (montant !== 0 && pieceOuverture.trim() === "") {
      setErreur("Indiquez la référence de la pièce justificative du solde d'ouverture.");
      return;
    }
    if (montant !== 0 && compteContrepartie.trim() === "") {
      setErreur("Indiquez le compte comptable qui équilibre le solde d'ouverture.");
      return;
    }

    const params: CreerCompteParams = {
      type_compte: type,
      libelle: libelle.trim(),
      banque_operateur: banqueOperateur.trim() || null,
      numero: numero.trim() || null,
      solde_ouverture: montant,
      piece_ouverture: pieceOuverture.trim() || null,
      compte_contrepartie_ouverture: compteContrepartie.trim() || null,
    };
    void appeler<TresorerieCompte>("tresorerie_compte_creer", { params })
      .then(() => {
        setLibelle("");
        setBanqueOperateur("");
        setNumero("");
        setSoldeOuverture("");
        setPieceOuverture("");
        setCompteContrepartie("");
        return rafraichir();
      })
      .catch(afficherErreur);
  };

  const ajouterOperateur = (evenement: FormEvent<HTMLFormElement>) => {
    evenement.preventDefault();
    setErreur("");
    const nom = nouvelOperateur.trim();
    if (nom === "") {
      setErreur("Saisissez le nom de l'opérateur Mobile Money.");
      return;
    }
    const params: CreerOperateurParams = { nom, libelle: null };
    void appeler<TresorerieOperateur>("tresorerie_operateur_creer", { params })
      .then((operateur) => {
        setNouvelOperateur("");
        setBanqueOperateur(operateur.nom);
        return rafraichir();
      })
      .catch(afficherErreur);
  };

  const configurerMobileMoney = () => {
    if (configuration === null) return;
    setErreur("");
    void appeler<TresorerieConfiguration>("tresorerie_mobile_money_configurer", {
      actif: !configuration.mobile_money_actif,
    })
      .then((miseAJour) => {
        setConfiguration(miseAJour);
        if (!miseAJour.mobile_money_actif) {
          setType("BANQUE");
          setBanqueOperateur("");
        }
      })
      .catch(afficherErreur);
  };

  const montant = soldeOuverture.trim() === "" ? 0 : Number(soldeOuverture);

  return (
    <div>
      <h1>Trésorerie</h1>
      <p className="texte-secondaire">
        Comptes bancaires, caisses et portefeuilles Mobile Money (MTN MoMo, Moov Money, Celtiis Cash).
      </p>
      {erreur !== "" ? <Bandeau type="alerte">{erreur}</Bandeau> : null}

      <Carte titre="Nouveau compte de trésorerie">
        <form onSubmit={creer}>
          <label htmlFor="tresorerie-type">Type de compte</label>
          <select
            id="tresorerie-type"
            value={type}
            onChange={(evenement) => {
              setType(evenement.target.value as TypeCompte);
              setBanqueOperateur("");
            }}
          >
            <option value="BANQUE">Banque</option>
            <option value="CAISSE">Caisse</option>
            <option value="MOBILE_MONEY" disabled={configuration?.mobile_money_actif !== true}>
              Mobile Money{configuration?.mobile_money_actif === false ? " (désactivé)" : ""}
            </option>
          </select>
          <Champ
            libelle="Libellé du compte"
            valeur={libelle}
            surChangement={setLibelle}
            obligatoire
          />
          {type === "BANQUE" ? (
            <Champ
              libelle="Banque"
              valeur={banqueOperateur}
              surChangement={setBanqueOperateur}
              obligatoire
            />
          ) : null}
          {type === "MOBILE_MONEY" ? (
            <div className="champ">
              <label htmlFor="tresorerie-operateur">Opérateur Mobile Money *</label>
              <select
                id="tresorerie-operateur"
                value={banqueOperateur}
                required
                onChange={(evenement) => setBanqueOperateur(evenement.target.value)}
              >
                <option value="">Choisir un opérateur</option>
                {operateurs
                  .filter((operateur) => operateur.actif)
                  .map((operateur) => (
                    <option key={operateur.id} value={operateur.nom}>
                      {operateur.nom}
                    </option>
                  ))}
              </select>
              <span className="champ__aide">Le numéro saisi est celui du portefeuille marchand.</span>
            </div>
          ) : null}
          {type !== "CAISSE" ? (
            <Champ
              libelle={type === "MOBILE_MONEY" ? "Téléphone du portefeuille" : "Numéro de compte bancaire"}
              valeur={numero}
              surChangement={setNumero}
              type="tel"
              obligatoire
              aide={
                type === "MOBILE_MONEY"
                  ? "Numéro béninois, par exemple +229 01 XX XX XX XX. Il sera masqué à l'affichage."
                  : "Le numéro sera masqué à l'affichage, sauf ses quatre derniers caractères."
              }
            />
          ) : null}
          <Champ
            libelle="Solde d'ouverture (FCFA)"
            valeur={soldeOuverture}
            surChangement={setSoldeOuverture}
            aide="Montant entier ; une caisse ne peut pas avoir un solde négatif."
          />
          {Number.isSafeInteger(montant) && montant !== 0 ? (
            <Champ
              libelle="Référence de la pièce justificative"
              valeur={pieceOuverture}
              surChangement={setPieceOuverture}
              obligatoire
              aide="Référence du relevé, de l'état de caisse ou du document qui justifie le solde initial."
            />
          ) : null}
          {Number.isSafeInteger(montant) && montant !== 0 ? (
            <Champ
              libelle="Compte de contrepartie du solde d'ouverture"
              valeur={compteContrepartie}
              surChangement={setCompteContrepartie}
              obligatoire
              aide="Saisissez un compte existant et mouvementable du plan comptable ; aucun compte de contrepartie n'est choisi automatiquement."
            />
          ) : null}
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
              <th>Banque ou opérateur</th>
              <th>Numéro masqué</th>
              <th>Compte comptable</th>
              <th>Journal</th>
              <th>Solde</th>
            </tr>
          </thead>
          <tbody>
            {comptes.map((compte) => {
              const soldeCompte = soldes.find((element) => element.compte === compte.compte_comptable);
              return (
                <tr key={compte.id}>
                  <td>{libelleTypeCompte(compte.type_compte)}</td>
                  <td>{compte.libelle}</td>
                  <td>{compte.banque_operateur ?? ""}</td>
                  <td>{compte.numero ?? ""}</td>
                  <td>{compte.compte_comptable}</td>
                  <td>{compte.journal}</td>
                  <td>{soldeCompte ? formaterMontant(soldeCompte.solde) : "0 FCFA"}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {comptes.length === 0 ? <p className="texte-secondaire">Aucun compte de trésorerie.</p> : null}
      </Carte>

      <Carte titre="Opérateurs Mobile Money">
        <p className="texte-secondaire">
          Les opérateurs sont des données. Leur ajout et l'activation du module sont réservés à l'administrateur du dossier.
        </p>
        {configuration ? (
          <div>
            <p>
              Module Mobile Money : {configuration.mobile_money_actif ? "activé" : "désactivé"}.
            </p>
            <Bouton onClick={configurerMobileMoney}>
              {configuration.mobile_money_actif ? "Désactiver Mobile Money" : "Activer Mobile Money"}
            </Bouton>
          </div>
        ) : null}
        <ul>
          {operateurs.map((operateur) => (
            <li key={operateur.id}>
              {operateur.nom}{operateur.actif ? "" : " (désactivé)"}
            </li>
          ))}
        </ul>
        <form onSubmit={ajouterOperateur}>
          <Champ
            libelle="Nom du nouvel opérateur"
            valeur={nouvelOperateur}
            surChangement={setNouvelOperateur}
          />
          <Bouton variante="secondaire" type="submit">
            Ajouter l'opérateur
          </Bouton>
        </form>
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
