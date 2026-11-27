//! Tests d'intégration de la plateforme (issue n° 12).
//!
//! Ces tests ont besoin d'un **PostgreSQL** réel : la variable `DATABASE_URL` doit
//! désigner une base de test (chaque test reçoit sa propre base vide). En intégration
//! continue, le workflow `plateforme.yml` fournit le service PostgreSQL.

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use chrono::Months;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

use amazone_plateforme::{
    application,
    courriel::EnvoyeurMemoire,
    jetons::{verifier_jeton, SignataireLicences},
    Etat,
};

const CLE_TEST: [u8; 32] = [7u8; 32];

fn signataire_test() -> SignataireLicences {
    SignataireLicences::nouveau(CLE_TEST)
}

/// Construit l'application de test (migrations comprises, transport en mémoire).
async fn app(pool: PgPool) -> (Router, Arc<EnvoyeurMemoire>) {
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrations");
    let env = Arc::new(EnvoyeurMemoire::nouveau());
    let etat = Etat::nouveau(
        pool,
        signataire_test(),
        env.clone(),
        "test@localhost".to_owned(),
        2,
    );
    (application(etat), env)
}

fn req_json(methode: &str, uri: &str, corps: &Value, jeton: Option<&str>) -> Request<Body> {
    let mut constructeur = Request::builder()
        .method(methode)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(j) = jeton {
        constructeur = constructeur.header("authorization", format!("Bearer {j}"));
    }
    constructeur
        .body(Body::from(corps.to_string()))
        .expect("construction de la requête")
}

async fn envoyer(app: &Router, requete: Request<Body>) -> (StatusCode, Value) {
    let reponse = app.clone().oneshot(requete).await.expect("réponse HTTP");
    let statut = reponse.status();
    let corps = to_bytes(reponse.into_body(), usize::MAX).await.expect("corps");
    let valeur = if corps.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&corps).expect("JSON de réponse")
    };
    (statut, valeur)
}

fn fiche_valide(ifu: &str) -> Value {
    json!({
        "raison_sociale": "Boutique Exemple SARL",
        "nom_commercial": "Boutique Exemple",
        "sigle": null,
        "forme_juridique": "SARL",
        "date_creation": "2021-03-15",
        "ifu": ifu,
        "rccm": "RB/COT/21 B 12345",
        "numero_cnss": null,
        "regime": "REEL_SIMPLIFIE",
        "centre_impots": "CIME Cotonou",
        "secteur": "COMMERCE",
        "activite_principale": "Vente de fournitures de bureau",
        "effectif": 0,
        "departement": "LITTORAL",
        "commune": "Cotonou",
        "adresse": "Quartier Ganhi, rue 123",
        "boite_postale": null,
        "telephone": "01 97 12 34 56",
        "courriel": "contact@exemple.bj",
        "site_web": null,
        "responsable": {
            "nom": "AHOUANSOU",
            "prenoms": "Marie",
            "fonction": "Gérante",
            "telephone": "+229 01 66 00 11 22",
            "courriel": "marie@exemple.bj"
        }
    })
}

