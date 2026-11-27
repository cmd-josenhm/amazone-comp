# Règles fiscales, sociales et comptables du Bénin — guide pour les agents

> **À lire par tous les agents.** Vous ne connaissez pas le droit béninois : ce document vous donne le
> minimum indispensable, avec les **articles exacts**. **Les chiffres exacts ne sont PAS ici** : ils sont
> dans `parametres-fiscaux/2026/*.toml` (règle R2). **Aucun chiffre n'est encore validé par un
> expert-comptable** (`verifie = false` partout). En cas de doute : issue `fiscal-a-verifier` (règle R14).
>
> Révision du **27 novembre 2026** (Lot 1 de A8) : chaque règle ci-dessous est adossée au **CGI 2026**
> (loi n° 2021-15 du 23 décembre 2021, mise à jour de la **loi n° 2025-22 du 8 décembre 2025** portant loi
> de finances pour la gestion 2026). Les points encore incertains portent le numéro de l'issue ouverte ;
> l'état complet des sources est dans `docs/fiscalite/sources.md`.

## 1. Cadre général

- **Pays** : République du Bénin (Afrique de l'Ouest), membre de l'**UEMOA** et de l'**OHADA**.
- **Monnaie** : franc CFA BCEAO (code ISO **XOF**, affiché « FCFA »). Pas de centimes en pratique.
- **Fuseau** : UTC+1, pas d'heure d'été. **Langue officielle** : français.
- **Administration fiscale** : DGI (Direction générale des impôts), portail **e-services** pour déposer les
  déclarations et payer. **Sécurité sociale** : CNSS.
- **Textes** : **CGI**, édition 2026, mis à jour chaque année par la **loi de finances**. La LF 2026 est la
  **loi n° 2025-22 du 8 décembre 2025** (adoptée le 4 décembre 2025) : pas de nouvel impôt, répartition de
  la TVA revue, déduction de TVA sur les carburants modifiée, plancher de l'IBA ramené à 250 000 FCFA.
- **Identifiants** : **IFU** (13 chiffres) obligatoire sur les factures et déclarations ; **RCCM**.

## 2. Comptabilité : SYSCOHADA révisé

- Référentiel obligatoire : **Acte uniforme OHADA relatif au droit comptable et à l'information financière**
  (2017), en vigueur depuis 2018 (comptes personnels) / 2019 (comptes consolidés).
- **Deux systèmes** : **normal** (bilan, compte de résultat, TFT, notes annexes) et **minimal de trésorerie
  (SMT)** pour les très petites entités. Les seuils du SMT sont fixés par l'**Acte uniforme**, pas par le
  CGI : ils ne se confondent pas avec le seuil fiscal de la TPS (**issue #29**).
- Plan de comptes à 9 classes ; principes imposés au logiciel : **partie double**, **intangibilité** des
  écritures validées (correction par contre-passation), **numérotation continue**, **pièce justificative**,
  exercice de 12 mois, conservation **10 ans**.
- **DSF** (déclaration statistique et fiscale) : déposée à l'appui de la déclaration de résultat, au plus
  tard le **30 avril N+1** (**CGI 2026, art. 49 et 50**).

## 3. Régimes d'imposition

| Régime | Qui (CGI 2026) | Ce que le logiciel gère |
|---|---|---|
| **Réel** (IS) | Personnes morales de plein droit : SA, SAS, SARL, SCS, coopératives, etc. (**art. 3, 1)**), plus les personnes morales exerçant une activité industrielle ou commerciale (**art. 3, 2)**) | TVA mensuelle, AIB, IS + minimum de perception, acomptes, retenues, ITS/VPS, DSF système normal |
| **Réel** (IBA) | Personnes physiques accomplissant habituellement, pour leur propre compte, une activité à caractère lucratif (**art. 54**) | Idem avec l'IBA (30 % / minimum 1,5 %) à la place de l'IS |
| **TPS** | Contribuables relevant de l'IBA dont le CA ne dépasse pas un **seuil fixé par arrêté** (**art. 178**), hors sociétés de capitaux | Impôt unique de **5 %** des recettes (minimum 10 000 FCFA) **libératoire** de l'IBA, des patentes, des licences et du **VPS** ; pas de TVA facturée |

