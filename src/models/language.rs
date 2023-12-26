use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, sqlx::Type, Deserialize, Serialize, PartialEq, Eq)]
#[sqlx(type_name = "Language", rename_all = "lowercase")]
pub enum Language {
    // Bg,
    // Cs,
    // Da,
    De,
    // El,
    En,
    // Es,
    // Et,
    // Fi,
    Fr,
    // Hu,
    // It,
    // Ja,
    // Lt,
    // Lv,
    // Mt,
    // Nl,
    // Pl,
    // Pt,
    // Ro,
    Ru,
    // Sk,
    // Sl,
    // Sv,
    Zh,
}
