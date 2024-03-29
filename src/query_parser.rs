use egui::TextBuffer;
use log::debug;
use regex::Regex;
use once_cell::sync::Lazy;

const QUERY_PATTERN: &str = r"^(?P<modifier>[a-zA-Z]*)?(\s*)?(?P<energy>([0-9]*[.])?[0-9]+){1}(\s*)?((?P<unit>[a-zA-Z]*)\s?){1}(\s*)?((?P<uncertainty>([0-9]*[.])?[0-9]+)%)?$";
pub struct EnergyPeak {
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

fn parse_line(line: &str) -> String {
    // static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"...").unwrap());
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(QUERY_PATTERN).unwrap());
    match RE.captures(line) {
        None => {
            return "NO MATCH\n".to_string();
        }
        Some(cap) => {
            return format!(
                "modifier: <{}>; energy: <{}>; unit: <{}>, uncertainty: <{}>\n",
                &cap["modifier"], &cap["energy"], &cap["unit"], match cap.name("uncertainty") {
                    Some(m) => m.as_str(),
                    None => ""
                },
            );
        }
    }
}

pub fn parse_user_query(input: String) -> String {
    let lines = input.split('\n');
    let mut ans = String::new();
    for line in lines {
        ans += line;
        ans += "\n";
        ans += &parse_line(line)
    }
    return ans;
}