> **Point ouvert majeur — issue #28.** Le CGI 2026 consulté **ne comporte pas** de régime intitulé « réel
> simplifié » : les personnes physiques relèvent de l'IBA (ou de la TPS sous le seuil) et les personnes
> morales de l'IS. Les dispositions de « régime du chiffre d'affaires réel simplifié » (déclaration
> trimestrielle sur demande) appartiennent au CGI 2010, pas aux versions récentes. **Le régime
> `ReelSimplifie` du contrat C11 n'est donc pas attesté en droit** : en attendant l'arbitrage du chef de
> projet (ADR éventuel), le logiciel ne doit afficher aucune obligation spécifique « réel simplifié » qui
> ne soit pas aussi celle du réel.
>
> **Deuxième point (art. 178)** : le régime dépend du **seuil**, mais aussi des **achats** de biens,
> services et équipements et des **contrats** signés — un contribuable à la TPS bascule au réel dès le
> 1er jour du mois suivant le dépassement ; à l'inverse, il ne revient à la TPS qu'après **deux exercices
> consécutifs** sous le seuil. L'assistant de démarrage (A3) et `regime::suggerer` (A8) doivent tenir
> compte de cette double condition.

Le régime choisi conditionne **tout** : facturation (TVA ou non), écrans, déclarations, calendrier.

## 4. TVA (CGI 2026, livre 2, titre 1, chapitre 1 — art. 223 à 262)

**Taux.** Le taux de droit commun est **18 %** (**art. 241**). Les exportations relèvent d'un régime
particulier (taux zéro ou exonération : **issue #51**) : la distinction change le droit à déduction, donc
elle sera tranchée avant tout calcul d'exportation.

**Assujettissement.** Art. **228** : les personnes dont le CA ou les recettes atteignent le **seuil fixé par
arrêté** du ministre des finances. Le montant du seuil **n'est pas dans le code** (issue #39) ; le franchir
en cours d'année rend la TVA exigible dès le 1er jour du mois suivant.

**Fait générateur (art. 235).** Livraison pour les ventes, échanges et travaux à façon ; exécution des
services et travaux pour les prestations ; première utilisation pour les livraisons à soi-même. La
constatation **ne peut jamais être postérieure à l'établissement d'une facture** totale ou partielle.

**Exigibilité (art. 236) — règle à appliquer telle quelle dans le logiciel** :

| Opération | Exigibilité |
|---|---|
| Ventes et livraisons de biens (y compris livraisons à soi-même) | **à la réalisation du fait générateur** (livraison) |
| Prestations de services | **à l'encaissement** du prix, des acomptes ou avances |
| Travaux immobiliers | **à l'encaissement** |
| Marchés publics de l'État, des collectivités locales et des sociétés, établissements et offices de l'État | **à l'encaissement** |
| Tous les autres cas | à la réalisation du fait générateur |

**Acomptes et avances (art. 236, 2)).** Pour toutes les opérations autres que les importations, le versement
d'une avance ou d'un acompte rend la taxe **exigible sur son montant**, que l'opération soit réalisée ou non.
Conséquence pour A4 et A6 : une facture d'acompte doit porter la TVA, et l'encaissement d'un acompte sur une
vente de biens rend la TVA exigible même si la livraison n'a pas eu lieu.

**Option « débits » (issue #30).** Le CGI permet aux redevables imposables à l'encaissement d'être autorisés
à payer la TVA d'après les débits (autorisation expresse du Directeur général des impôts). Le texte exact de
l'autorisation n'a pas été localisé au Lot 1 : **le logiciel ne propose pas encore cette option**.

**Déclaration et crédit.** Déclaration au plus tard le **10 de chaque mois** pour le mois précédent
(**art. 259, 1)**) ; l'excédent de déduction est un **crédit de TVA imputable** sur la taxe des périodes
suivantes, sans limitation de délai (**art. 252**). Une entreprise n'a donc jamais de « TVA due négative » :
elle a un crédit à reporter.

