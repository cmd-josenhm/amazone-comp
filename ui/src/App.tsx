// Routes de l'application — Propriétaire : A3.
import { HashRouter, Route, Routes } from "react-router-dom";
import { Coque } from "./coque/Coque";
import { modules } from "./modules/registre";
import { APropos } from "./pages/a-propos/APropos";
import { Demarrage } from "./pages/Demarrage";
import { Parametres } from "./pages/Parametres";
import { DemoDesign } from "./pages/DemoDesign";

export function App() {
  return (
    <HashRouter>
      <Coque>
        <Routes>
          {modules.map((m) => (
            <Route key={m.id} path={m.chemin === "/" ? "/" : `${m.chemin}/*`} element={<m.Page />} />
          ))}
          <Route path="/parametres" element={<Parametres />} />
          <Route path="/a-propos" element={<APropos />} />
          <Route path="/demarrage" element={<Demarrage />} />
          {import.meta.env.DEV ? <Route path="/demo-design" element={<DemoDesign />} /> : null}
          <Route path="*" element={<p>Page introuvable.</p>} />
        </Routes>
      </Coque>
    </HashRouter>
  );
}
