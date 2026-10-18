mod cli;
mod i18n;
mod output;
mod probe;
mod sysfs;

use std::io::IsTerminal;

use clap::FromArgMatches;
use serde_json::{json, Map, Value};

use cli::{parse_item, Cli, Group, Item};
use i18n::Msg;
use sysfs::Sysfs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    i18n::init(i18n::resolve(i18n::pre_scan(&argv)));
    let matches = i18n::command().get_matches_from(argv);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    i18n::init(i18n::resolve(cli.lang));
    let group = cli.group();
    let item = match parse_item(group, cli.raw_item()) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{}", i18n::t_args(Msg::ErrorPrefix, &[("e", &e)]));
            std::process::exit(2);
        }
    };

    let sys = Sysfs::new(cli.sysfs_root.clone());
    let mut sections: Vec<(String, Value)> = Vec::new();

    if group == Group::Info {
        if matches!(item, Item::All | Item::Model) {
            sections.push(("model".into(), probe::model::model(&sys)));
        }
        if matches!(item, Item::All | Item::Spec) {
            sections.push(("spec".into(), probe::spec::spec(&sys)));
        }
    } else if group == Group::Config {
        if matches!(item, Item::All | Item::Performance) {
            sections.push(("performance".into(), probe::config::performance(&sys)));
        }
        if matches!(item, Item::All | Item::Charge) {
            sections.push(("charge".into(), probe::config::charge(&sys)));
        }
        if matches!(item, Item::All | Item::Lid) {
            sections.push(("lid".into(), probe::config::lid(&sys)));
        }
        if matches!(item, Item::All | Item::Power) {
            sections.push(("power".into(), probe::config::power(&sys)));
        }
        if matches!(item, Item::All | Item::Io) {
            sections.push(("io".into(), probe::config::io(&sys)));
        }
    } else {
        sections.push(("firmware".into(), probe::firmware::firmware(&sys)));
    }

    if cli.json {
        let mut map = Map::new();
        map.insert("version".into(), json!(VERSION));
        for (name, v) in &sections {
            map.insert(name.clone(), v.clone());
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&Value::Object(map)).unwrap_or_else(|_| "{}".into())
        );
    } else {
        println!("Another Vantage ver {VERSION}");
        let color = !cli.no_color && std::io::stdout().is_terminal();
        print!("{}", output::render(&sections, color));
    }

    if cli.verbose {
        eprintln!("sysfs_root = {}", cli.sysfs_root.display());
    }
}
