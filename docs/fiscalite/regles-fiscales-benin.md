# Règles fiscales, sociales et comptables du Bénin — guide pour les agents

> **À lire par tous les agents.** Vous ne connaissez pas le droit béninois : ce document vous donne
> le minimum indispensable. **Les chiffres exacts ne sont PAS ici** : ils sont dans
> `parametres-fiscaux/2026/*.toml` (règle R2). **Aucun chiffre n'est encore validé par un
> expert-comptable** (`verifie = false` partout). En cas de doute : issue `fiscal-a-verifier` (règle R14).

## 1. Cadre général

- **Pays** : République du Bénin (Afrique de l'Ouest), membre de l'**UEMOA** et de l'**OHADA**.
- **Monnaie** : franc CFA BCEAO (code ISO **XOF**, affiché « FCFA »). Pas de centimes en pratique.
- **Fuseau** : UTC+1, pas d'heure d'été. **Langue officielle** : français.
- **Administration fiscale** : DGI (Direction générale des impôts), portail **e-services** pour déposer les déclarations et payer.
- **Sécurité sociale** : CNSS (Caisse nationale de sécurité sociale).
- **Textes** : Code général des impôts (**CGI**, édition 2026) modifié chaque année par la **loi de finances** (LF 2026 adoptée le 4 décembre 2025 ; aucun nouvel impôt, pas de changement des taux principaux selon la presse).
- **Identifiant fiscal** : **IFU** (Identifiant fiscal unique), 13 chiffres, obligatoire sur les factures et les déclarations.
- **Registre** : RCCM (Registre du commerce et du crédit mobilier).

## 2. Comptabilité : SYSCOHADA révisé

- Référentiel obligatoire : **Acte uniforme OHADA relatif au droit comptable et à l'information financière** (2017), appelé **SYSCOHADA révisé**, en vigueur depuis 2018 (comptes personnels) / 2019 (comptes consolidés).
- **Deux systèmes** :
  - **Système normal** : bilan, compte de résultat, tableau des flux de trésorerie (**TFT**), notes annexes ;
  - **Système minimal de trésorerie (SMT)** : pour les très petites entités (seuils de chiffre d'affaires fixés par l'Acte uniforme selon l'activité — à paramétrer et vérifier) ; comptabilité de trésorerie (recettes / dépenses) et états simplifiés.
- **Plan de comptes** à 9 classes : 1 ressources durables, 2 actif immobilisé, 3 stocks, 4 tiers, 5 trésorerie, 6 charges des activités ordinaires, 7 produits des activités ordinaires, 8 autres charges et produits (dont HAO et impôt sur le résultat), 9 engagements / analytique.
- Principes imposés au logiciel : **partie double**, **intangibilité** des écritures validées (correction par contre-passation), **numérotation continue**, **pièce justificative** pour chaque écriture, **exercice de 12 mois** (en général du 1er janvier au 31 décembre), conservation **10 ans**.
- **DSF** (déclaration statistique et fiscale) : liasse d'états financiers + tableaux annexes, déposée avec la déclaration de résultat (au plus tard le 30 avril N+1 — à vérifier).

## 3. Régimes d'imposition

| Régime | Qui | Ce que le logiciel gère |
|---|---|---|
| **Réel normal** | Sociétés et entreprises au-delà du seuil | TVA mensuelle, AIB, IS ou IBA + minimum de perception, acomptes, retenues, ITS/VPS, DSF système normal |
| **Réel simplifié** | Entreprises intermédiaires (définition exacte à vérifier dans le CGI 2026) | Idem avec obligations allégées à préciser par A8 |
| **TPS** | Personnes physiques dont le CA HT ≤ seuil (50 000 000 FCFA — à vérifier) ; les sociétés de capitaux en sont exclues | Pas de TVA facturée ; impôt unique = % du CA avec minimum ; acomptes ; souvent SMT |

Le régime est choisi dans l'assistant de démarrage et conditionne **tout** : facturation (TVA ou non), écrans, déclarations, calendrier.

## 4. Impôts et taxes à traiter

### 4.1 TVA (taxe sur la valeur ajoutée)
- Taux normal unique (18 %), exportations à 0 %, opérations exonérées listées par le CGI.
- **TVA collectée** sur les ventes − **TVA déductible** sur les achats = TVA à payer (ou **crédit de TVA** reportable).
- Déclaration et paiement **mensuels** (au plus tard le 10 du mois suivant).
- Points spéciaux : **retenue de TVA à la source** par certains clients (État, sociétés désignées) ; **limitation de déduction à 90 % pour le BTP et l'industrie** (LF 2026, à confirmer) ; TVA sur carburant déductible si lié à l'objet social (LF 2026).
- **Exigibilité** : par défaut à la livraison pour les biens et à l'encaissement pour les services (À VÉRIFIER : A8 doit confirmer et paramétrer).

### 4.2 AIB (acompte sur impôt assis sur les bénéfices)
- Prélèvement fait **par le vendeur sur son client** (et à l'importation) au moment de la facturation, puis reversé à la DGI le 10 du mois suivant.
- Taux selon que le client est immatriculé à l'IFU ou non, et selon la nature (biens / services / marchés publics) : 1 %, 3 %, 5 % (à vérifier). Base : prix hors TVA.
- L'AIB **subi** par l'entreprise (prélevé par ses fournisseurs) est un **crédit d'impôt** imputable sur l'IS / IBA / TPS.
- Sur la facture normalisée, l'AIB apparaît sous un code (A = 1 %, B = 5 % — à confirmer).

### 4.3 IS (impôt sur les sociétés) et IBA (impôt sur le bénéfice d'affaires, personnes physiques au réel)
- Taux de l'IS : 30 % général, 25 % industrie et écoles privées (à vérifier). IBA : à vérifier.
- **Minimum de perception** : un pourcentage du produit encaissable (chiffre d'affaires), avec un plancher en FCFA ; on paie le plus élevé entre l'impôt calculé et le minimum.
- **Acomptes** trimestriels (10 mars, 10 juin, 10 septembre, 10 décembre) ; solde avec la déclaration de résultat.
- Le logiciel calcule le **résultat fiscal** à partir du résultat comptable + réintégrations − déductions (tableau à saisir avec aide).

### 4.4 TPS (taxe professionnelle synthétique)
- Impôt synthétique des petites entreprises : un pourcentage du CA annuel, avec un minimum, plus une redevance audiovisuelle. Remplace IBA, patente, VPS (à vérifier).
- Acomptes (10 février, 10 juin) et solde au 30 avril N+1.

### 4.5 Retenues à la source
- **Loyers** versés : retenue sur le loyer, taux différent selon que le bailleur est une personne physique ou morale.
- **Prestataires non-résidents** : retenue sur le montant HT.
- **Revenus de capitaux mobiliers (IRCM)** : dividendes, intérêts (taux à documenter).
- Reversement le 10 du mois suivant.

### 4.6 Impôts sur salaires
- **ITS** (impôt sur les traitements et salaires) : **barème mensuel progressif par tranches**, retenu par l'employeur sur le salaire et reversé le 10 du mois suivant. Redevance audiovisuelle sur certains mois (à vérifier).
- **VPS** (versement patronal sur salaires) : à la charge de l'employeur, % de la masse salariale (taux réduit pour l'enseignement privé ; non dû par les entreprises à la TPS).

### 4.7 CNSS
- Cotisations sur les salaires : **prestations familiales** (employeur), **pension** (employeur + salarié), **risques professionnels** (employeur, taux selon l'activité).
- Plafond éventuel de l'assiette, périodicité (mensuelle / trimestrielle) et date limite : **À VÉRIFIER** (contradictions entre sources).
- **SMIG** : salaire minimum (52 000 FCFA mensuels selon nos sources — à confirmer).

### 4.8 Autres (rappels seulement en V1)
- **Patente** (contribution des patentes) : acompte en février, solde au 30 avril. Calcul non automatisé en V1, rappel au calendrier.
- **Taxe foncière**, **taxe sur les véhicules à moteur (TVM)** : rappels.

## 5. Facture normalisée (MECeF / e-MECeF)

- Au Bénin, les entreprises au réel doivent émettre des **factures normalisées** : chaque facture est enregistrée auprès de la DGI, soit par un boîtier **MECeF** (machine électronique certifiée de facturation), soit en ligne via **e-MECeF** (API de la DGI).
- La DGI renvoie des **éléments de sécurité** imprimés sur la facture : **code MECeF/DGI**, **NIM** (numéro d'identification de la machine), **compteurs**, **date et heure**, **QR code**.
- Une facture ne peut **pas être modifiée** ni supprimée : on émet une **facture d'avoir** qui référence la facture d'origine.
- Chaque article porte un **groupe de taxation** (A à F) qui indique son traitement TVA.
- **Aujourd'hui, nous n'avons pas l'accès à l'API** : le logiciel utilise un **simulateur** et imprime « SIMULATION — NON VALABLE FISCALEMENT ». L'architecture (trait `Normaliseur`) permet de brancher l'API réelle sans rien changer ailleurs.

## 6. Moyens de paiement au Bénin

Espèces (caisse), banques (virements, chèques), **Mobile Money** très répandu : **MTN MoMo**, **Moov Money**, **Celtiis Cash**. Chaque portefeuille Mobile Money est un compte de trésorerie distinct (frais de transaction à comptabiliser). Les relevés s'importent en CSV.

## 7. Ce que le logiciel ne fait PAS

- Il **ne dépose pas** les déclarations : il produit des **PDF et Excel** au format attendu, que l'utilisateur dépose lui-même sur **e-services**.
- Il ne donne pas de conseil fiscal personnalisé ; il affiche les règles et leurs sources.
- Il ne gère qu'**une entreprise par installation**, en **FCFA**, au **Bénin**.

Sources consultées : `docs/fiscalite/sources.md`.
