# Sources fiscales consultées (recherche du 27 novembre 2026)

**Aucune de ces sources n'a valeur de validation.** Seuls le texte officiel (CGI 2026, loi de finances,
décrets, arrêtés, spécification DGI) et l'avis d'un expert-comptable permettent de passer `verifie = true`.

## Textes officiels à obtenir en priorité (agent A8, chef de projet)

1. **Code général des impôts du Bénin, édition 2026** : une copie PDF est diffusée par un cabinet :
   <https://gpac-sa.com/public/documents/6984ebbbb7bc0-Benin-Code-General-des-Impots-2026-69d5057a65391.pdf>
   (à comparer avec la version publiée par la DGI : <https://www.impots.bj>).
2. **Loi de finances pour la gestion 2026** (adoptée le 4 décembre 2025) : texte intégral au Journal officiel / site du ministère de l'Économie et des Finances.
3. **Doctrine fiscale du Bénin** (DGI, septembre 2023) : <https://api.impots.bj/media/652d651ce03aa_DOCTRINE%20FISCALE%20DU%20BENIN-2023.pdf>
4. **Spécification de l'API e-MECeF** : à demander à la DGI (accès développeur).
5. **CNSS** : <https://cnss.bj> (taux, assiette, procédure de recouvrement).
6. **Décret fixant le SMIG** en vigueur.

## Sources secondaires utilisées pour les valeurs provisoires

| Sujet | Source | Remarque |
|---|---|---|
| LF 2026 : mesures (répartition TVA, déduction 90 % BTP/industrie, carburant, remboursements) | [Banouto — axes clés du budget 2026](https://www.banouto.bj/economie/article/20251123-assemblee-nationale-au-benin-les-axes-cles-du-budget-2026-a-retenir), [Daabaaru — innovations LF 2026](https://daabaaru.bj/loi-de-finances-gestion-2026-au-benin-les-innovations-et-mesures-fiscales-incitatives-apportees), [La Nouvelle Tribune](https://lanouvelletribune.info/2025/12/parlement-beninois-la-loi-de-finances-2026-adoptee-a-lunanimite/) | Presse ; « aucun nouvel impôt » |
| Panorama 2026 (TVA, AIB, IS, minimum, TPS, ITS, VPS, CNSS, pénalités) | [Yoon — fiscalité des entreprises au Bénin 2026](https://www.yoon.africa/guide-pratique/fiscalite-entreprises-benin) | Site commercial, source principale des valeurs provisoires |
| Services numériques étrangers soumis à la TVA | [Auditia Africa](https://www.auditiaafrica.com/actualites/37) | Source unique |
| AIB : 5 % pour IFU désactivé (7 sept. 2026), avoirs et AIB | [Banouto](https://banouto.bj/index.php/economie/article/20260907-recouvrement-d-impots-au-benin-ce-qui-change-a-partir-de-ce-lundi-7-septembre-04-choses-a-retenir), [Banouto — facture annulée et AIB](https://banouto.bj/economie/article/20260909-impots-au-benin-ce-quil-faut-faire-quand-la-facture-est-annulee-apres-le-prelevement-de-laib), [Bénin Web TV](https://beninwebtv.com/recouvrement-de-laib-au-benin-ce-qui-change-a-partir-de-ce-lundi-7-septembre/) | Presse |
| TPS (seuil 50 M, 5 %, minimum 10 000) | [Guide simplifié DGI sur la TPS](https://www.impots.finances.gouv.bj/guide-simplifie-sur-la-tps/), [Doctrine fiscale 2023](https://api.impots.bj/media/652d651ce03aa_DOCTRINE%20FISCALE%20DU%20BENIN-2023.pdf) | Le guide DGI cite encore 2 % (ancien taux) |
| ITS, CNSS, VPS | [CNSS — recouvrement](https://cnss.bj/procedure-de-recouvrement-des-cotisations/), [Africa Paie RH](https://africapaierh.com/juridique/cotisations-sociales-et-impots-en-republique-du-benin/), [Formulaire impôts sur salaires (eRegulations)](https://benin.eregulations.org/media/formulaire%20imp%C3%B4t%20sur%20salaire.pdf) | Divergences sur le 1er seuil ITS et le plafond CNSS |
| e-MECeF (principe) | [DDD Invoices — fiscalisation au Bénin](https://dddinvoices.com/learn/fiscalization-and-real-time-reporting-in-benin) | Blog commercial ; groupes A–F et codes AIB de mémoire, à confirmer |

## Contradictions ouvertes (à transformer en issues `fiscal-a-verifier`)

1. Premier seuil du barème ITS : 60 000 ou 50 000 FCFA ?
2. Plafond mensuel de l'assiette CNSS : aucun ou 600 000 FCFA ?
3. Taux AIB 3 % sur services : existe-t-il en 2026 ? Code e-MECeF correspondant ?
4. Taux de l'IBA et du minimum de perception IBA.
5. Redevance audiovisuelle sur salaires : mois, montants, exemptions.
6. Date et périodicité de déclaration CNSS selon l'effectif.
7. Définition exacte du régime réel simplifié en 2026.
8. Seuils du système minimal de trésorerie (SYSCOHADA) et lien avec la TPS.
9. Exigibilité de la TVA (livraison / encaissement) par type d'opération.
10. Règle d'arrondi officielle (par ligne / par facture) et cohérence avec e-MECeF.
