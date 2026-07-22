use axum::{extract::Path};

const FRONT_VOWELS: [char; 4] = ['e', 'é', 'i', 'í'];
const FRONT_VOWELS_ROUNDED: [char; 4] = ['ö', 'ő', 'ü', 'ű'];
const BACK_VOWELS: [char; 6] = ['a', 'á', 'o', 'ó', 'u', 'ú'];

pub async fn classify(Path(verb): Path<String>) -> String {
   // For now, assume we get a valid verb infinitive
   // TODO: Validate that it's a verb
    let root = verb
        .char_indices()
        .nth_back(1)
        .map_or("", |(idx, _)| &verb[..idx]);

    for c in root.chars().rev() {
        if FRONT_VOWELS.contains(&c) {
            return format!("The verb is a front, unrounded verb");
        }
        else if FRONT_VOWELS_ROUNDED.contains(&c) {
            return format!("The verb is a front, rounded verb");
        }
        else if BACK_VOWELS.contains(&c) {
            return format!("The verb is a back verb");
        }
    }
    format!("Invalid verb")
}
