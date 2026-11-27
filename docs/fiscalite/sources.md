# Sources fiscales consultées

État au **27 novembre 2026** (Lot 1 de `PROMPT_A8.md`, agent A8). Ce fichier remplace la version
provisoire du 27 novembre : chaque contradiction est désormais **tranchée sur le texte** ou **suivie par
une issue `fiscal-a-verifier`**.

**Aucune de ces sources n'a valeur de validation.** Seuls le texte officiel (CGI 2026, loi de finances,
décrets, arrêtés, spécification DGI) et l'avis d'un expert-comptable permettent de passer `verifie = true`
— et cette décision appartient au chef de projet.

## 1. Textes officiels : ce que nous avons réellement

| Texte | Obtention | Usage au Lot 1 |
|---|---|---|
| **CGI 2026** — loi n° 2021-15 du 23 décembre 2021, mise à jour de la **loi n° 2025-22 du 8 décembre 2025** portant loi de finances pour la gestion 2026 | Copie PDF officielle de la DGI (`api.impots.bj`) et copie d'un cabinet (`gpac-sa.com`), identiques | Source de **tous** les articles cités dans `parametres-fiscaux/2026/` |
| **Loi de finances pour la gestion 2026** — loi n° 2025-22 du **8 décembre 2025** (adoptée à l'unanimité le 4 décembre 2025) | Comptes rendus parlementaires + reprise dans le CGI consolidé | Répartition de la TVA, déduction carburant (art. 247, 2)), plancher de l'IBA ramené à 250 000 FCFA (art. 64, 3) a)) |
| **Doctrine fiscale du Bénin** (DGI, 2023) | `api.impots.bj` | Textes d'application, taux de retenue de TVA (40 %) |
| **Rescrits fiscaux DGI 2026** | `api.impots.bj` | Retenue non-résidents (20 % depuis 2023), loyers et AIB (art. 106 / 131-7) |
| **Spécification de l'API e-MECeF** | **non obtenue** (accès développeur DGI à demander) | Rien n'est vérifiable : issues #24 et #54 |
| **Code de sécurité sociale / CNSS** | **non obtenu** | Paramètres sociaux : A7, issues #23, #27 et #47 |
| **Arrêté fixant le seuil d'assujettissement (TVA / TPS)** | **non obtenu** | Seuil de 50 000 000 FCFA non justifiable : issue #39 |

> Les copies du CGI utilisées sont **tronquées par endroits** (livre 4 « sanctions », art. 119 à 129 pour
> l'ITS, barème de la patente). Chaque valeur concernée porte une `note` qui renvoie à l'issue ouverte
> plutôt qu'une affirmation non vérifiée (règle R14).
>
> **Seconde passe (27 novembre 2026, même journée).** Les articles qui manquaient ont été retrouvés par
> **recherche plein texte dans la copie indexée du CGI 2026** (`gpac-sa.com`) : **art. 110** (exonérations
> d'AIB), **art. 146** (réduction d'impôt des entreprises nouvelles), **art. 180, 3)** (exonération de TPS
> des entreprises nouvelles) et **art. 198** (exonération de la contribution des patentes). Trois articles
> seulement ont ainsi pu être cités ; l'art. 110 reste **à confirmer** (issue #59) car le texte de la
> nouvelle copie est plus elliptical que celui de la copie DGI.

## 2. Ce que la lecture du CGI 2026 a établi (articles exacts)

