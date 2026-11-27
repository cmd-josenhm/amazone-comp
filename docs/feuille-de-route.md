# Feuille de route V1 (6 mois : décembre 2026 → mai 2027)

Jalons validés par le chef de projet. Une tâche n'est « faite » que fusionnée dans `main`, CI verte.

## Ordre de démarrage (dépendances)

```
Semaine 1-2 :  A1 (noyau + écritures)   A2 (base chiffrée, rôles)   A3 (coque, design)   A10 (plateforme : comptes + licence)
Semaine 2+  :  A8 (paramètres + TVA/AIB purs)   A7 (ITS/CNSS purs)
Semaine 3+  :  A4 (ventes)   A5 (achats/stocks)   A6 (trésorerie)        ← besoin de passer_ecriture (A1)
Semaine 6+  :  A9 (balance, états, reprise)                               ← besoin des lectures A1
```

Avant que leurs dépendances soient livrées, les agents travaillent sur leurs **fonctions pures** (calculs,
validations, modèles) et leurs **tests**, en se basant sur les contrats de `docs/contrats.md`.

## Mois 1 — Fondations (décembre 2026)

- A1 : `Montant`/`Taux` finalisés, plan SYSCOHADA complet, exercices/périodes, `passer_ecriture` complète, contre-passation.
- A2 : SQLCipher + clé enveloppée + clé de secours, utilisateurs/rôles, audit, sauvegarde locale.
- A3 : coque, design system, connexion, assistant de démarrage (écrans), Paramètres, À propos + avis.
- A10 : plateforme (comptes, **inscription des entreprises** avec IFU/RCCM, CGU, émission de licence Ed25519), vérification hors ligne dans `ac-licence`.
- A8 : chargement typé de tous les paramètres 2026, calculs purs TVA / AIB / minimum de perception / TPS.
- A7 : barème ITS, CNSS, VPS (fonctions pures + tests).
- **Jalon M1** : on crée un dossier, on se connecte, on passe une écriture OD équilibrée, licence vérifiée.

## Mois 2 — Opérations courantes (janvier 2027)

- A4 : clients, factures, avoirs, simulateur de normalisation, PDF facture.
- A5 : fournisseurs, factures fournisseurs, articles.
- A6 : comptes de trésorerie, encaissements/décaissements, Mobile Money.
- A7 : salariés, bulletins, écritures de paie.
- A8 : création des paramètres **2027** dès publication de la loi de finances 2027.
- **Jalon M2** : cycle vente → encaissement → TVA collectée visible dans la balance.

## Mois 3 — Compléments (février 2027)

- A5 : stocks (CMUP), inventaire. A6 : rapprochement bancaire, import de relevés. A1 : lettrage, `ac-documents` (PDF/Excel).
- A8 : déclaration de TVA mensuelle (PDF + Excel), déclaration AIB, calendrier + alertes.
- A9 : balance, grand livre, journaux, tableau de bord.
- **Jalon M3** : déclaration TVA d'un mois de test exportée et vérifiable à la main.

## Mois 4 — Fin d'exercice (mars 2027)

- A1 : clôture, à-nouveaux. A8 : IS/IBA, minimum, acomptes, TPS annuelle, ITS/VPS mensuels.
- A9 : bilan, compte de résultat, TFT, notes, DSF ; reprise de données (balance d'ouverture, tiers, articles).
- A10 : sauvegarde distante chiffrée, restauration, envoi des avis, **ASSISTANCE** (module logiciel + console).
- **Jalon M4** : un exercice complet de démonstration produit des états financiers équilibrés.

## Mois 5 — Intégration et bêta (avril 2027)

- A10 : installateur NSIS/MSI, mises à jour signées (auto/manuel), page de téléchargement, **console d'administration web** complète (entreprises, licences, annonces, publications).
- Tous : corrections, performances (PC 4 Go), accessibilité. Tests de bout en bout.
- **Bêta fermée** avec 5 à 10 PME + **validation fiscale par un expert-comptable** (passage `verifie = true`).

## Mois 6 — Version 1.0 (mai 2027)

- Corrections de la bêta, documentation utilisateur, vidéo de prise en main.
- Brancher `EMecefApi` si l'accès DGI est obtenu (sinon V1 avec simulateur clairement signalé + saisie manuelle des références MECeF).
- **Publication V1.0** (tag `v1.0.0`).
