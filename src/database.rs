use crate::{app::RadiationType, query_parser::Energy};
use core::fmt;
use log::debug;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

const DATABASE_BYTES: &[u8] = include_bytes!("../assets/database.bin");

//const DATABASE: Vec<Transition> = rmp_serde::from_slice(DATABASE_BYTES).unwrap();
static DATABASE: Lazy<Vec<Transition>> =
    Lazy::new(|| rmp_serde::from_slice(DATABASE_BYTES).unwrap());

#[derive(Debug, Clone, Deserialize)]
pub struct Transition {
    pub parent: String,
    pub daughter: String,
    pub decay_type: String,
    pub radiation_type: String,
    pub transition_energy: String,
    pub uncertainty: String,
    pub intensity: f64,
    pub lteb: f64,
    pub uteb: f64,
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " {:>7.7} ({})", self.transition_energy, self.uncertainty)
    }
}

pub struct TransitionResult {
    pub t: Transition,
    pub found: bool,
}

fn energy_in_transition_range(e: &Energy, t: &Transition) -> bool {
    if (t.lteb <= e.lteb && e.lteb <= t.uteb) || (e.lteb <= t.lteb && t.lteb <= e.uteb) {
        return true;
    }
    return false;
}

fn filter_by_energy(e: &Energy, radiation_type: &RadiationType) -> HashSet<String> {
    let parents_vec = DATABASE
        .iter()
        .filter(|t| energy_in_transition_range(e, &t) && *radiation_type == t.radiation_type)
        .map(|t| t.decay_type.clone());
    HashSet::from_iter(parents_vec)
}

fn filter_by_decay_type(p: &String, radiation_type: &RadiationType) -> Vec<Transition> {
    let ts = DATABASE
        .iter()
        .filter(|t| t.decay_type == *p && *radiation_type == t.radiation_type)
        .cloned()
        .collect();
    return ts;
}

fn mark_found_transitions(es: &Vec<Energy>, ts: Vec<Transition>) -> Vec<TransitionResult> {
    let mut ans: Vec<TransitionResult> = vec![];
    for t in ts {
        let mut found = false;
        for e in es {
            if energy_in_transition_range(&e, &t) {
                found = true;
                break;
            }
        }
        ans.push(TransitionResult { t, found });
    }
    ans.sort_by(|a, b| {
        a.t.transition_energy
            .parse::<f64>()
            .unwrap()
            .partial_cmp(&b.t.transition_energy.parse::<f64>().unwrap())
            .expect("Error while sorting transition energies")
    });
    return ans;
}

pub fn query_database(
    energies: &Vec<Energy>,
    radiation_type: &RadiationType,
) -> Option<HashMap<String, Vec<TransitionResult>>> {
    let mut decays: HashSet<String> = filter_by_energy(&energies[0], &radiation_type);

    for e in energies {
        let current_decays = filter_by_energy(&e, &radiation_type);
        decays.retain(|x| current_decays.contains(x));
    }

    debug!("search finished");
    debug!("{}: {:?}", decays.len(), decays);
    if decays.is_empty() {
        return None;
    }

    let mut results: HashMap<String, Vec<TransitionResult>> = HashMap::new();
    for p in decays {
        results.insert(
            p.clone(),
            mark_found_transitions(&energies, filter_by_decay_type(&p, &radiation_type)),
        );
    }

    return Some(results);
}