**Déduction.** Droit à déduction sur facture normalisée. Depuis la LF 2026 (**art. 247, 2)**) la TVA d'amont
sur les produits pétroliers pour cuves, le gaz combustible des usines, le gasoil, les huiles et graisses des
entreprises industrielles et du BTP est déductible ; les frais de carburant des véhicules ne le sont pas,
sauf véhicules affectés exclusivement au transport public assujetti à la TVA. **Pour les entreprises du BTP
la déduction est plafonnée à 90 %** de la TVA de la facture d'achat. L'extension du plafond aux industries,
à la logistique et aux déchets est une **contradiction ouverte** (**issue #32**) : le paramètre porte la
valeur 0,90 avec une `note` et reste `verifie = false`.

**Retenue de TVA à la source (art. 263).** Les structures désignées (État, collectivités, sociétés
publiques, entreprises désignées par la DGI) retiennent une fraction de la TVA facturée : 40 % pour les
fournisseurs au réel, 100 % pour les fournisseurs à la TPS et pour les prestataires non-résidents
(**issue #35** — seul le taux de 40 % est adossé à un texte d'application identifié). Les attestations de
retenue permettent au fournisseur de l'imputer sur sa déclaration.

**Prorata.** Un assujetti qui réalise à la fois des opérations taxables et des opérations exonérées ne
déduit la TVA d'amont qu'au prorata de son chiffre d'affaires taxable (**issue #51** pour l'articulation avec
les exportations). Le module ne calcule pas encore ce prorata : il sera livré avec la déclaration de TVA
(Lot 4).

## 5. AIB (acompte sur impôt assis sur les bénéfices) — art. 130 à 136

- **Taux (art. 132, 2))** : **1 %** sur les importations, les achats commerciaux et les fournitures de
  travaux des entreprises immatriculées à l'IFU, ainsi que sur les fournitures à l'État et aux entreprises
  publiques ; **3 %** sur les **prestations de services** des personnes immatriculées à l'IFU ; **5 %** sur
  les opérations des personnes **non immatriculées** (et sur les factures normalisées des personnes non
  connues du fichier, art. 132, 2) d)).
- **Base** : prix **hors TVA** ; à l'importation, valeur en douane majorée des droits et taxes exigibles,
  à l'exception de la TVA.
- **Qui prélève** : le vendeur ou le prestataire sur son client, et le service des douanes à l'importation.
- **Reversement (art. 133, 3))** : au plus tard le **10 du mois suivant** la facturation ou le prélèvement ;
  la déclaration est obligatoire **même sans prélèvement** dans le mois.
- **Imputation (art. 133, 4) et 5))** : l'AIB subi est un **acompte imputable** sur l'IS ou l'IBA ; pour un
  contribuable à la TPS, il est **déductible du solde de la TPS** affectée au budget national.
- **IFU désactivé** : depuis le 7 septembre 2026, la DGI applique **automatiquement 5 %** d'AIB sur les
  factures des contribuables dont l'IFU est désactivé dans CFISC (communiqué du 3 septembre 2026). L'AIB
  ainsi payé reste imputable ; il n'est pas remboursé.
- **Loyers exclus** : les loyers d'immeubles ne relèvent **pas** de l'AIB mais de la retenue de l'art. 106
  (art. 131, 7)) — ne jamais étiqueter un loyer en `AibSupporte`.
- **Exonérations (art. 110, à confirmer — issue #59)** : « Sont exemptés de la taxe : … 4) les entreprises
  nouvelles justifiant qu'elles relèvent de la TPS, au titre de leurs douze (12) premiers mois
  d'activités ; 5) les personnes physiques ou morales justifiant d'une exonération de l'IR ou de la TPS ».
  En pratique : une entreprise nouvelle à la TPS ne supporte pas d'AIB pendant ses douze premiers mois,
  et une entreprise exonérée de l'IR ou de la TPS ne peut pas se voir prélever d'AIB. **La position exacte
  de l'art. 110 reste à confirmer** (le mot « la taxe » y est ambigu) : tant que l'issue #59 n'est pas
  tranchée, le paramètre reste `verifie = false`.
- **Codes e-MECeF** : A = 1 %, B = 5 % seulement ; **aucun code connu pour 3 %** (**issue #24**) — A4 ne doit
  pas inventer de code.

## 6. IS et IBA

**IS (art. 2 à 53).** Champ : **art. 3** ; exonérations : **art. 4**. Taux (**art. 46, 1)**) : **25 %** pour
les personnes morales ayant une activité industrielle (hors industries extractives) et les écoles privées ;
**30 %** pour les autres. Le taux industriel de 25 % ne figurait pas dans les CGI 2023-2024 : sa date
d'entrée en vigueur doit être établie (**issue #33**).

**IBA (art. 54 à 68).** Champ : **art. 54** ; taux : **30 %** (**art. 63, 1)**), ramené à **25 %** pour les
établissements privés d'enseignement (**art. 63, 2)**).

