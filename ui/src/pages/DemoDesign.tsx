// Démonstration interne du design system — visible uniquement en développement.
import { useState } from "react";
import {
  Assistant,
  Bandeau,
  Bouton,
  Carte,
  CaseACocher,
  Champ,
  ChampDate,
  ChampMontant,
  Chargement,
  Dialogue,
  EtatVide,
  MessageErreur,
  Montant,
  Notification,
  Onglets,
  Selecteur,
  Tableau,
  type ColonneTableau,
  type Onglet,
} from "../design/composants";

type LigneDemo = {
  id: number;
  libelle: string;
  montant: number;
};

const lignes: readonly LigneDemo[] = [
  { id: 1, libelle: "Vente de fournitures", montant: 1250000 },
  { id: 2, libelle: "Prestation de conseil", montant: 450000 },
];

const colonnes: readonly ColonneTableau<LigneDemo>[] = [
  { id: "libelle", entete: "Libellé", cle: "libelle" },
  { id: "montant", entete: "Montant", cle: "montant", format: "montant" },
];

const onglets: readonly Onglet[] = [
  { id: "general", libelle: "Général", contenu: <p>Les réglages généraux sont regroupés ici.</p> },
  { id: "aide", libelle: "Aide", contenu: <p>Une aide courte accompagne chaque champ.</p> },
];

export function DemoDesign() {
  const [nom, setNom] = useState("");
  const [montant, setMontant] = useState(1250000);
  const [date, setDate] = useState("27/11/2026");
  const [choix, setChoix] = useState("services");
  const [cochee, setCochee] = useState(false);
  const [dialogueOuvert, setDialogueOuvert] = useState(false);
  const [notificationVisible, setNotificationVisible] = useState(true);
  const [etape, setEtape] = useState(0);

  return (
    <div className="demo-design">
      <h1>Démonstration du design system</h1>
      <Bandeau>Cette page interne présente les composants utilisables par tous les modules.</Bandeau>

      <Carte titre="Champs et choix">
        <div className="grille">
          <Champ libelle="Nom de l'entreprise" valeur={nom} surChangement={setNom} aide="Une phrase courte et claire." />
          <ChampMontant
            libelle="Montant"
            valeur={montant}
            surChangement={setMontant}
            aide="Les montants sont toujours des entiers en FCFA."
          />
          <ChampDate libelle="Date" valeur={date} surChangement={setDate} />
          <Selecteur
            libelle="Secteur"
            options={[
              { valeur: "commerce", libelle: "Commerce" },
              { valeur: "services", libelle: "Services" },
              { valeur: "industrie", libelle: "Industrie" },
            ]}
            valeur={choix}
            surChangement={setChoix}
          />
        </div>
        <CaseACocher
          libelle="Je souhaite recevoir les informations utiles"
          cochee={cochee}
          surChangement={setCochee}
          aide="Vous pourrez modifier ce choix dans les paramètres."
        />
        <p>
          Aperçu du montant : <Montant francs={montant} />
        </p>
      </Carte>

      <Carte titre="Tableau paginé et triable">
        <Tableau
          colonnes={colonnes}
          lignes={lignes}
          page={1}
          total={8}
          surPage={2}
          libelle="Exemples de montants"
          cleLigne={(ligne) => ligne.id}
          libelleLigne={(ligne) => ligne.libelle}
          surChangementPage={() => undefined}
          surChangementTri={() => undefined}
          surLigne={(ligne) => setNom(ligne.libelle)}
        />
      </Carte>

      <Carte titre="Assistant et onglets">
        <Assistant
          etapes={[
            { titre: "Préparer", contenu: <p>Vérifiez les informations avant de continuer.</p> },
            { titre: "Relire", contenu: <p>Cette étape permet de relire les choix effectués.</p> },
            { titre: "Terminer", contenu: <p>La validation sera demandée après le résumé.</p> },
          ]}
          etape={etape}
          surChangementEtape={setEtape}
          resumeFinal={<p>Les informations sont prêtes à être confirmées.</p>}
          surValider={() => setNotificationVisible(true)}
        />
        <div className="demo-design__separation">
          <Onglets onglets={onglets} />
        </div>
      </Carte>

      <Carte titre="Dialogue et retours d'état">
        <div className="demo-design__actions">
          <Bouton variante="principal" onClick={() => setDialogueOuvert(true)}>
            Ouvrir le dialogue
          </Bouton>
          <Bouton onClick={() => setNotificationVisible(true)}>Afficher une notification</Bouton>
        </div>
        {notificationVisible ? (
          <Notification type="succes" surFermer={() => setNotificationVisible(false)}>
            Votre préférence a été enregistrée.
          </Notification>
        ) : null}
        <div className="demo-design__retours">
          <EtatVide titre="Aucun document" description="Créez votre premier document pour le retrouver ici." />
          <Chargement />
          <MessageErreur
            erreur="Le service est momentanément indisponible."
            actionProposee="Vérifiez votre connexion puis réessayez."
            surRéessayer={() => setNotificationVisible(true)}
          />
        </div>
        <Dialogue
          ouvert={dialogueOuvert}
          titre="Confirmer cette action"
          surFermer={() => setDialogueOuvert(false)}
          surConfirmer={() => setDialogueOuvert(false)}
        >
          <p>Cette action peut être annulée avant la confirmation.</p>
        </Dialogue>
      </Carte>
    </div>
  );
}
