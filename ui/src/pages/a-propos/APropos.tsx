// Page « À propos » : logiciel, créateur, avis des utilisateurs — Propriétaire : A3.
// L'envoi des avis vers la plateforme est fourni par A10 (commande `compte_envoyer_avis`).
import { useState } from "react";
import { Bandeau, Bouton, Carte, Champ } from "../../design/composants";
import { appeler, dansTauri, ErreurAppel } from "../../lib/ipc";
import { proprietaire } from "./proprietaire";

const typesAvis = ["Suggestion", "Problème rencontré", "Critique", "Félicitations"] as const;
type TypeAvis = (typeof typesAvis)[number];

export function APropos() {
  return (
    <div>
      <h1>À propos d'AMAZONE COMP</h1>
      <Carte titre="Le logiciel">
        <p>
          Logiciel gratuit de comptabilité pour les PME du Bénin : SYSCOHADA révisé, fiscalité
          béninoise, facture normalisée, paie. Code source libre sous licence GNU AGPL-3.0.
        </p>
        <p className="texte-secondaire">
          Le logiciel prépare vos déclarations ; vous restez responsable de leur dépôt sur
          e-services de la DGI.
        </p>
      </Carte>
      <Createur />
      <FormulaireAvis />
    </div>
  );
}

function Createur() {
  const lignes: [string, string][] = [
    ["Nom", proprietaire.nom],
    ["Contact", proprietaire.contact],
    ["LinkedIn", proprietaire.linkedin],
    ["GitHub", proprietaire.github],
    ["Autres", proprietaire.autres],
  ];
  return (
    <Carte titre="Le créateur">
      <div style={{ display: "flex", gap: "var(--espace-6)", alignItems: "flex-start" }}>
        {proprietaire.photo ? (
          <img src={proprietaire.photo} alt={`Photo de ${proprietaire.nom}`} width={96} height={96} />
        ) : (
          <div
            aria-label="Photo : A COMPLETER"
            style={{
              width: 96,
              height: 96,
              border: "1px dashed var(--bordure)",
              borderRadius: "50%",
              display: "grid",
              placeItems: "center",
              fontSize: 11,
              textAlign: "center",
            }}
          >
            Photo
            <br />A COMPLETER
          </div>
        )}
        <dl style={{ margin: 0, display: "grid", gridTemplateColumns: "auto 1fr", gap: "4px 16px" }}>
          {lignes.map(([cle, val]) => (
            <div key={cle} style={{ display: "contents" }}>
              <dt className="texte-secondaire">{cle}</dt>
              <dd style={{ margin: 0 }}>{val}</dd>
            </div>
          ))}
        </dl>
      </div>
    </Carte>
  );
}

function FormulaireAvis() {
  const [type, setType] = useState<TypeAvis>("Suggestion");
  const [note, setNote] = useState(0);
  const [message, setMessage] = useState("");
  const [contact, setContact] = useState("");
  const [retour, setRetour] = useState<{ type: "info" | "alerte"; texte: string } | null>(null);

  async function envoyer() {
    if (message.trim().length < 5) {
      setRetour({ type: "alerte", texte: "Écrivez au moins quelques mots avant d'envoyer." });
      return;
    }
    if (!dansTauri()) {
      setRetour({ type: "alerte", texte: "L'envoi n'est possible que dans l'application installée." });
      return;
    }
    try {
      await appeler<null>("compte_envoyer_avis", { avis: { type, note, message, contact } });
      setRetour({
        type: "info",
        texte: "Merci ! Votre avis est enregistré. Il sera transmis dès que vous serez connecté à Internet.",
      });
      setMessage("");
    } catch (e) {
      setRetour({
        type: "alerte",
        texte: e instanceof ErreurAppel ? e.message : "Envoi impossible pour le moment.",
      });
    }
  }

  return (
    <Carte titre="Donner votre avis">
      <p className="texte-secondaire">
        Une idée, un problème, une critique ? Le créateur lit tous les messages.
      </p>
      <div className="champ">
        <label htmlFor="type-avis">Type d'avis</label>
        <select id="type-avis" value={type} onChange={(e) => setType(e.target.value as TypeAvis)}>
          {typesAvis.map((t) => (
            <option key={t}>{t}</option>
          ))}
        </select>
      </div>
      <fieldset className="champ" style={{ border: "none", padding: 0 }}>
        <legend>Note (facultative)</legend>
        <div style={{ display: "flex", gap: 8 }}>
          {[1, 2, 3, 4, 5].map((n) => (
            <Bouton
              key={n}
              variante={n <= note ? "principal" : "secondaire"}
              aria-pressed={n <= note}
              aria-label={`${n} sur 5`}
              onClick={() => setNote(n === note ? 0 : n)}
            >
              {n}
            </Bouton>
          ))}
        </div>
      </fieldset>
      <Champ libelle="Votre message" valeur={message} surChangement={setMessage} multiligne obligatoire />
      <Champ
        libelle="Vos coordonnées (facultatif)"
        valeur={contact}
        surChangement={setContact}
        aide="Seulement si vous souhaitez une réponse."
      />
      {retour ? <Bandeau type={retour.type}>{retour.texte}</Bandeau> : null}
      <Bouton variante="principal" onClick={() => void envoyer()}>
        Envoyer mon avis
      </Bouton>
    </Carte>
  );
}
