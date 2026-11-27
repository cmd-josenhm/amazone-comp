#!/usr/bin/env node
// Vérifie qu'une branche ne modifie que les fichiers du périmètre de son agent (règle R10).
//
// Usage local :   node scripts/verifier-perimetre.mjs a4            (compare à origin/main)
// Usage CI :      node scripts/verifier-perimetre.mjs --branche a4/feat-factures --base origin/main
//
// L'agent est déduit du préfixe de branche « aN/ ». Les branches « chef/ » (chef de projet),
// « dependabot/ » et « ext/ » (contributeurs externes, relus à la main) ne sont pas restreintes.
// Les branches « arena/<id> » (imposées par la plateforme d'agents Arena) : l'agent est lu dans la
// description de la PR, ligne « Agent : **AN** » du modèle (variable d'environnement CORPS_PR en CI).

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const args = process.argv.slice(2);
const option = (nom) => {
  const i = args.indexOf(nom);
  return i >= 0 ? args[i + 1] : undefined;
};

const branche = option("--branche");
const base = option("--base") ?? "origin/main";
let agent = args.find((a) => /^a\d+$/i.test(a))?.toLowerCase();

if (!agent && branche) {
  const prefixe = branche.split("/")[0].toLowerCase();
  if (["chef", "dependabot", "ext"].includes(prefixe)) {
    console.log(`Branche « ${branche} » : pas de restriction de périmètre automatique.`);
    process.exit(0);
  }
  if (/^a\d+$/.test(prefixe)) agent = prefixe;
  if (prefixe === "arena") {
    const trouve = /Agent\s*:\s*\**\s*A(\d+)\b/i.exec(process.env.CORPS_PR ?? "");
    if (trouve) agent = `a${trouve[1]}`;
  }
}

if (!agent) {
  console.error(
    "ERREUR : impossible de déterminer l'agent. Nommer la branche « aN/<type>-<sujet> » (ex. a4/feat-avoirs) ou, " +
      "pour une branche « arena/… », remplir la ligne « Agent : **AN** » de la description de PR.",
  );
  process.exit(2);
}

const config = JSON.parse(readFileSync(new URL("../.github/perimetres.json", import.meta.url), "utf8"));
const regles = config.agents[agent];
if (!regles) {
  console.error(`ERREUR : agent inconnu « ${agent} ».`);
  process.exit(2);
}

const sortie = execFileSync("git", ["diff", "--name-status", `${base}...HEAD`], { encoding: "utf8" });
const changements = sortie
  .split("\n")
  .filter(Boolean)
  .map((ligne) => {
    const [statut, ...chemins] = ligne.split("\t");
    return { statut: statut[0], chemin: chemins[chemins.length - 1] };
  });

const commence = (chemin, prefixes) => prefixes.some((p) => chemin === p || chemin.startsWith(p));

const fautes = [];
for (const { statut, chemin } of changements) {
  if (commence(chemin, regles.interdit)) {
    fautes.push(`${chemin} (interdit pour ${agent.toUpperCase()})`);
    continue;
  }
  if (commence(chemin, regles.autorise) || commence(chemin, config.communs)) continue;
  if (chemin.startsWith(config.adr_nouveaux_seulement) && statut === "A") continue;
  fautes.push(`${chemin} (hors périmètre de ${agent.toUpperCase()})`);
}

if (fautes.length > 0) {
  console.error(`Périmètre NON respecté (${fautes.length} fichier(s)) — règle R10 d'AGENTS.md :`);
  for (const f of fautes) console.error(`  - ${f}`);
  console.error("Ouvrez une issue pour l'agent propriétaire au lieu de modifier ces fichiers.");
  process.exit(1);
}
console.log(`Périmètre respecté pour ${agent.toUpperCase()} (${changements.length} fichier(s) modifié(s)).`);