| Sujet | Article | Ce que dit le texte | Fichier |
|---|---|---|---|
| Taux de la TVA | **art. 241** | « Le taux de la taxe sur la valeur ajoutée est fixé à 18 % » | `tva.toml` |
| Seuil d'assujettissement à la TVA | **art. 228** | Seuil fixé par **arrêté** du ministre des finances (pas de montant dans le code) | `tva.toml` |
| Exigibilité de la TVA | **art. 236** | Biens : au fait générateur ; services, travaux immobiliers et marchés publics de l'État : **à l'encaissement** ; avances et acomptes : exigibles à leur versement | `regles-fiscales-benin.md` §4 |
| Fait générateur | **art. 235** | Livraison pour les biens, exécution pour les services ; jamais postérieur à la facture totale ou partielle | `regles-fiscales-benin.md` §4 |
| Déclaration mensuelle de TVA | **art. 259, 1)** | Au plus tard le **10 de chaque mois**, au titre du mois précédent | `calendrier.toml`, `tva.toml` |
| Crédit de TVA | **art. 252** | L'excédent de déduction est imputable sur la taxe de la période suivante, jusqu'à épuisement | `regles-fiscales-benin.md` §4 |
| Déduction carburant et plafond BTP | **art. 247, 2)** (LF 2026) | Produits pétroliers pour cuves, gaz des usines, gasoil/huiles/graisses des industriels et du BTP ; **90 % pour les BTP** | `tva.toml` (issue #32) |
| Retenue de TVA à la source | **art. 263** | Structures désignées (État, collectivités…), 40 % / 100 % | `tva.toml` (issue #35) |
| Taux de l'IS | **art. 46, 1)** | **25 %** industrie (hors extractives) et écoles privées ; **30 %** autres personnes morales | `impot-resultat.toml` (issue #33) |
| Minimum de perception IS | **art. 47, 1)** | 1 % des produits encaissables (3 % BTP, 10 % prépondérance immobilière) | `impot-resultat.toml` |
| Plancher de l'IS | **art. 47, 3)** | **250 000 FCFA** dans tous les cas | `impot-resultat.toml` |
| Redevance ORTB de l'IS | **art. 47, 6)** | 4 000 FCFA, payables avec l'acompte du 10 mars | `impot-resultat.toml` (issue #55) |
| Déclaration de résultat | **art. 49, 1)** | Au plus tard le **30 avril** | `impot-resultat.toml` |
| Acomptes de l'IS | **art. 51, 1) et 2)** | 4 acomptes (10 mars, juin, septembre, décembre) = **quart de l'impôt N-1**, régularisé au 2ᵉ acompte | `impot-resultat.toml` (issue #34) |
| Taux de l'IBA | **art. 63, 1)** | 30 % ; 25 % pour les écoles privées (art. 63, 2)) | `impot-resultat.toml` (issue #25) |
| Minimum de perception IBA | **art. 64, 1)** et **2)** | 1,5 % des produits encaissables ; 3 % BTP ; 10 % immobilier | `impot-resultat.toml` |
| Plancher de l'IBA | **art. 64, 3) a)** | **250 000 FCFA** (LF 2026 ; la version 2025 portait 500 000) | `impot-resultat.toml` (issue #25) |
| Champ de la TPS | **art. 178** | Contribuables de l'IBA sous un **seuil fixé par arrêté** ; TPS libératoire de l'IBA, des patentes, des licences et du **VPS** | `tps.toml` (issue #39) |
| Taux et minimum de la TPS | **art. 183** | **5 %** des recettes, minimum **10 000 FCFA**, redevance **4 000 FCFA**, taxe **par commune et par établissement** | `tps.toml` |
| Acomptes et déclaration TPS | **art. 184** et **185** | Déclaration au 30 avril ; 2 acomptes (10 février, 10 juin) sur l'impôt N-1 | `tps.toml`, `calendrier.toml` |
| Taux de l'AIB | **art. 132, 2)** | 1 % (a), **3 % services IFU** (b), 5 % (c), 5 % factures normalisées non connus (d) | `aib.toml` (issue #24) |
| Reversement de l'AIB | **art. 133, 3) et 4)** | Au plus tard le 10 du mois suivant ; imputation sur IS/IBA/TPS | `aib.toml` |
| Retenue sur loyers | **art. 106, 1)** | **12 %**, réduite à **10 %** si le bailleur relève de l'IBA ou de l'IS | `retenues.toml` (issue #36) |
| Prestataires non-résidents | **art. 141** et **142** | Retenue **20 %** sur le HT, frais accessoires compris | `retenues.toml` (issue #37) |
| IRCM | **art. 86** | **5 %** dividendes de non-résidents, **10 %** autres dividendes, **15 %** autres revenus | `retenues.toml` (issue #37) |
| Facture normalisée | **art. 256** | Toute opération d'un redevable fait l'objet d'une facture normalisée émise et transmise par une **machine électronique certifiée** | `emecef.toml` (issue #54) |
| Réduction d'impôt des entreprises nouvelles | **art. 146, 1)** (modifié par la LF 2023) | IS ou IBA dû par les **sociétés nouvelles régulièrement créées** réduit de **25 %** (1ᵉʳ exercice), **25 %** (2ᵉ exercice), **50 %** (3ᵉ) ; sans effet en cas de rappel de droits (art. 146, 2) a)) | `impot-resultat.toml` (issue #52) |
| Exonération de TPS des entreprises nouvelles | **art. 180, 3)** | **Douze (12) premiers mois** d'activité ; la taxe est due à partir du premier jour du mois suivant l'expiration ; les entreprises devenues imposables pour la première fois paient le minimum (art. 183, 2)) dans les dix premiers jours | `tps.toml` (issue #39) |
| Exonération de la contribution des patentes | **art. 198** | **Douze (12) premiers mois** d'activité pour les entreprises nouvelles régulièrement créées | `calendrier.toml` (issue #53) |
| Exonérations d'AIB | **art. 110** (à confirmer) | « Sont exemptés de la taxe : … 4) les entreprises nouvelles justifiant qu'elles relèvent de la TPS, au titre de leurs douze (12) premiers mois d'activités ; 5) les personnes physiques ou morales justifiant d'une exonération de l'IR ou de la TPS » | `aib.toml` (issue #59) |

## 3. Les 10 contradictions initiales : état

| # | Contradiction | État au 2026-11-27 | Issue |
|---|---|---|---|
| 1 | Premier seuil du barème ITS : 60 000 ou 50 000 FCFA ? | **Ouverte** — art. 125 du CGI 2026 non lisible dans nos copies ; propriété de A7 | [#22](https://github.com/cmd-josenhm/amazone-comp/issues/22) (doublon chez A7 : #43) |
| 2 | Plafond mensuel de l'assiette CNSS | **Ouverte** — hors CGI (Code de sécurité sociale, non obtenu) ; A7 | [#23](https://github.com/cmd-josenhm/amazone-comp/issues/23) (doublon chez A7 : #45) |
| 3 | Taux AIB 3 % sur services et code e-MECeF | **Tranchée sur le texte** (art. 132, 2) b)) ; le **code e-MECeF** reste ouvert | [#24](https://github.com/cmd-josenhm/amazone-comp/issues/24) |
| 4 | Taux de l'IBA et minimum de perception IBA | **Tranchée sur le texte** (art. 63 et 64) ; plancher corrigé à 250 000 FCFA ; validation expert à faire | [#25](https://github.com/cmd-josenhm/amazone-comp/issues/25) |
| 5 | Redevance audiovisuelle sur salaires | **Ouverte** — mécanisme absent de nos copies ; A7 | [#26](https://github.com/cmd-josenhm/amazone-comp/issues/26) (doublon chez A7 : #46) |
| 6 | Date et périodicité de déclaration CNSS | **Ouverte** — hors CGI ; A7 et A8 | [#27](https://github.com/cmd-josenhm/amazone-comp/issues/27) (doublon chez A7 : #47) |
| 7 | Définition du régime réel simplifié en 2026 | **Ouverte — critique** : aucun intitulé « réel simplifié » dans le CGI 2026 consulté ; ADR éventuel pour `Regime` (C11) | [#28](https://github.com/cmd-josenhm/amazone-comp/issues/28) |
| 8 | Seuils du système minimal de trésorerie (SYSCOHADA) | **Ouverte** — Acte uniforme OHADA non obtenu ; concerne A9 et A1 | [#29](https://github.com/cmd-josenhm/amazone-comp/issues/29) |
| 9 | Exigibilité de la TVA par type d'opération | **Tranchée sur le texte** (art. 235 et 236) ; reste l'option « débits » | [#30](https://github.com/cmd-josenhm/amazone-comp/issues/30) |
| 10 | Règle d'arrondi et cohérence e-MECeF | **Ouverte** — aucune règle d'arrondi dans le CGI ; spécification e-MECeF à obtenir | [#31](https://github.com/cmd-josenhm/amazone-comp/issues/31) |

## 4. Contradictions découvertes au Lot 1

| Sujet | Pourquoi c'est une contradiction | Issue |
|---|---|---|
| TVA à 90 % : BTP seulement ou industries aussi ? | Le texte (art. 247, 2) / 224) ne vise que les BTP ; la presse et un séminaire DGI ajoutent les industries, la logistique et les déchets ; numéro d'article variable selon les copies | [#32](https://github.com/cmd-josenhm/amazone-comp/issues/32) |
| Taux industriel de l'IS à 25 % | Absent des CGI 2023-2024 (30 % sauf écoles), présent à l'art. 46 du CGI 2026 : date d'entrée en vigueur à établir | [#33](https://github.com/cmd-josenhm/amazone-comp/issues/33) |
| Mode de calcul des acomptes IS et TPS | Le quart de l'impôt N-1 et la régularisation au 2ᵉ acompte n'étaient pas documentés ni paramétrés | [#34](https://github.com/cmd-josenhm/amazone-comp/issues/34) |
| Retenue de TVA 100 % / 40 % | Seul le taux de 40 % est attesté par la doctrine (arrêté de 2007) ; les 100 % ne viennent que de sources secondaires | [#35](https://github.com/cmd-josenhm/amazone-comp/issues/35) |
| Retenue sur loyers : critère | Le texte vise le **régime** du bailleur (IBA/IS), le TOML visait sa **forme** (personne physique/morale) | [#36](https://github.com/cmd-josenhm/amazone-comp/issues/36) |
| Retenue non-résidents et IRCM | Taux IRCM absents du fichier ; 20 % à confirmer (10 / 12 % pour les contrats antérieurs) | [#37](https://github.com/cmd-josenhm/amazone-comp/issues/37) |
| Pénalités (art. 484 à 502) | Numéros d'articles recopiés d'une source secondaire ; livre 4 non lisible dans nos copies | [#38](https://github.com/cmd-josenhm/amazone-comp/issues/38) |
| TPS : seuil, exonération, par commune | Le seuil de 50 000 000 FCFA vient d'un **arrêté**, pas du CGI ; l'exonération des 12 premiers mois est désormais identifiée (**art. 180, 3)**) mais son champ exact reste à confirmer ; taxe due **par commune et par établissement** (art. 183, 4)) | [#39](https://github.com/cmd-josenhm/amazone-comp/issues/39) |
| TVA sur les services numériques étrangers | Une seule source secondaire ; l'art. 224, 9) rend l'opération imposable sans trancher l'autoliquidation | [#40](https://github.com/cmd-josenhm/amazone-comp/issues/40) |
| Taux zéro des exportations | L'art. 241 lu ne contient que 18 % ; taux zéro ou exonération changent le droit à déduction | [#51](https://github.com/cmd-josenhm/amazone-comp/issues/51) |
| Réduction d'impôt des entreprises nouvelles | Article désormais identifié (**art. 146**) : reste la portée (« sociétés nouvelles régulièrement créées », IBA des personnes physiques exclue ?) et l'ordre « exercices » vs années d'activité | [#52](https://github.com/cmd-josenhm/amazone-comp/issues/52) |
| Patente | Exonération des 12 premiers mois identifiée (**art. 198**) ; l'article de l'acompte (10 février) et celui du solde (30 avril) restent non identifiés | [#53](https://github.com/cmd-josenhm/amazone-comp/issues/53) |
| e-MECeF : groupes de taxation et codes | Nomenclature technique DGI inventée « de mémoire » ; facture non conforme si faux | [#54](https://github.com/cmd-josenhm/amazone-comp/issues/54) |
| Redevance ORTB : cumul ? | 4 000 FCFA prévus par l'IS (art. 47, 6)), par la TPS (art. 183, 3)) et par la retenue sur salaires : risque de triple comptage | [#55](https://github.com/cmd-josenhm/amazone-comp/issues/55) |
| Exonérations d'AIB : quel article ? | L'**art. 110** (« Sont exemptés de la taxe : … ») semble être l'article des exonérations d'AIB, mais la formule employée et l'insertion exacte dans le chapitre de l'AIB (art. 108 à 136) restent à confirmer | [#59](https://github.com/cmd-josenhm/amazone-comp/issues/59) |

## 5. Sources secondaires utilisées (jamais comme validation)

| Sujet | Source | Remarque |
|---|---|---|
| LF 2026 : mesures (répartition TVA, déduction 90 % BTP, carburant, remboursements) | [Assemblée nationale du Bénin](https://assemblee-nationale.bj/index.php/2025/12/04/assemblee-nationale-la-loi-de-finances-gestion-2026-adoptee-a-lunanimite-des-deputes/), [Banouto](https://www.banouto.bj/economie/article/20251205-budget-general-de-l-etat-exercice-2026-au-benin-les-grandes-orientations-de-la-loi-des-finances-adoptee-sans-les-deputes-de-lopposition), [La Nouvelle Tribune](https://lanouvelletribune.info/2025/12/parlement-beninois-la-loi-de-finances-2026-adoptee-a-lunanimite/), [La Nation (séminaire DGI, janv. 2026)](https://lanation.bj/index.php/economie/loi-de-finances-2026-les-experts-comptables-eclaires-sur-les-nouvelles-regles-fiscales) | Confirment le numéro de la loi (2025-22) et le plafond BTP ; le périmètre « industries » varie selon les articles (issue #32) |
| Panorama 2026 (TVA, AIB, IS, minimum, TPS, ITS, VPS, CNSS, pénalités) | [Yoon — fiscalité des entreprises au Bénin 2026](https://www.yoon.africa/guide-pratique/fiscalite-entreprises-benin) | Site commercial : **ne jamais citer seul**, mais concorde avec le CGI sur l'IS, la TVA et les minima |
| AIB : 5 % pour IFU désactivé (7 septembre 2026) | [Banouto](https://banouto.bj/index.php/economie/article/20260907-recouvrement-d-impots-au-benin-ce-qui-change-a-partir-de-ce-lundi-7-septembre-04-choses-a-retenir), [FAQ DGI relayée](https://banouto.bj/economie/article/20260907-recouvrement-de-l-aib-au-benin-15-questions-reponses-pour-comprendre-ce-qui-change-des-septembre-2026), [Bénin Web TV](https://beninwebtv.com/recouvrement-de-laib-au-benin-ce-qui-change-a-partir-de-ce-lundi-7-septembre/) | Reprend les art. 132 d) et 135 |
| TPS (seuil, 5 %, minimum 10 000) | [Guide simplifié DGI sur la TPS](https://www.impots.finances.gouv.bj/guide-simplifie-sur-la-tps/) | **Ancien** : cite encore 2 % (taux antérieur au CGI 2022) — ne pas s'en servir pour le taux |
| Retenues (loyers, non-résidents, IRCM) | [Rescrits fiscaux DGI 2026](https://api.impots.bj/media/698b0c5f7a212_RESCRITS%20FISCAUX%202026.pdf) | Confirment 20 % non-résidents et l'exclusion des loyers de l'AIB |
| e-MECeF (principe, groupes A–F) | [DDD Invoices — fiscalisation au Bénin](https://dddinvoices.com/learn/fiscalization-and-real-time-reporting-in-benin) | Blog commercial : groupes et codes **de mémoire** → issue #54 |

## 6. Règles de travail qui en découlent

1. **Aucun paramètre sans article.** Un `source` qui ne cite pas un article (ou un arrêté pour les seuils
   fixés par voie réglementaire) est un défaut : le test `toutes_les_sources_citent_un_texte_legal` de
   `ac-fiscal` échoue.
2. **Rien n'est « vérifié » par ce document.** `verifie = false` partout ; seul le chef de projet, après avis
   d'un expert-comptable, passe une valeur à `true`.
3. **Une contradiction non tranchée = une issue.** La liste des sections 3 et 4 est la référence : une PR
   fiscale doit mettre à jour ce tableau **et** la liste des issues ouvertes.
4. **Jamais de réécriture d'une année publiée** : une clé mal nommée (ex. `loyer_bailleur_personne_morale`,
   issue #36) se corrige dans la version suivante du fichier, jamais en place.
