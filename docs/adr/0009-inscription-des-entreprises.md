# ADR 0009 — Inscription obligatoire des entreprises (IFU, RCCM, identité)

- Statut : **Accepté** (propriétaire, 2026-11-27) — implémentation : A10 (plateforme), A3 (formulaire), A2 (stockage local), A1 (type partagé)

## Contexte
Le propriétaire veut savoir **qui** utilise le logiciel : chaque entreprise béninoise doit fournir ses informations légales
(IFU, registre du commerce, etc.) pour s'inscrire. Ces informations servent aussi à la comptabilité (en-têtes de factures,
déclarations) : elles ne doivent être saisies **qu'une seule fois**.

## Décision

### 1. Une seule fiche, deux usages
La fiche est saisie dans l'**assistant de premier lancement** (étapes 2 et 3). Elle est envoyée à la plateforme
(inscription) **et** enregistrée dans le dossier local (`donnees_entreprise`). Le type partagé est
`ac_noyau::entreprise::FicheEntreprise`, avec une validation unique `FicheEntreprise::valider()` utilisée par
l'application **et** par la plateforme (qui dépend de `ac-noyau` par chemin).

### 2. Champs (★ = obligatoire)

| Bloc | Champs |
|---|---|
| Identité légale | Raison sociale ★, nom commercial, sigle, forme juridique ★ (EI, SARL, SARLU, SA, SAS, SASU, SNC, SCS, GIE, Société coopérative, Association, Autre), date de création ★ |
| Identifiants | **IFU ★** (13 chiffres, unique sur la plateforme), **RCCM ★** (ex. `RB/COT/23 B 12345`), n° employeur CNSS (obligatoire si l'entreprise déclare des salariés) |
| Fiscalité | Régime fiscal ★ (réel normal, réel simplifié, TPS), centre des impôts de rattachement ★ |
| Activité | Secteur d'activité ★ (commerce, services, BTP, industrie, agriculture/élevage/pêche, transport, enseignement, santé, hôtellerie/restauration, autre), activité principale ★ (texte court), effectif salarié |
| Adresse | Département ★ (les 12 départements), commune ★, quartier / rue ★, boîte postale |
| Contact entreprise | Téléphone ★ (format béninois à 10 chiffres `01 XX XX XX XX`, préfixe +229), courriel ★, site web |
| Responsable du compte | Nom ★, prénoms ★, fonction ★, téléphone ★, courriel ★ (identifiant de connexion) |
| Justificatifs | Attestation IFU, extrait RCCM (PDF/JPG/PNG ≤ 5 Mo) : **facultatifs par défaut**, rendus obligatoires si l'administrateur active « vérification renforcée » |
| Consentements | CGU ★, politique de confidentialité ★ (versions enregistrées) |

On ne collecte **pas** : numéro de pièce d'identité, date de naissance, données bancaires (minimisation, loi n° 2017-20).

### 3. Contrôles
- **Bloquants** : champs ★ vides, IFU ≠ 13 chiffres, courriel invalide, téléphone invalide, date de création future, IFU déjà inscrit par un autre compte.
- **Avertissements** (non bloquants tant que le format officiel n'est pas confirmé) : RCCM au format non reconnu, ancien numéro de téléphone à 8 chiffres, forme « EI » au régime réel normal, société de capitaux au régime TPS.
- Formats RCCM et téléphone : **à faire confirmer** (issue `fiscal-a-verifier`) ; ils sont codés comme avertissements pour ne bloquer personne à tort.

### 4. Statut d'une entreprise sur la plateforme
`EN_ATTENTE_VERIFICATION` → `VERIFIEE` | `REJETEE` (motif) | `SUSPENDUE` (motif). Par défaut la licence est délivrée **immédiatement**
à l'inscription (l'utilisateur peut travailler) ; l'administrateur vérifie ensuite. Réglage administrateur « vérification préalable » :
la licence n'est alors délivrée qu'après passage en `VERIFIEE`. Une entreprise `SUSPENDUE` ne peut plus renouveler sa licence ni
recevoir de mises à jour (une licence déjà délivrée reste valable hors ligne jusqu'à son expiration : c'est une limite assumée).

### 5. Modifications ultérieures
Modifiables dans Paramètres et synchronisées à la plateforme. **Changement d'IFU** = demande validée par l'administrateur (la licence est liée à l'IFU).
