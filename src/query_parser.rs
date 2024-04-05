use crate::app::{PrintMode, RadiationType};
use crate::database::TransitionResult;
use log::{debug, error};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

use crate::database::query_database;

//const QUERY_PATTERN: &str = r"^(?P<modifier>[a-zA-Z]*)?(\s*)?(?P<energy>([0-9]*[.])?[0-9]+){1}(\s*)?((?P<unit>[a-zA-Z]*)\s?){1}(\s*)?((?P<uncertainty>([0-9]*[.])?[0-9]+)%)?$";
//const QUERY_PATTERN: &str = r"^(?P<modifier>[a-zA-Z]*)?(\s*)?(?P<energy>([0-9]*[.])?[0-9]+)(\s*)?((?P<unit>[a-zA-Z]*)\s?)\s+((?P<uncertainty>([0-9]*[.])?[0-9]+)%)?\s*";
const QUERY_PATTERN: &str = r"^(?P<modifier>[a-zA-Z]*)?[[:blank:]]?(?P<energy>([0-9]*[.])?[0-9]+)[[:blank:]]?(?P<unit>[a-zA-Z]+)([[:blank:]]+(?P<uncertainty>([0-9]*[.])?[0-9]+)%)?";

pub enum Modifier {
    Definitely,
    Maybe,
}

impl std::fmt::Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Modifier::Definitely => "definitely",
                Modifier::Maybe => "maybe",
            }
        )
    }
}

pub struct Energy {
    pub lteb: f64,
    pub uteb: f64,
    pub modifier: Modifier,
}

impl std::fmt::Display for Energy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "lower bound={}; upper bound={};modifier: {}",
            self.lteb, self.uteb, self.modifier
        )
    }
}

pub struct InputError;

fn calculate_energy_bounds(energy: &str, uncertainty: &str, unit: &str) -> (f64, f64) {
    let e = energy.parse::<f64>().unwrap();
    let u: f64 = if !uncertainty.is_empty() {
        uncertainty.parse::<f64>().unwrap() / 100.0
    } else {
        0.0
    };

    let base: f64 = 10.0;
    let m = match unit {
        "MeV" => 3.0,
        "keV" => 0.0,
        "eV" => -3.0,
        _ => 0.0,
    };

    let lteb = e * base.powf(m) * (1.0 - u);
    let uteb = e * base.powf(m) * (1.0 + u);
    (lteb, uteb)
}

fn parse_line(line: &str) -> Result<Energy, InputError> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(QUERY_PATTERN).unwrap());
    match RE.captures(line) {
        None => {
            debug!("NO MATCH: {line}");
            Err(InputError)
        }
        Some(cap) => {
            let uncert = match cap.name("uncertainty") {
                Some(m) => m.as_str(),
                None => "",
            };
            //debug!(
            //    "\n<{line}>\n{}",
            //    format!(
            //        "modifier: <{}>; energy: <{}>; unit: <{}>, uncertainty: <{}>\n",
            //        &cap["modifier"], &cap["energy"], &cap["unit"], uncert,
            //    )
            //);
            let (lteb, uteb) = calculate_energy_bounds(&cap["energy"], uncert, &cap["unit"]);
            let modifier = match cap["modifier"].to_lowercase().as_ref() {
                "definitely" => Modifier::Definitely,
                "" => Modifier::Definitely,
                "maybe" => Modifier::Maybe,
                _ => Modifier::Definitely,
            };

            let e = Energy {
                lteb,
                uteb,
                modifier,
            };

            //debug!("{e}");

            Ok(e)
        }
    }
}

fn parse_user_query(input: String) -> Result<Vec<Energy>, InputError> {
    let lines = input.split('\n');
    let mut energies = Vec::new();

    for line in lines {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        match parse_line(line) {
            Ok(energy) => energies.push(energy),
            Err(e) => {
                error!("Error while parsing line: {line}");
                return Err(e);
            }
        }
    }
    Ok(energies)
}

fn print_results(
    energies: HashMap<String, Vec<TransitionResult>>,
    print_mode: &PrintMode,
) -> String {
    let mut ans = String::new();
    // Summarize findings
    let noun = match energies.len() {
        1 => "decay",
        _ => "decays",
    };
    ans += format!(
        "{} {} found (energies are given in keV, * denotes a match):\n",
        energies.len(),
        noun
    )
    .as_str();
    for (d, list) in energies.into_iter() {
        // print header for the trasition
        ans += format!("\n{d}\n").as_str();
        let mut i = 1;
        // print each record inside the transition
        for r in list {
            if *print_mode == PrintMode::OnlyMatches && !r.found {
                // Omit not matching records for this printing mode
                continue;
            }
            ans += format!(
                "{}{:>5}{}\n",
                match r.found {
                    true => "*",
                    false => " ",
                },
                i,
                r.t
            )
            .as_str();
            i += 1;
        }
    }
    ans
}

pub fn search_energies(
    input: String,
    radiation_type: &RadiationType,
    print_mode: &PrintMode,
) -> String {
    let energies = match parse_user_query(input) {
        Ok(l) => l,
        Err(_) => {
            return "Verify the search query".to_string();
        }
    };

    match query_database(&energies, radiation_type) {
        Some(map) => print_results(map, print_mode),
        None => "No results found".to_string(),
    }
}
