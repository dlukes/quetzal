//! Parse an entire EAF file.

use std::{collections::HashMap, fs, io::BufReader, path::Path};

use sxd_document::parser;
use sxd_xpath::{evaluate_xpath, Value};

use super::parser::Parsed;

enum AnnotationContent {
    Freeform(Parsed),
    // TODO: maybe a ref into a vocab collection instead? a pain to pass around though
    ControlledVocab(String),
}

type Milliseconds = u32;

struct Annotation {
    content: AnnotationContent,
    start: Milliseconds,
    end: Milliseconds,
}

struct Tier {
    id: String,
    time_slots: HashMap<String, Milliseconds>,
    annotations: Vec<Annotation>,
}

struct Eaf {
    // TODO: speaker and doc metadata? we probably want to vc those in the repo as well,
    // but we might just fetch them from the db as needed instead of storing them here
    tiers: Vec<Tier>,
}

impl Eaf {
    fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let xml = fs::read_to_string(path).expect("failed to open EAF file");
        let xml = parser::parse(&xml).expect("failed to parse EAF XML");
        let doc = xml.as_document();
        let adoc = evaluate_xpath(&doc, "/ANNOTATION_DOCUMENT").expect("XPath evaluation failed");
        dbg!(&adoc);
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let eaf = Eaf::from_file("19A029F.eaf");
    }
}