**Minimum de perception.** On paie le **plus élevé** entre l'impôt calculé sur le bénéfice et le minimum
assis sur les **produits encaissables** :
- IS : **1 %** (**art. 47, 1)**), **3 %** pour le BTP, **10 %** pour les entreprises à prépondérance
  immobilière ;
- IBA : **1,5 %** (**art. 64, 1)**), **3 %** BTP et **10 %** immobilier (**art. 64, 2)**) ;
- **plancher « en tout état de cause » : 250 000 FCFA** pour l'IS (**art. 47, 3)**) et pour l'IBA
  (**art. 64, 3) a)**, modifié par la LF 2026 — c'était 500 000 FCFA au titre de 2025).
- Les **produits encaissables** sont définis à l'**art. 47, 2)** : tous les produits ayant donné lieu (ou
  susceptibles de donner lieu) à un flux, y compris financiers et hors activité ordinaire, **à l'exclusion**
  de la production immobilisée, de la production stockée, des transferts de charges et des reprises de
  provisions et d'amortissements. Une entreprise déficitaire paie donc quand même le minimum.

**Redevance ORTB.** **4 000 FCFA** s'ajoutent à l'IS, payables avec l'acompte du **10 mars** (**art. 47, 6)**).
Le même montant existe pour la TPS (**art. 183, 3)**) et `social.toml` prévoit une retenue audiovisuelle sur
salaires : le **cumul** des trois n'est pas documenté (**issue #55**).

**Acomptes et solde — mode de calcul (art. 49 et 51).**
1. La déclaration de résultat est déposée au plus tard le **30 avril** (**art. 49, 1)**).
2. L'impôt est payé en **quatre acomptes**, au plus tard le **10 mars, 10 juin, 10 septembre et
   10 décembre** (**art. 51, 1)**).
3. **Chaque acompte = le quart de l'impôt dû l'année précédente** (**art. 51, 2)**).
4. Si la déclaration de l'exercice précédent n'est pas encore déposée au 10 mars, le premier acompte est
   calculé **provisoirement sur l'impôt de l'avant-dernier exercice**, puis **régularisé au deuxième
   acompte**.
5. Le **solde** est acquitté **le jour du dépôt de la déclaration** (**art. 51, 3)**) ; des acomptes
   excédentaires deviennent un **crédit d'impôt** imputable sur les échéances suivantes.
6. Une **demande en réduction d'acomptes** est possible auprès du Directeur général des impôts
   (**art. 51, 4) à 6)**) : non gérée en V1 (**issue #34**).

**Résultat fiscal.** Résultat comptable **+ réintégrations − déductions**. Les déficits sont reportables
**cinq exercices** (**art. 43**) ; les amortissements différés d'une période déficitaire sont assimilés à des
déficits (art. 43, 6)).

**Entreprises nouvelles (art. 146, modifié par la LF 2023).** L'IS ou l'IBA dû par les **sociétés
nouvelles régulièrement créées** est réduit de **25 %** au titre du premier exercice, **25 %** au titre du
second et **50 %** au titre du troisième (**art. 146, 1)**). La réduction **ne s'applique pas** en cas de
rappel de droits suite à une procédure de contrôle fiscal (**art. 146, 2) a)**). Attention : le texte vise
les **sociétés** et parle d'**exercices**, pas d'années d'activité ; l'IBA due par une personne physique
nouvelle relève d'autres dispositions (**issue #52**). Les exonérations jumelles sont l'**art. 180, 3)**
(TPS, 12 premiers mois) et l'**art. 198** (patentes, 12 premiers mois).

## 7. TPS (taxe professionnelle synthétique) — art. 178 à 190

