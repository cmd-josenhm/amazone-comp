// Coque de l'application (menu + barre supérieure) — Propriétaire : A3.
import type { ReactNode } from "react";
import { NavLink } from "react-router-dom";
import { modules } from "../modules/registre";
import "./coque.css";

const liensFixes = [
  { chemin: "/parametres", libelle: "Paramètres" },
  { chemin: "/a-propos", libelle: "À propos" },
];

export function Coque({ children }: { children: ReactNode }) {
  return (
    <div className="coque">
      <header className="coque__barre">
        <span className="coque__marque">AMAZONE COMP</span>
        <input
          className="coque__recherche"
          type="search"
          placeholder="Rechercher un client, une facture, un compte…"
          aria-label="Rechercher"
        />
      </header>
      <nav className="coque__menu" aria-label="Menu principal">
        <ul>
          {modules.map((m) => (
            <li key={m.id}>
              <NavLink to={m.chemin} end={m.chemin === "/"}>
                {m.libelle}
              </NavLink>
            </li>
          ))}
        </ul>
        <hr />
        <ul>
          {liensFixes.map((l) => (
            <li key={l.chemin}>
              <NavLink to={l.chemin}>{l.libelle}</NavLink>
            </li>
          ))}
        </ul>
      </nav>
      <main className="coque__contenu">{children}</main>
    </div>
  );
}
