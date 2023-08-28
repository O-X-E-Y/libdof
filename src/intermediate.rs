use serde::{Deserialize, Serialize};
use serde_with::{serde_as, serde_conv, DisplayFromStr};
use thiserror::Error;

use std::collections::BTreeMap;

use crate::definitions::{self, *};

#[derive(Debug, Error)]
pub enum DofIntermediateError {
    #[error("couldn't parse fingering")]
    DefinitionError(#[from] definitions::DefinitionError),
}

macro_rules! impl_keyboard {
    ($type:ty, $ret:ty, $alias:ident) => {
        impl $type {
            pub fn rows(&self) -> impl Iterator<Item = &Vec<$ret>> {
                self.0.iter()
            }
            pub fn keys(&self) -> impl Iterator<Item = &$ret> {
                self.rows().flatten()
            }
            pub fn shape(&self) -> Vec<usize> {
                self.rows().map(|r| r.len()).collect()
            }
        }

        serde_conv!(
            $alias,
            Vec<$ret>,
            |row: &Vec<$ret>| {
                if row.len() == 0 {
                    String::new()
                } else {
                    row.into_iter()
                        .take(row.len() - 1)
                        .map(|e| format!("{e} "))
                        .chain([row.last().unwrap().to_string()])
                        .collect::<String>()
                }
            },
            |line: String| {
                line.split_whitespace()
                    .map(|s| s.parse::<$ret>())
                    .collect::<Result<Vec<_>, crate::definitions::DefinitionError>>()
            }
        );
    };
}

impl_keyboard!(Fingering, Finger, FingeringStrAsRow);

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fingering(#[serde_as(as = "Vec<FingeringStrAsRow>")] Vec<Vec<Finger>>);

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParsedFingering {
    Explicit(Fingering),
    Implicit(#[serde_as(as = "DisplayFromStr")] NamedFingering),
}

impl_keyboard!(Layer, Key, LayerStrAsRow);

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer(#[serde_as(as = "Vec<LayerStrAsRow>")] Vec<Vec<Key>>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Anchor(u8, u8);

/// Main struct to use for parsing
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DofIntermediate {
    name: String,
    authors: Option<Vec<String>>,
    #[serde_as(as = "DisplayFromStr")]
    board: KeyboardType,
    year: Option<u32>,
    notes: Option<String>,
    layers: BTreeMap<String, Layer>,
    anchor: Option<Anchor>,
    // alt_fingerings: Option<Vec<String>>,
    // combos: Option<HashMap<String, String>>,
    fingerings: ParsedFingering,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn fingerings() {
        use Finger::*;
        use Key::*;
        use SpecialKey::*;

        let minimal_json = json!({
            "name": "Qwerty",
            "board": "ansi",
            "layers": {},
            "fingerings": "angle"
        });

        let maximal_json = json!({
            "name": "Qwerty",
            "authors": ["Christopher Latham Sholes"],
            "board": "ansi",
            "year": 1878,
            "notes": "the OG. Without Qwerty, none of this would be necessary.",
            "anchor": [1, 2],
            "layers": {
                "main": [
                    "` 1 2 3 4 5  6 7 8 9 0 - = bsp",
                    "tb q w e r t  y u i o p [ ] \\",
                    "cps a s d f g  h j k l ; ' ret",
                    "shft z x c v b  n m , . / shft",
                    "ct fn mt alt spc altgr mt ct"
                ],
                "shift": [
                    "\\~ ! @ # $ %  ^ & \\* ( ) _ + bsp",
                    "tab  Q W E R T  Y U   I O P { } |",
                    "caps  A S D F G  H J   K L : \" ent",
                    "*      Z X C V B  N M   < > ? shft",
                    "ct fn mt alt spc altgr mt ct"
                ]
            },
            "fingerings": [
                "0  0  1  2  3  3   6  6  7  8  9  9  9  9  9",
                "LP LP LR LM LI LI  RI RI RM RR RP RP RP RP",
                "LP LP LR LM LI LI  RI RI RM RR RP RP RP",
                "LP LR LM LI LI LI  RI RI RM RR RP RP",
                "LP  LP  LT  LT    LT    RT  RT  RP"
            ]
        });

        let minimal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: None,
            board: KeyboardType::Ansi,
            year: None,
            notes: None,
            anchor: None,
            layers: BTreeMap::new(),
            fingerings: { ParsedFingering::Implicit(NamedFingering::Angle) },
        };

        let maximal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: Some(vec!["Christopher Latham Sholes".into()]),
            board: KeyboardType::Ansi,
            year: Some(1878),
            notes: Some("the OG. Without Qwerty, none of this would be necessary.".into()),
            anchor: Some(Anchor(1, 2)),
            layers: BTreeMap::from_iter([
                (
                    "main".into(),
                    Layer(vec![
                        vec![
                            Char('`'),
                            Char('1'),
                            Char('2'),
                            Char('3'),
                            Char('4'),
                            Char('5'),
                            Char('6'),
                            Char('7'),
                            Char('8'),
                            Char('9'),
                            Char('0'),
                            Char('-'),
                            Char('='),
                            Special(Backspace),
                        ],
                        vec![
                            Special(Tab),
                            Char('q'),
                            Char('w'),
                            Char('e'),
                            Char('r'),
                            Char('t'),
                            Char('y'),
                            Char('u'),
                            Char('i'),
                            Char('o'),
                            Char('p'),
                            Char('['),
                            Char(']'),
                            Char('\\'),
                        ],
                        vec![
                            Special(Caps),
                            Char('a'),
                            Char('s'),
                            Char('d'),
                            Char('f'),
                            Char('g'),
                            Char('h'),
                            Char('j'),
                            Char('k'),
                            Char('l'),
                            Char(';'),
                            Char('\''),
                            Special(Enter),
                        ],
                        vec![
                            Special(Shift),
                            Char('z'),
                            Char('x'),
                            Char('c'),
                            Char('v'),
                            Char('b'),
                            Char('n'),
                            Char('m'),
                            Char(','),
                            Char('.'),
                            Char('/'),
                            Special(Shift),
                        ],
                        vec![
                            Special(Ctrl),
                            Special(Fn),
                            Special(Meta),
                            Special(Alt),
                            Special(Space),
                            Layer {
                                name: "altgr".into(),
                            },
                            Special(Meta),
                            Special(Ctrl),
                        ],
                    ]),
                ),
                (
                    "shift".into(),
                    Layer(vec![
                        vec![
                            Char('~'),
                            Char('!'),
                            Char('@'),
                            Char('#'),
                            Char('$'),
                            Char('%'),
                            Char('^'),
                            Char('&'),
                            Char('*'),
                            Char('('),
                            Char(')'),
                            Char('_'),
                            Char('+'),
                            Special(Backspace),
                        ],
                        vec![
                            Special(Tab),
                            Char('Q'),
                            Char('W'),
                            Char('E'),
                            Char('R'),
                            Char('T'),
                            Char('Y'),
                            Char('U'),
                            Char('I'),
                            Char('O'),
                            Char('P'),
                            Char('{'),
                            Char('}'),
                            Char('|'),
                        ],
                        vec![
                            Special(Caps),
                            Char('A'),
                            Char('S'),
                            Char('D'),
                            Char('F'),
                            Char('G'),
                            Char('H'),
                            Char('J'),
                            Char('K'),
                            Char('L'),
                            Char(':'),
                            Char('\"'),
                            Special(Enter),
                        ],
                        vec![
                            Transparent,
                            Char('Z'),
                            Char('X'),
                            Char('C'),
                            Char('V'),
                            Char('B'),
                            Char('N'),
                            Char('M'),
                            Char('<'),
                            Char('>'),
                            Char('?'),
                            Special(Shift),
                        ],
                        vec![
                            Special(Ctrl),
                            Special(Fn),
                            Special(Meta),
                            Special(Alt),
                            Special(Space),
                            Layer {
                                name: "altgr".into(),
                            },
                            Special(Meta),
                            Special(Ctrl),
                        ],
                    ]),
                ),
            ]),
            fingerings: {
                ParsedFingering::Explicit(Fingering(vec![
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP, RP],
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP],
                    vec![LP, LR, LM, LI, LI, LI, RI, RI, RM, RR, RP, RP],
                    vec![LP, LP, LT, LT, LT, RT, RT, RP],
                ]))
            },
        };

        let dof_minimal = serde_json::from_value::<DofIntermediate>(minimal_json)
            .expect("couldn't parse implicit json");
        let dof_maximal = serde_json::from_value::<DofIntermediate>(maximal_json)
            .expect("couldn't parse explicit json");

        assert_eq!(dof_minimal, minimal_test);
        assert_eq!(dof_maximal, maximal_test);
    }
}