pub mod definitions;

use serde::{Serialize, Deserialize};
use serde_with::{serde_conv, serde_as};
use thiserror::Error;

use std::collections::BTreeMap;

use crate::definitions::*;

#[derive(Debug, Error)]
pub enum DofError {
    #[error("couldn't parse fingering")]
    DefinitionError(#[from] definitions::DefinitionError)
}

serde_conv!(
    FingeringStrAsRow,
    Vec<Finger>,
    |row: &Vec<Finger>| {
        if row.len() == 0 {
            String::new()
        } else {
            row.into_iter()
                .take(row.len() - 1)
                .map(|e| format!("{e:?} "))
                .chain([row.last().unwrap().to_string()])
                .collect::<String>()
        }
    },
    |line: String| {
        line.split_whitespace()
            .map(|s| s.parse::<Finger>())
            .collect::<Result<Vec<_>, DefinitionError>>()
    }
);

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FingeringRows(
    #[serde_as(as = "Vec<FingeringStrAsRow>")]
    Vec<Vec<Finger>>
);

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Fingering {
    Explicit(FingeringRows),
    Implicit(String)
}

serde_conv!(
    LayerStrAsRow,
    Vec<Key>,
    |row: &Vec<Key>| {
        if row.len() == 0 {
            String::new()
        } else {
            row.into_iter()
                .take(row.len() - 1)
                .map(|e| format!("{} ", e.to_string()))
                .chain([row.last().unwrap().to_string()])
                .collect::<String>()
        }
    },
    |line: String| {
        line.split_whitespace()
            .map(|s| s.parse::<Key>())
            .collect::<Result<Vec<_>, DefinitionError>>()
    }
);

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer(
    #[serde_as(as = "Vec<LayerStrAsRow>")]
    Vec<Vec<Key>>
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Anchor(u8, u8);

/// Main struct to use for parsing
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DofIntermediate {
    name: String,
    authors: Option<Vec<String>>,
    board: String,
    year: Option<u32>,
    notes: Option<String>,
    layers: BTreeMap<String, Layer>,
    anchor: Option<Anchor>,
    // alt_fingerings: Option<Vec<String>>,
    // combos: Option<HashMap<String, String>>,
    fingerings: Fingering,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    
    use super::*;

    #[test]
    fn fingerings() {
        use Finger::*;
        use Fingering::*;
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
            "year": 1889,
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
            board: "ansi".into(),
            year: None,
            notes: None,
            anchor: None,
            layers: BTreeMap::new(),
            fingerings: {
                Implicit("angle".into())
            }
        };

        let maximal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: Some(vec!["Christopher Latham Sholes".into()]),
            board: "ansi".into(),
            year: Some(1889),
            notes: Some("the OG. Without Qwerty, none of this would be necessary.".into()),
            anchor: Some(Anchor(1, 2)),
            layers: BTreeMap::from_iter([
                ("main".into(), Layer(vec![
                    vec![
                        Char('`'), Char('1'), Char('2'), Char('3'), Char('4'), Char('5'), Char('6'),
                        Char('7'), Char('8'), Char('9'), Char('0'), Char('-'), Char('='), Special(Backspace)
                    ],
                    vec![
                        Special(Tab), Char('q'), Char('w'), Char('e'), Char('r'), Char('t'),
                        Char('y'), Char('u'), Char('i'), Char('o'), Char('p'), Char('['), Char(']'), Char('\\')
                    ],
                    vec![
                        Special(Caps), Char('a'), Char('s'), Char('d'), Char('f'), Char('g'),
                        Char('h'), Char('j'), Char('k'), Char('l'), Char(';'), Char('\''), Special(Enter)
                    ],
                    vec![
                        Special(Shift), Char('z'), Char('x'), Char('c'), Char('v'), Char('b'),
                        Char('n'),  Char('m'), Char(','), Char('.'),  Char('/'), Special(Shift)
                    ],
                    vec![
                        Special(Ctrl), Special(Fn), Special(Meta), Special(Alt), Special(Space),
                        Layer { name: "altgr".into() }, Special(Meta), Special(Ctrl)
                    ],
                ])),
                ("shift".into(), Layer(vec![
                    vec![
                        Char('~'), Char('!'), Char('@'), Char('#'), Char('$'), Char('%'), Char('^'),
                        Char('&'), Char('*'), Char('('), Char(')'), Char('_'), Char('+'), Special(Backspace)
                    ],
                    vec![
                        Special(Tab), Char('Q'), Char('W'), Char('E'), Char('R'), Char('T'),
                        Char('Y'), Char('U'), Char('I'),  Char('O'),  Char('P'), Char('{'), Char('}'), Char('|')
                    ],
                    vec![
                        Special(Caps), Char('A'), Char('S'), Char('D'), Char('F'), Char('G'),
                        Char('H'), Char('J'), Char('K'), Char('L'),  Char(':'), Char('\"'), Special(Enter)
                    ],
                    vec![
                        Transparent, Char('Z'), Char('X'), Char('C'), Char('V'), Char('B'),
                        Char('N'), Char('M'), Char('<'),  Char('>'),  Char('?'), Special(Shift)
                    ],
                    vec![
                        Special(Ctrl), Special(Fn), Special(Meta), Special(Alt), Special(Space),
                        Layer { name: "altgr".into() }, Special(Meta), Special(Ctrl)
                    ],
                ]))
            ]),
            fingerings: {
                Explicit(FingeringRows(
                    vec![
                        vec![LP, LP, LR, LM, LI, LI,  RI, RI, RM, RR, RP, RP, RP, RP, RP],
                        vec![LP, LP, LR, LM, LI, LI,  RI, RI, RM, RR, RP, RP, RP, RP],
                        vec![LP, LP, LR, LM, LI, LI,  RI, RI, RM, RR, RP, RP, RP],
                        vec![LP, LR, LM, LI, LI, LI,  RI, RI, RM, RR, RP, RP],
                        vec![LP,  LP,  LT,  LT,     LT,     RT,  RT,  RP]
                    ]
                ))
            }
        };

        let dof_minimal = serde_json::from_value::<DofIntermediate>(minimal_json).expect("couldn't parse implicit json");
        let dof_maximal = serde_json::from_value::<DofIntermediate>(maximal_json).expect("couldn't parse explicit json");
        
        assert_eq!(dof_minimal, minimal_test);
        assert_eq!(dof_maximal, maximal_test);
    }
}