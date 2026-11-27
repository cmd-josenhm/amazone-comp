// Écran « Clients » du module Ventes — Propriétaire : A4 (Lot 1 : fichier clients).
// Aucun calcul métier ici : les contrôles (IFU, téléphone, doublons) sont faits par `ac-ventes`.
import { useCallback, useEffect, useId, useState, type FormEvent } from "react";
import { Bandeau, Bouton, Carte, Champ } from "../../design/composants";
import { appeler } from "../../lib/ipc";
import type { Client, InformationsClient, TypeClient } from "./types";
import "./ventes.css";

/** Code du client « Comptant » (vente au comptoir), créé avec chaque dossier. */
const CODE_COMPTANT = "CLI-000001";

/** Résultats demandés à chaque recherche. */
const LIMITE_RECHERCHE = 100;

const LIBELLES_TYPES: Record<TypeClient, string> = {
  PARTICULIER: "Particulier",
  ENTREPRISE: "Entreprise",
  ADMINISTRATION: "Administration",
};

function estTypeClient(valeur: string): valeur is TypeClient {
  return valeur === "PARTICULIER" || valeur === "ENTREPRISE" || valeur === "ADMINISTRATION";
}

/** Champ facultatif : `null` si vide (attendu par le Rust). */
function ouVide(texte: string): string | null {
  const nettoye = texte.trim();
  return nettoye === "" ? null : nettoye;
}

function messageErreur(e: unknown): string {
  return e instanceof Error
    ? e.message
    : "Une erreur inattendue est survenue. Réessayez, puis contactez l'assistance.";
}

function rechercherClients(texte: string, inclureInactifs: boolean): Promise<Client[]> {
  return appeler<Client[]>("ventes_clients_rechercher", {
    recherche: { texte, inclure_inactifs: inclureInactifs, limite: LIMITE_RECHERCHE },
  });
}

function creerClient(informations: InformationsClient): Promise<Client> {
  return appeler<Client>("ventes_client_creer", { informations });
}

function modifierClient(id: number, informations: InformationsClient): Promise<Client> {
  return appeler<Client>("ventes_client_modifier", { id, informations });
}

function desactiverClient(id: number): Promise<void> {
  return appeler("ventes_client_desactiver", { id }).then(() => undefined);
}

function reactiverClient(id: number): Promise<void> {
  return appeler("ventes_client_reactiver", { id }).then(() => undefined);
}

function SelecteurTypeClient({
  valeur,
  surChangement,
}: {
  valeur: TypeClient;
  surChangement: (valeur: TypeClient) => void;
}) {
  const id = useId();
  return (
    <div className="champ">
      <label htmlFor={id}>Type de client</label>
      <select
        id={id}
        value={valeur}
        onChange={(e) => {
          if (estTypeClient(e.target.value)) {
            surChangement(e.target.value);
          }
        }}
      >
        <option value="PARTICULIER">Particulier</option>
        <option value="ENTREPRISE">Entreprise</option>
        <option value="ADMINISTRATION">Administration</option>
      </select>
    </div>
  );
}

function FormulaireClient({
  client,
  surAnnuler,
  surEnregistre,
}: {
  client: Client | null;
  surAnnuler: () => void;
  surEnregistre: (client: Client) => void;
}) {
  const [nom, setNom] = useState(client?.nom ?? "");
  const [typeClient, setTypeClient] = useState<TypeClient>(client?.type_client ?? "PARTICULIER");
  const [ifu, setIfu] = useState(client?.ifu ?? "");
  const [rccm, setRccm] = useState(client?.rccm ?? "");
  const [adresse, setAdresse] = useState(client?.adresse ?? "");
  const [telephone, setTelephone] = useState(client?.telephone ?? "");
  const [courriel, setCourriel] = useState(client?.courriel ?? "");
  const [retientTva, setRetientTva] = useState(client?.retient_tva ?? false);
  const [exonereAib, setExonereAib] = useState(client?.exonere_aib ?? false);
  const [motif, setMotif] = useState(client?.motif_exoneration_aib ?? "");
  const [erreur, setErreur] = useState<string | null>(null);
  const [enCours, setEnCours] = useState(false);

  async function surSoumission(evenement: FormEvent<HTMLFormElement>) {
    evenement.preventDefault();
    setErreur(null);
    const informations: InformationsClient = {
      nom: nom.trim(),
      type_client: typeClient,
      ifu: ouVide(ifu),
      rccm: ouVide(rccm),
      adresse: ouVide(adresse),
      telephone: ouVide(telephone),
      courriel: ouVide(courriel),
      retient_tva: retientTva,
      exonere_aib: exonereAib,
      motif_exoneration_aib: ouVide(motif),
    };
    setEnCours(true);
    try {
      const enregistre =
        client === null ? await creerClient(informations) : await modifierClient(client.id, informations);
      surEnregistre(enregistre);
    } catch (e) {
      setErreur(messageErreur(e));
    } finally {
      setEnCours(false);
    }
  }

  return (
    <form aria-label="Formulaire du client" onSubmit={surSoumission}>
      {erreur !== null ? <Bandeau type="alerte">{erreur}</Bandeau> : null}
      <Champ libelle="Nom ou raison sociale" valeur={nom} surChangement={setNom} obligatoire />
      <SelecteurTypeClient valeur={typeClient} surChangement={setTypeClient} />
      <Champ
        libelle="IFU (identifiant fiscal unique)"
        valeur={ifu}
        surChangement={setIfu}
        aide="13 chiffres, si le client en a un."
      />
      <Champ libelle="RCCM" valeur={rccm} surChangement={setRccm} />
      <Champ libelle="Adresse" valeur={adresse} surChangement={setAdresse} multiligne />
      <Champ
        libelle="Téléphone"
        valeur={telephone}
        surChangement={setTelephone}
        type="tel"
        aide="Exemple : 01 97 12 34 56."
      />
      <Champ libelle="Adresse e-mail" valeur={courriel} surChangement={setCourriel} type="email" />
      <label className="case">
        <input
          type="checkbox"
          checked={retientTva}
          onChange={(e) => setRetientTva(e.target.checked)}
        />
        Ce client retient la TVA à la source (État, société désignée)
      </label>
      <label className="case">
        <input
          type="checkbox"
          checked={exonereAib}
          onChange={(e) => setExonereAib(e.target.checked)}
        />
        Ce client est exonéré d'AIB
      </label>
      {exonereAib ? (
        <Champ
          libelle="Motif de l'exonération d'AIB"
          valeur={motif}
          surChangement={setMotif}
          obligatoire
        />
      ) : null}
      <div className="actions-ligne">
        <Bouton type="submit" variante="principal" disabled={enCours}>
          Enregistrer
        </Bouton>
        <Bouton onClick={surAnnuler} disabled={enCours}>
          Annuler
        </Bouton>
      </div>
    </form>
  );
}

