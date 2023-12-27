use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, sqlx::Type, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[sqlx(type_name = "language", rename_all = "lowercase")]
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