/// Crée un compte vérifié et ouvre une session ; renvoie le jeton d'accès.
async fn compte_verifie(app: &Router, env: &EnvoyeurMemoire, courriel: &str) -> String {
    let (statut, _) = envoyer(
        app,
        req_json(
            "POST",
            "/v1/comptes",
            &json!({
                "courriel": courriel,
                "mot_de_passe": "un-mot-de-passe-solide",
                "nom": "Marie AHOUANSOU"
            }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED, "création du compte");
    let code = env.dernier_code(courriel).expect("code de vérification envoyé");
    let (statut, _) = envoyer(
        app,
        req_json(
            "POST",
            "/v1/comptes/verifications",
            &json!({ "courriel": courriel, "code": code }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::OK, "vérification du courriel");
    let (statut, corps) = envoyer(
        app,
        req_json(
            "POST",
            "/v1/sessions",
            &json!({ "courriel": courriel, "mot_de_passe": "un-mot-de-passe-solide" }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::OK, "connexion");
    corps["jeton"].as_str().expect("jeton de session").to_owned()
}

/// Accepte les CGU courantes puis inscrit l'entreprise ; renvoie son identifiant.
async fn entreprise_inscrite(app: &Router, jeton: &str, ifu: &str) -> i64 {
    let (statut, corps) = envoyer(
        app,
        req_json("GET", "/v1/cgu/courante", &json!({}), None),
    )
    .await;
    assert_eq!(statut, StatusCode::OK, "CGU courantes");
    let version = corps["version"].as_i64().expect("version des CGU");
    let (statut, _) = envoyer(
        app,
        req_json(
            "POST",
            "/v1/cgu/acceptations",
            &json!({ "version": version }),
            Some(jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED, "acceptation des CGU");
    let (statut, corps) = envoyer(
        app,
        req_json(
            "POST",
            "/v1/entreprises",
            &json!({ "fiche": fiche_valide(ifu) }),
            Some(jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED, "inscription de l'entreprise");
    assert_eq!(corps["statut"], "EN_ATTENTE_VERIFICATION");
    corps["entreprise_id"].as_i64().expect("identifiant")
}

#[sqlx::test]
async fn parcours_complet_inscription_verification_connexion_entreprise_licence(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton = compte_verifie(&app, &env, "marie@exemple.bj").await;
    let entreprise_id = entreprise_inscrite(&app, &jeton, "3201912345678").await;

    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/licences",
            &json!({ "ifu": "3201912345678", "empreinte_poste": "empreinte-poste-a" }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED, "émission de la licence");
    assert_eq!(corps["offre"], "GRATUITE");

    // Le jeton est vérifiable hors ligne avec la clé publique, exactement comme le fera
    // l'application (lot 2) ; un octet modifié est refusé.
    let jeton_licence = corps["jeton"].as_str().expect("jeton de licence");
    let charge = verifier_jeton(jeton_licence, &signataire_test().cle_publique_hex())
        .expect("jeton de licence valide");
    assert_eq!(charge.ifu, "3201912345678");
    assert_eq!(charge.empreinte_poste, "empreinte-poste-a");
    assert_eq!(charge.offre, "GRATUITE");
    assert_eq!(charge.format, 1);
    assert_eq!(
        charge.expire_le,
        charge.emise_le.checked_add_months(Months::new(6)).expect("6 mois")
    );
    let mut altere = jeton_licence.to_owned();
    altere.replace_range(0..1, if &jeton_licence[0..1] == "A" { "B" } else { "A" });
    assert!(verifier_jeton(&altere, &signataire_test().cle_publique_hex()).is_err());

    // L'état de l'entreprise est consultable par son compte.
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "GET",
            &format!("/v1/entreprises/{entreprise_id}/statut"),
            &json!({}),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps["statut"], "EN_ATTENTE_VERIFICATION");
}

#[sqlx::test]
async fn fiche_invalide_renvoie_la_meme_liste_erreurs(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton = compte_verifie(&app, &env, "pierre@exemple.bj").await;
    let (statut, _) = envoyer(
        &app,
        req_json("POST", "/v1/cgu/acceptations", &json!({ "version": 1 }), Some(&jeton)),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED);

    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/entreprises",
            &json!({
                "fiche": serde_json::to_value(
                    ac_noyau::entreprise::FicheEntreprise::default()
                ).expect("sérialisation de la fiche vide")
            }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::UNPROCESSABLE_ENTITY);
    // La liste « erreurs » est exactement celle de FicheEntreprise::valider().
    let attendu = ac_noyau::entreprise::FicheEntreprise::default().valider(amazone_plateforme::aujourdhui());
    let champs: Vec<String> = corps["erreurs"]
        .as_array()
        .expect("liste d'erreurs")
        .iter()
        .map(|e| e["champ"].as_str().expect("champ").to_owned())
        .collect();
    let attendus: Vec<String> = attendu.erreurs.iter().map(|p| p.champ.clone()).collect();
    assert_eq!(champs, attendus);
    assert!(corps["avertissements"].as_array().is_some());
}

#[sqlx::test]
async fn ifu_deja_inscrirefus(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton_a = compte_verifie(&app, &env, "a@exemple.bj").await;
    let jeton_b = compte_verifie(&app, &env, "b@exemple.bj").await;
    entreprise_inscrite(&app, &jeton_a, "3201912345678").await;

    // CGU acceptées pour le second compte, puis IFU identique.
    let (statut, _) = envoyer(
        &app,
        req_json("POST", "/v1/cgu/acceptations", &json!({ "version": 1 }), Some(&jeton_b)),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED);
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/entreprises",
            &json!({ "fiche": fiche_valide("3201 9123 45678") }),
            Some(&jeton_b),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CONFLICT);
    assert_eq!(corps["code"], "IFU_DEJA_INSCRIT");
}

#[sqlx::test]
async fn mot_de_passe_faux_dix_fois_limitation(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton = compte_verifie(&app, &env, "marie@exemple.bj").await;
    drop(jeton);

    for _ in 0..10 {
        let (statut, _) = envoyer(
            &app,
            req_json(
                "POST",
                "/v1/sessions",
                &json!({ "courriel": "marie@exemple.bj", "mot_de_passe": "mauvais-mot-de-passe" }),
                None,
            ),
        )
        .await;
        assert_eq!(statut, StatusCode::UNAUTHORIZED);
    }
    // 11ᵉ tentative : même avec le bon mot de passe, le compte est verrouillé.
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/sessions",
            &json!({ "courriel": "marie@exemple.bj", "mot_de_passe": "un-mot-de-passe-solide" }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(corps["code"], "TROP_DE_TENTATIVES");
}

#[sqlx::test]
async fn transfert_au_dela_de_la_limite_refuse(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton = compte_verifie(&app, &env, "marie@exemple.bj").await;
    entreprise_inscrite(&app, &jeton, "3201912345678").await;

    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/licences",
            &json!({ "ifu": "3201912345678", "empreinte_poste": "poste-a" }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED);

    // Deux transferts autorisés par an (paramètre par défaut) : A→B puis B→C.
    for (ancienne, nouvelle) in [("poste-a", "poste-b"), ("poste-b", "poste-c")] {
        let (statut, _) = envoyer(
            &app,
            req_json(
                "POST",
                "/v1/licences/transferer",
                &json!({
                    "ifu": "3201912345678",
                    "ancienne_empreinte": ancienne,
                    "nouvelle_empreinte": nouvelle
                }),
                Some(&jeton),
            ),
        )
        .await;
        assert_eq!(statut, StatusCode::OK, "transfert {ancienne} → {nouvelle}");
    }

    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/licences/transferer",
            &json!({
                "ifu": "3201912345678",
                "ancienne_empreinte": "poste-c",
                "nouvelle_empreinte": "poste-d"
            }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::FORBIDDEN);
    assert_eq!(corps["code"], "LIMITE_TRANSFERTS");
}

#[sqlx::test]
async fn renouvellement_refuse_si_entreprise_suspendue(pool: PgPool) {
    let (app, env) = app(pool.clone()).await;
    let jeton = compte_verifie(&app, &env, "marie@exemple.bj").await;
    let entreprise_id = entreprise_inscrite(&app, &jeton, "3201912345678").await;
    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/licences",
            &json!({ "ifu": "3201912345678", "empreinte_poste": "poste-a" }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED);

    // Suspension (action d'administration, lot 7) : renouvellement et émission refusés.
    sqlx::query("UPDATE entreprises SET statut = 'SUSPENDUE', motif = 'motif de test' WHERE id = $1")
        .bind(entreprise_id)
        .execute(&pool)
        .await
        .expect("suspension");
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/licences/renouveler",
            &json!({ "ifu": "3201912345678", "empreinte_poste": "poste-a" }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::FORBIDDEN);
    assert_eq!(corps["code"], "ENTREPRISE_SUSPENDUE");
    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/licences",
            &json!({ "ifu": "3201912345678", "empreinte_poste": "poste-b" }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::FORBIDDEN, "émission refusée aussi");

    // Réactivation : le renouvellement redevient possible.
    sqlx::query("UPDATE entreprises SET statut = 'VERIFIEE' WHERE id = $1")
        .bind(entreprise_id)
        .execute(&pool)
        .await
        .expect("réactivation");
    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/licences/renouveler",
            &json!({ "ifu": "3201912345678", "empreinte_poste": "poste-a" }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::OK, "renouvellement après réactivation");
}

#[sqlx::test]
async fn verification_prealable_bloque_la_licence(pool: PgPool) {
    let (app, env) = app(pool.clone()).await;
    let jeton = compte_verifie(&app, &env, "marie@exemple.bj").await;
    let entreprise_id = entreprise_inscrite(&app, &jeton, "3201912345678").await;

    // Le propriétaire active la « vérification préalable » (console, lot 7).
    sqlx::query("UPDATE parametres SET valeur = 'true' WHERE cle = 'verification_prealable'")
        .execute(&pool)
        .await
        .expect("réglage");
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/licences",
            &json!({ "ifu": "3201912345678", "empreinte_poste": "poste-a" }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::FORBIDDEN);
    assert_eq!(corps["code"], "VERIFICATION_EN_ATTENTE");

    // Après vérification par l'administrateur, la licence est délivrée.
    sqlx::query("UPDATE entreprises SET statut = 'VERIFIEE' WHERE id = $1")
        .bind(entreprise_id)
        .execute(&pool)
        .await
        .expect("vérification");
    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/licences",
            &json!({ "ifu": "3201912345678", "empreinte_poste": "poste-a" }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED);
}

#[sqlx::test]
async fn mot_de_passe_oublie_puis_reinitialisation(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton = compte_verifie(&app, &env, "marie@exemple.bj").await;
    drop(jeton);

    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/mots-de-passe/oublie",
            &json!({ "courriel": "marie@exemple.bj" }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::ACCEPTED);
    let code = env.dernier_code("marie@exemple.bj").expect("code envoyé");

    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/mots-de-passe/reinitialiser",
            &json!({
                "courriel": "marie@exemple.bj",
                "code": code,
                "nouveau_mot_de_passe": "un-nouveau-mot-de-passe"
            }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::OK);

    // L'ancien mot de passe ne fonctionne plus ; le nouveau oui.
    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/sessions",
            &json!({ "courriel": "marie@exemple.bj", "mot_de_passe": "un-mot-de-passe-solide" }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::UNAUTHORIZED);
    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/sessions",
            &json!({ "courriel": "marie@exemple.bj", "mot_de_passe": "un-nouveau-mot-de-passe" }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::OK);
}

#[sqlx::test]
async fn acces_entreprise_etrangere_refuse(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton_a = compte_verifie(&app, &env, "a@exemple.bj").await;
    let entreprise_id = entreprise_inscrite(&app, &jeton_a, "3201912345678").await;
    let jeton_b = compte_verifie(&app, &env, "b@exemple.bj").await;

    let (statut, corps) = envoyer(
        &app,
        req_json(
            "GET",
            &format!("/v1/entreprises/{entreprise_id}/statut"),
            &json!({}),
            Some(&jeton_b),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::FORBIDDEN);
    assert_eq!(corps["code"], "ACCES_REFUSE");

    let (statut, _) = envoyer(
        &app,
        req_json(
            "PUT",
            &format!("/v1/entreprises/{entreprise_id}"),
            &json!({ "fiche": fiche_valide("3201912345678") }),
            Some(&jeton_b),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn cgu_obligatoires_avant_inscription(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton = compte_verifie(&app, &env, "marie@exemple.bj").await;
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/entreprises",
            &json!({ "fiche": fiche_valide("3201912345678") }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::FORBIDDEN);
    assert_eq!(corps["code"], "CGU_NON_ACCEPTEES");
}

#[sqlx::test]
async fn code_verification_invalide_puis_epuise(pool: PgPool) {
    let (app, env) = app(pool).await;
    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/comptes",
            &json!({
                "courriel": "marie@exemple.bj",
                "mot_de_passe": "un-mot-de-passe-solide",
                "nom": "Marie"
            }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED);
    let bon_code = env.dernier_code("marie@exemple.bj").expect("code envoyé");

    for _ in 0..5 {
        let (statut, _) = envoyer(
            &app,
            req_json(
                "POST",
                "/v1/comptes/verifications",
                &json!({ "courriel": "marie@exemple.bj", "code": "000000" }),
                None,
            ),
        )
        .await;
        assert_eq!(statut, StatusCode::BAD_REQUEST);
    }
    // Même le bon code est refusé après cinq tentatives : il faut en demander un nouveau.
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            "/v1/comptes/verifications",
            &json!({ "courriel": "marie@exemple.bj", "code": bon_code }),
            None,
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::BAD_REQUEST);
    assert!(corps["message"]
        .as_str()
        .expect("message")
        .contains("nouveau"));
}

#[sqlx::test]
async fn justificatifs_controles(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton = compte_verifie(&app, &env, "marie@exemple.bj").await;
    let entreprise_id = entreprise_inscrite(&app, &jeton, "3201912345678").await;
    let uri = format!("/v1/entreprises/{entreprise_id}/justificatifs");

    // Fichier correctement accepté.
    let contenu = base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.4 test");
    let (statut, _) = envoyer(
        &app,
        req_json(
            "POST",
            &uri,
            &json!({
                "categorie": "ATTESTATION_IFU",
                "nom_fichier": "attestation.pdf",
                "mime": "application/pdf",
                "contenu_base64": contenu
            }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::CREATED);

    // Format non accepté.
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            &uri,
            &json!({
                "categorie": "ATTESTATION_IFU",
                "nom_fichier": "virus.exe",
                "mime": "application/x-msdownload",
                "contenu_base64": contenu
            }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(corps["code"], "TYPE_NON_SUPPORTE");

    // Fichier de plus de 5 Mo refusé.
    let gros = base64::engine::general_purpose::STANDARD.encode(vec![0u8; 6 * 1024 * 1024]);
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "POST",
            &uri,
            &json!({
                "categorie": "EXTRAIT_RCCM",
                "nom_fichier": "gros.pdf",
                "mime": "application/pdf",
                "contenu_base64": gros
            }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(corps["code"], "FICHIER_TROP_GRAND");
}

#[sqlx::test]
async fn modification_fiche_et_demande_changement_ifu(pool: PgPool) {
    let (app, env) = app(pool).await;
    let jeton = compte_verifie(&app, &env, "marie@exemple.bj").await;
    let entreprise_id = entreprise_inscrite(&app, &jeton, "3201912345678").await;

    // Changement d'IFU : demande soumise à l'administrateur, autres champs modifiés.
    let mut fiche = fiche_valide("3012345678901");
    fiche["commune"] = json!("Abomey-Calavi");
    let (statut, corps) = envoyer(
        &app,
        req_json(
            "PUT",
            &format!("/v1/entreprises/{entreprise_id}"),
            &json!({ "fiche": fiche }),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps["demande_changement_ifu"], true);

    let (statut, corps) = envoyer(
        &app,
        req_json(
            "GET",
            &format!("/v1/entreprises/{entreprise_id}/statut"),
            &json!({}),
            Some(&jeton),
        ),
    )
    .await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps["ifu"], "3201912345678", "l'IFU actuel ne change pas tout seul");
    assert_eq!(corps["demandes_ifu_en_attente"], json!(["3012345678901"]));
}