/** Fichier clients : liste, recherche, création, modification, désactivation. */
export function Clients() {
  const [filtre, setFiltre] = useState<{ texte: string; inclureInactifs: boolean }>({
    texte: "",
    inclureInactifs: false,
  });
  const [texteSaisi, setTexteSaisi] = useState("");
  const [clients, setClients] = useState<Client[]>([]);
  const [erreur, setErreur] = useState<string | null>(null);
  const [modeFormulaire, setModeFormulaire] = useState(false);
  const [enEdition, setEnEdition] = useState<Client | null>(null);

  const charger = useCallback((f: { texte: string; inclureInactifs: boolean }) => {
    rechercherClients(f.texte, f.inclureInactifs)
      .then((liste) => {
        setClients(liste);
        setErreur(null);
      })
      .catch((e: unknown) => setErreur(messageErreur(e)));
  }, []);

  useEffect(() => {
    charger(filtre);
  }, [charger, filtre]);

  async function surDesactivation(id: number) {
    try {
      await desactiverClient(id);
      charger(filtre);
    } catch (e) {
      setErreur(messageErreur(e));
    }
  }

  async function surReactivation(id: number) {
    try {
      await reactiverClient(id);
      charger(filtre);
    } catch (e) {
      setErreur(messageErreur(e));
    }
  }

  function ouvrirCreation() {
    setEnEdition(null);
    setModeFormulaire(true);
    setErreur(null);
  }

  function ouvrirEdition(client: Client) {
    setEnEdition(client);
    setModeFormulaire(true);
    setErreur(null);
  }

  return (
    <Carte titre="Fichier clients">
      {erreur !== null ? <Bandeau type="alerte">{erreur}</Bandeau> : null}
      {modeFormulaire ? (
        <FormulaireClient
          client={enEdition}
          surAnnuler={() => {
            setModeFormulaire(false);
            setErreur(null);
          }}
          surEnregistre={() => {
            setModeFormulaire(false);
            setEnEdition(null);
            charger(filtre);
          }}
        />
      ) : (
        <>
          <form
            className="barre-recherche"
            aria-label="Rechercher un client"
            onSubmit={(e) => {
              e.preventDefault();
              setFiltre({ texte: texteSaisi, inclureInactifs: filtre.inclureInactifs });
            }}
          >
            <Champ
              libelle="Rechercher (nom, IFU, téléphone)"
              valeur={texteSaisi}
              surChangement={setTexteSaisi}
              aide="Laisser vide pour afficher tous les clients."
            />
            <Bouton type="submit" variante="principal">
              Rechercher
            </Bouton>
            <Bouton onClick={ouvrirCreation}>Nouveau client</Bouton>
            <label className="case">
              <input
                type="checkbox"
                checked={filtre.inclureInactifs}
                onChange={(e) =>
                  setFiltre({ texte: filtre.texte, inclureInactifs: e.target.checked })
                }
              />
              Afficher les clients désactivés
            </label>
          </form>
          {clients.length === 0 ? (
            <p className="texte-secondaire">Aucun client ne correspond à cette recherche.</p>
          ) : (
            <table className="tableau-clients">
              <thead>
                <tr>
                  <th>Code</th>
                  <th>Nom</th>
                  <th>Type</th>
                  <th>IFU</th>
                  <th>Téléphone</th>
                  <th>Statut</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {clients.map((c) => (
                  <tr key={c.id}>
                    <td>{c.code}</td>
                    <td>{c.nom}</td>
                    <td>{LIBELLES_TYPES[c.type_client]}</td>
                    <td>{c.ifu ?? "—"}</td>
                    <td>{c.telephone ?? "—"}</td>
                    <td>{c.actif ? "Actif" : "Désactivé"}</td>
                    <td>
                      {c.code === CODE_COMPTANT ? (
                        <span className="texte-secondaire">Obligatoire</span>
                      ) : (
                        <div className="actions-ligne">
                          <Bouton onClick={() => ouvrirEdition(c)}>Modifier</Bouton>
                          {c.actif ? (
                            <Bouton onClick={() => surDesactivation(c.id)}>Désactiver</Bouton>
                          ) : (
                            <Bouton onClick={() => surReactivation(c.id)}>Réactiver</Bouton>
                          )}
                        </div>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      )}
    </Carte>
  );
}