- **Champ (art. 178)** : contribuables relevant de l'IBA dont le CA ne dépasse pas le **seuil fixé par
  arrêté** (valeur provisoire 50 000 000 FCFA, **issue #39**) ; la taxe est **libératoire** de l'IBA, de la
  contribution des patentes, de la contribution des licences et du **versement patronal sur salaires**.
- **Montant (art. 183)** : **5 %** des recettes annuelles, **minimum 10 000 FCFA**, **redevance de
  4 000 FCFA** en sus, et la taxe est due **par commune et par établissement** (un utilisateur avec deux
  établissements dans deux communes doit deux TPS).
- **Acomptes et solde (art. 184 et 185)** : deux acomptes calculés sur l'impôt de l'année précédente, dans
  les **dix premiers jours de février et de juin** ; déclaration et solde au **30 avril**, avec imputation
  des acomptes **et des AIB subis**.
- **Exonération** des douze premiers mois d'activité des entreprises nouvelles (**art. 180, 3)**) : la taxe
  est due à partir du premier jour du mois suivant l'expiration de la période d'exonération, et les
  entreprises devenues imposables pour la première fois paient l'**impôt minimum** (art. 183, 2)) dans les
  dix premiers jours du mois suivant. Le champ exact de « entreprises nouvelles régulièrement créées »
  reste à confirmer (**issue #39**). L'exonération de la **contribution des patentes** pour la même durée
  est l'**art. 198**.

## 8. Retenues à la source

| Retenue | Taux | Base | Échéance | Article |
|---|---|---|---|---|
| **Loyers** (bailleur au réel) | **12 %** | montant brut du loyer | 10 du mois suivant | **art. 106, 1)** |
| **Loyers** (bailleur relevant de l'IBA ou de l'IS) | **10 %** | montant brut du loyer | 10 du mois suivant | **art. 106, 1)** |
| **Prestataires non-résidents** | **20 %** | montant HT, frais accessoires compris | 10 du mois suivant | **art. 141 et 142** |
| **IRCM — dividendes de non-résidents** | **5 %** | montant distribué | 10 du mois suivant | **art. 86, 1)** |
| **IRCM — autres dividendes, bénéfices d'ES** | **10 %** | montant distribué | 10 du mois suivant | **art. 86, 2)** |
| **IRCM — autres revenus (intérêts…)** | **15 %** | montant des intérêts | 10 du mois suivant | **art. 86, 3)** |

Points d'attention :
- **art. 106, 2)** : les **locataires personnes physiques sont exemptés** de la retenue, sauf s'ils louent par
  l'intermédiaire d'une entité soumise à l'impôt sur les bénéfices ;
