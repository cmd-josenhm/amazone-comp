//! Point d'entrée Windows d'AMAZONE COMP.

// Pas de console en version publiée.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    amazone_comp_lib::lancer();
}
