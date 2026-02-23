use rand::Rng;

use crate::error::AppError;

/// Alphanumeric + special characters for password generation.
const ALPHA_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}|;:,.<>?";

/// BIP39-based word list subset for passphrase generation (common English words).
const PASSPHRASE_WORDS: &[&str] = &[
    "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract",
    "absurd", "abuse", "access", "accident", "account", "accuse", "achieve", "acid",
    "across", "action", "actor", "actual", "adapt", "address", "adjust", "admit",
    "adult", "advance", "advice", "afford", "agree", "airport", "alarm", "album",
    "alert", "alien", "almost", "alone", "alpha", "already", "alter", "always",
    "amazing", "among", "amount", "ancient", "anger", "angle", "animal", "annual",
    "another", "answer", "antenna", "anxiety", "apart", "apology", "appear", "apple",
    "approve", "arena", "argue", "army", "arrow", "artist", "artwork", "assume",
    "attack", "attend", "attract", "auction", "audit", "august", "aunt", "author",
    "avocado", "avoid", "awake", "aware", "awesome", "awful", "awkward", "axis",
    "baby", "bachelor", "bacon", "badge", "balance", "banana", "banner", "barely",
    "barrel", "basic", "basket", "battle", "beach", "beauty", "because", "become",
    "before", "begin", "behind", "believe", "below", "bench", "benefit", "best",
    "betray", "better", "between", "beyond", "bicycle", "bitter", "black", "blade",
    "blame", "blanket", "blast", "bleak", "bless", "blind", "blood", "blossom",
    "blue", "blur", "board", "boat", "body", "bomb", "bone", "bonus",
    "book", "border", "boring", "borrow", "bottom", "bounce", "brain", "brand",
    "brave", "bread", "breeze", "brick", "bridge", "brief", "bright", "bring",
    "broken", "bronze", "brother", "brown", "brush", "bubble", "buddy", "budget",
    "buffalo", "build", "bullet", "bundle", "burden", "burger", "burst", "busy",
    "butter", "cabin", "cable", "cactus", "cage", "cake", "camera", "camp",
    "canal", "cancel", "candy", "cannon", "canyon", "capable", "capital", "captain",
    "carbon", "cargo", "carpet", "carry", "castle", "casual", "catch", "cattle",
    "cause", "ceiling", "celery", "cement", "census", "cereal", "certain", "chair",
    "chalk", "champion", "change", "chaos", "chapter", "charge", "chase", "cheap",
    "check", "cheese", "cherry", "chicken", "chief", "child", "chimney", "choice",
    "chronic", "chunk", "circle", "citizen", "claim", "clap", "clarify", "claw",
    "clean", "clerk", "clever", "cliff", "climb", "clinic", "clock", "close",
    "cloud", "clown", "cluster", "coach", "coconut", "coffee", "collect", "color",
    "column", "combine", "comfort", "common", "company", "concert", "conduct", "confirm",
    "congress", "connect", "consider", "control", "convince", "cookie", "copper", "coral",
    "correct", "cosmic", "cotton", "couch", "country", "couple", "course", "cousin",
    "cover", "crack", "cradle", "craft", "cream", "credit", "cricket", "crime",
    "crisp", "critic", "crop", "cross", "crouch", "crowd", "crucial", "cruel",
    "cruise", "crumble", "crush", "crystal", "cube", "culture", "cupboard", "curious",
    "current", "curtain", "curve", "cushion", "custom", "cycle", "damage", "dance",
    "danger", "daring", "dash", "daughter", "dawn", "debate", "debris", "decade",
    "december", "decide", "decline", "decorate", "decrease", "defense", "define", "delay",
    "deliver", "demand", "denial", "dentist", "depend", "deposit", "depth", "deputy",
    "derive", "describe", "desert", "design", "destroy", "detail", "detect", "develop",
    "device", "devote", "diagram", "diamond", "diary", "diesel", "diet", "differ",
    "digital", "dignity", "dilemma", "dinner", "dinosaur", "direct", "discover", "disease",
    "display", "distance", "divide", "dizzy", "doctor", "document", "dolphin", "domain",
    "donate", "donkey", "donor", "door", "double", "dove", "draft", "dragon",
    "drama", "drastic", "dream", "dress", "drift", "drink", "drive", "drop",
    "drum", "during", "dust", "dutch", "duty", "dwarf", "dynamic", "eager",
    "eagle", "early", "earth", "easily", "east", "easy", "echo", "ecology",
    "economy", "educate", "effort", "eight", "either", "elbow", "elder", "electric",
    "elegant", "element", "elephant", "elite", "else", "embrace", "emerge", "emotion",
    "employ", "empower", "empty", "enable", "enact", "endless", "endorse", "enemy",
    "energy", "enforce", "engage", "engine", "enhance", "enjoy", "enough", "enrich",
    "ensure", "entire", "entry", "envelope", "episode", "equal", "equip", "erode",
    "erosion", "error", "escape", "essay", "estate", "eternal", "evidence", "evil",
    "evolve", "exact", "example", "excess", "exchange", "excite", "exclude", "excuse",
];

/// Generate a random password based on the given type and length.
#[tauri::command]
pub async fn generate_password(
    length: u32,
    password_type: String,
) -> Result<String, AppError> {
    let mut rng = rand::thread_rng();

    match password_type.as_str() {
        "passphrase" => {
            // Generate a passphrase with the specified number of words
            let word_count = (length / 5).max(4).min(12); // ~5 chars per word on average
            let words: Vec<&str> = (0..word_count)
                .map(|_| {
                    let idx = rng.gen_range(0..PASSPHRASE_WORDS.len());
                    PASSPHRASE_WORDS[idx]
                })
                .collect();
            Ok(words.join("-"))
        }
        _ => {
            // Alphanumeric with special characters
            let len = length.max(12).min(128) as usize;
            let password: String = (0..len)
                .map(|_| {
                    let idx = rng.gen_range(0..ALPHA_CHARS.len());
                    ALPHA_CHARS[idx] as char
                })
                .collect();
            Ok(password)
        }
    }
}