- le critère du taux réduit de 10 % est le **régime du bailleur** (IBA ou IS), pas sa forme juridique
  (**issue #36**) ;
- le taux de 20 % sur les non-résidents est **unique depuis le 1er janvier 2023** (rescrit DGI) ; les
  contrats enregistrés avant la LF 2023 gardent 10 % (personnes physiques) ou 12 % (personnes morales) ;
  les conventions fiscales peuvent réduire le taux (**issue #37**).

## 9. Impôts sur salaires (détail : A7, `social.toml`)

- **ITS** : barème mensuel progressif, retenu par l'employeur et reversé le **10 du mois suivant**. Le
  premier seuil (60 000 ou 50 000 FCFA) et les tranches ne sont **pas** confirmés : **issues #22 et #43**
  (propriété de A7), relayées ici parce qu'elles conditionnent aussi les lignes `Its` de la déclaration
  récapitulative (Lot 5).
- **VPS** : à la charge de l'employeur, **non dû** par les contribuables assujettis à la TPS
  (**art. 178 et 192**).
- **CNSS** : relève du **Code de sécurité sociale**, pas du CGI. Plafond d'assiette (**issue #23**) et date
  limite (**issue #27**, c'est le paramètre `CNSS` du `calendrier.toml`) ne sont pas confirmés.
- **Redevance audiovisuelle sur salaires** : mécanisme non documenté (**issue #26**) ; ne pas la confondre
  avec la redevance ORTB de l'IS et de la TPS (**issue #55**).

## 10. Calendrier : les dates que le logiciel doit connaître

| Échéance | Date | Article |
|---|---|---|
| Déclaration et paiement de la TVA du mois précédent | **10 de chaque mois** | **art. 259, 1)** |
| Déclaration et reversement de l'AIB | **10 du mois suivant** | **art. 133, 3)** |
| ITS et VPS | **10 du mois suivant** le paiement du salaire | titre 1, chap. 6 ; titre 3, chap. 2 |
| Retenues (loyers, non-résidents, IRCM) | **10 du mois suivant** | **art. 106, 4)** ; **art. 90** |
| Acomptes d'IS / IBA | **10 mars, 10 juin, 10 septembre, 10 décembre** | **art. 51, 1)** |
| Redevance ORTB (IS) | avec l'acompte du **10 mars** | **art. 47, 6)** |
| Déclaration de résultat, DSF et solde | **30 avril N+1** | **art. 49, 1)** ; **art. 51, 3)** |
| Acomptes de TPS | **10 février et 10 juin** | **art. 185, 1)** |
| Déclaration annuelle et solde de TPS | **30 avril N+1** | **art. 184, 1)** |
| Patente (acompte 50 %, puis solde) | **10 février**, puis **30 avril** | art. 198 (exonération 12 mois) ; acompte et solde à identifier (**issue #53**) |
| CNSS | jour **provisoire : 15** | Code de sécurité sociale (**issue #27**) |

## 11. Arrondis et précision des calculs

Le CGI 2026 **ne fixe aucune règle d'arrondi** pour la TVA ; la seule règle d'arrondi écrite concerne
l'assiette de l'impôt sur le revenu (revenu arrondi au millier inférieur, **art. 181**), qui ne s'applique
pas à la TVA. En conséquence :

1. le logiciel n'a **pas** de règle d'arrondi « officielle » à prétendre appliquer : il applique une règle
   **explicite, paramétrée et vérifiable**, documentée à chaque calcul (règle R9) ;
2. la règle retenue aujourd'hui est l'arrondi **au franc le plus proche, ligne par ligne**
   (`ac_fiscal::tva::depuis_ht`, `Arrondi::PlusProche`), et la ventilation est bornée par
   `ht + tva = ttc` : tout arrondi se corrige sur la ligne de TVA, jamais sur le HT ;
3. cette règle doit être confrontée à la **spécification e-MECeF** (le TTC certifié par la DGI doit être
   identique au nôtre) : **issue #31** ;
4. montants toujours en **francs entiers** (`ac_noyau::Montant`), taux en **décimal exact**
   (`ac_noyau::Taux`) : aucun flottant, jamais (règle R1).

## 12. Facture normalisée (MECeF / e-MECeF)

- **art. 256** : toute opération d'un redevable doit faire l'objet d'une **facture normalisée** (ou d'un
  document en tenant lieu : décompte de travaux, feuilles d'honoraires, acomptes), **émise et transmise par
  une machine électronique certifiée de facturation**. La DGI autorise une version **dématérialisée
  (e-MECeF)**.
- La DGI renvoie des **éléments de sécurité** imprimés : code MECeF/DGI, **NIM**, compteurs, date et heure,
  **QR code**.
- Une facture ne peut être **ni modifiée ni supprimée** : on émet un **avoir** référencant la facture
  d'origine.
- Les **groupes de taxation A à F**, les **types de facture** et les **modes de paiement** sont une
  nomenclature technique DGI que nous n'avons pas : les valeurs de `emecef.toml` viennent d'une source
  commerciale et restent `verifie = false` (**issue #54**). A4 ne doit pas émettre de facture « réelle »
  avant confirmation — le simulateur porte la mention « SIMULATION — NON VALABLE FISCALEMENT ».

## 13. Moyens de paiement au Bénin

Espèces (caisse), banques (virements, chèques), **Mobile Money** très répandu : **MTN MoMo**, **Moov Money**,
**Celtiis Cash**. Chaque portefeuille Mobile Money est un compte de trésorerie distinct (frais de
transaction à comptabiliser). Les relevés s'importent en CSV.

## 14. Ce que le logiciel ne fait PAS

- Il **ne dépose pas** les déclarations : il produit des **PDF et Excel** au format attendu, que
  l'utilisateur dépose lui-même sur **e-services**.
- Il ne donne pas de conseil fiscal personnalisé ; il affiche les règles et leurs sources.
- Il ne gère qu'**une entreprise par installation**, en **FCFA**, au **Bénin**.
- Il ne calcule pas encore : le prorata de déduction, la patente, les droits d'enregistrement, la TVM, la
  taxe foncière (rappels au calendrier seulement).

Sources consultées et état de chaque contradiction : `docs/fiscalite/sources.md`.
