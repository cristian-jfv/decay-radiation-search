use log::{debug, error};
use once_cell::sync::Lazy;
use regex::Regex;

//const QUERY_PATTERN: &str = r"^(?P<modifier>[a-zA-Z]*)?(\s*)?(?P<energy>([0-9]*[.])?[0-9]+){1}(\s*)?((?P<unit>[a-zA-Z]*)\s?){1}(\s*)?((?P<uncertainty>([0-9]*[.])?[0-9]+)%)?$";
//const QUERY_PATTERN: &str = r"^(?P<modifier>[a-zA-Z]*)?(\s*)?(?P<energy>([0-9]*[.])?[0-9]+)(\s*)?((?P<unit>[a-zA-Z]*)\s?)\s+((?P<uncertainty>([0-9]*[.])?[0-9]+)%)?\s*";
const QUERY_PATTERN: &str = r"^(?P<modifier>[a-zA-Z]*)?[[:blank:]]?(?P<energy>([0-9]*[.])?[0-9]+)[[:blank:]]?(?P<unit>[a-zA-Z]+)([[:blank:]]+(?P<uncertainty>([0-9]*[.])?[0-9]+)%)?";
pub struct Energy {
    lteb: f64,
    uteb: f64,
}

enum Modifier {
    Definitely,
    Maybe,
}

enum UnitPrefix {
    Mega,
    Kilo,
}

pub struct InputError;
fn calculate_energy_bounds(energy: &str, uncertainty: &str, unit: &str) -> (f64, f64) {
    return (1.234, 1.234);
}

fn parse_line(line: &str) -> Result<Energy, InputError> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(QUERY_PATTERN).unwrap());
    match RE.captures(line) {
        None => {
            debug!("NO MATCH: {line}");
            return Err(InputError);
        }
        Some(cap) => {
            let uncert = match cap.name("uncertainty") {
                Some(m) => m.as_str(),
                None => "",
            };
            debug!(
                "{line}: {}",
                format!(
                    "modifier: <{}>; energy: <{}>; unit: <{}>, uncertainty: <{}>\n",
                    &cap["modifier"], &cap["energy"], &cap["unit"], uncert,
                )
            );
            let (lteb, uteb) = calculate_energy_bounds(&cap["energy"], uncert, &cap["unit"]);
            return Ok(Energy { lteb, uteb });
        }
    }
}

pub fn parse_user_query(input: String) -> Result<Vec<Energy>, InputError> {
    let lines = input.split('\n');
    let mut energies = Vec::new();

    for line in lines {
        match parse_line(line) {
            Ok(energy) => energies.push(energy),
            Err(e) => {
                error!("Error while parsing line: {line}");
                return Err(e);
            }
        }
    }
    return Ok(energies);
}
