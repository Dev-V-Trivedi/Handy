//! Devanagari to Roman letters ("Hinglish") for Hindi, Marathi and Nepali text.
//!
//! Speech models write Hindi in Devanagari. Many people type and read Hindi in
//! English letters ("mera naam Dev hai"). This converts each Devanagari word
//! to the casual spelling people actually use. It is rule based and fully
//! offline. Latin text and punctuation pass through untouched.

/// How a consonant ends.
#[derive(Clone, Copy, PartialEq)]
enum Vowel {
    /// No sign written: carries the inherent "a" (may be dropped by schwa rules).
    Inherent,
    /// A vowel sign, already in Roman letters.
    Sign(&'static str),
    /// Halant: the consonant is joined to the next one with no vowel.
    Halant,
}

enum Item {
    Cons(&'static str, Vowel),
    /// Independent vowel such as अ or ए.
    Letter(&'static str),
    /// Nasal sign (anusvara or chandrabindu). `m` when it sits before a lip consonant.
    Nasal,
    Visarga,
}

/// Words whose everyday spelling differs from the letter-by-letter result.
const COMMON: &[(&str, &str)] = &[
    ("है", "hai"),
    ("हैं", "hain"),
    ("हूँ", "hoon"),
    ("हूं", "hoon"),
    ("था", "tha"),
    ("थी", "thi"),
    ("थे", "the"),
    ("में", "mein"),
    ("मैं", "main"),
    ("नहीं", "nahi"),
    ("नही", "nahi"),
    ("को", "ko"),
    ("से", "se"),
    ("का", "ka"),
    ("की", "ki"),
    ("के", "ke"),
    ("कि", "ki"),
    ("और", "aur"),
    ("यह", "yeh"),
    ("ये", "ye"),
    ("वह", "woh"),
    ("वो", "woh"),
    ("क्या", "kya"),
    ("तो", "toh"),
    ("भी", "bhi"),
    ("पर", "par"),
    ("ही", "hi"),
    ("कुछ", "kuch"),
    ("यहाँ", "yahan"),
    ("वहाँ", "wahan"),
    ("कहाँ", "kahan"),
    ("आप", "aap"),
    ("हम", "hum"),
    ("तुम", "tum"),
    ("मेरा", "mera"),
    ("मेरी", "meri"),
    ("मेरे", "mere"),
    ("तेरा", "tera"),
    ("हमारा", "hamara"),
    ("आपका", "aapka"),
    ("आपकी", "aapki"),
    ("एक", "ek"),
    ("दो", "do"),
    ("तीन", "teen"),
    ("बहुत", "bahut"),
    ("अच्छा", "accha"),
    ("ठीक", "theek"),
    ("हाँ", "haan"),
    ("जी", "ji"),
    ("कर", "kar"),
    ("रहा", "raha"),
    ("रही", "rahi"),
    ("रहे", "rahe"),
    ("लिए", "liye"),
    ("साथ", "saath"),
    ("बात", "baat"),
    ("काम", "kaam"),
    ("कैसे", "kaise"),
    ("कब", "kab"),
    ("क्यों", "kyon"),
    ("अब", "ab"),
    ("फिर", "phir"),
    ("बस", "bas"),
];

fn consonant(c: char) -> Option<&'static str> {
    Some(match c {
        'क' => "k",
        'ख' => "kh",
        'ग' => "g",
        'घ' => "gh",
        'ङ' => "n",
        'च' => "ch",
        'छ' => "chh",
        'ज' => "j",
        'झ' => "jh",
        'ञ' => "n",
        'ट' => "t",
        'ठ' => "th",
        'ड' => "d",
        'ढ' => "dh",
        'ण' => "n",
        'त' => "t",
        'थ' => "th",
        'द' => "d",
        'ध' => "dh",
        'न' => "n",
        'प' => "p",
        'फ' => "ph",
        'ब' => "b",
        'भ' => "bh",
        'म' => "m",
        'य' => "y",
        'र' => "r",
        'ल' => "l",
        'ळ' => "l",
        'व' => "v",
        'श' => "sh",
        'ष' => "sh",
        'स' => "s",
        'ह' => "h",
        // Letters with a built-in dot.
        '\u{0958}' => "q",
        '\u{0959}' => "kh",
        '\u{095A}' => "g",
        '\u{095B}' => "z",
        '\u{095C}' => "r",
        '\u{095D}' => "rh",
        '\u{095E}' => "f",
        _ => return None,
    })
}

fn independent_vowel(c: char) -> Option<&'static str> {
    Some(match c {
        'अ' => "a",
        'आ' => "aa",
        'इ' => "i",
        'ई' => "ee",
        'उ' => "u",
        'ऊ' => "oo",
        'ऋ' => "ri",
        'ए' => "e",
        'ऐ' => "ai",
        'ओ' => "o",
        'औ' => "au",
        'ऑ' => "o",
        _ => return None,
    })
}

fn vowel_sign(c: char) -> Option<&'static str> {
    Some(match c {
        'ा' => "aa",
        'ि' => "i",
        'ी' => "ee",
        'ु' => "u",
        'ू' => "oo",
        'ृ' => "ri",
        'े' => "e",
        'ै' => "ai",
        'ो' => "o",
        'ौ' => "au",
        'ॉ' => "o",
        'ॅ' => "e",
        _ => return None,
    })
}

fn is_devanagari(c: char) -> bool {
    ('\u{0900}'..='\u{097F}').contains(&c) || c == '\u{200D}' || c == '\u{200C}'
}

fn parse(word: &[char]) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    let mut i = 0;
    while i < word.len() {
        let c = word[i];
        // Precomposed nukta letters arrive as base + U+093C. Fold them.
        let (ch, step) = if word.get(i + 1) == Some(&'\u{093C}') {
            let folded = match c {
                'क' => '\u{0958}',
                'ख' => '\u{0959}',
                'ग' => '\u{095A}',
                'ज' => '\u{095B}',
                'ड' => '\u{095C}',
                'ढ' => '\u{095D}',
                'फ' => '\u{095E}',
                other => other,
            };
            (folded, 2)
        } else {
            (c, 1)
        };
        i += step;

        if let Some(r) = consonant(ch) {
            // क्ष and ज्ञ read as one sound.
            if ch == 'क' && word.get(i) == Some(&'\u{094D}') && word.get(i + 1) == Some(&'ष') {
                i += 2;
                items.push(Item::Cons("ksh", Vowel::Inherent));
            } else if ch == 'ज' && word.get(i) == Some(&'\u{094D}') && word.get(i + 1) == Some(&'ञ')
            {
                i += 2;
                items.push(Item::Cons("gy", Vowel::Inherent));
            } else {
                items.push(Item::Cons(r, Vowel::Inherent));
            }
            // Attach a following vowel sign or halant.
            if let Some(&next) = word.get(i) {
                if next == '\u{094D}' {
                    if let Some(Item::Cons(_, v)) = items.last_mut() {
                        *v = Vowel::Halant;
                    }
                    i += 1;
                } else if let Some(sign) = vowel_sign(next) {
                    if let Some(Item::Cons(_, v)) = items.last_mut() {
                        *v = Vowel::Sign(sign);
                    }
                    i += 1;
                }
            }
        } else if let Some(v) = independent_vowel(ch) {
            items.push(Item::Letter(v));
        } else if ch == 'ं' || ch == 'ँ' {
            items.push(Item::Nasal);
        } else if ch == 'ः' {
            items.push(Item::Visarga);
        } else if ('०'..='९').contains(&ch) {
            // Devanagari digits are handled by the caller as plain digits.
        }
    }
    items
}

/// Does this item produce a spoken vowel (used by the schwa rule)?
fn has_vowel(item: &Item, is_last: bool, word_len: usize) -> bool {
    match item {
        Item::Letter(_) => true,
        Item::Cons(_, Vowel::Sign(_)) => true,
        Item::Cons(_, Vowel::Inherent) => !(is_last && word_len > 1),
        _ => false,
    }
}

fn render(items: &[Item], next_is_lip: impl Fn(usize) -> bool) -> String {
    let n = items.len();
    let mut out = String::new();
    // Whether the previous item ended with a pronounced vowel.
    let mut prev_vowel = false;
    for (idx, item) in items.iter().enumerate() {
        let is_last = idx + 1 == n;
        match item {
            Item::Letter(v) => {
                out.push_str(v);
                prev_vowel = true;
            }
            Item::Cons(r, Vowel::Halant) => {
                out.push_str(r);
                prev_vowel = false;
            }
            Item::Cons(r, Vowel::Sign(s)) => {
                out.push_str(r);
                // A long "aa" at the very end of a word is said short: kya, mera.
                let s = if *s == "aa" && is_last { "a" } else { s };
                out.push_str(s);
                prev_vowel = true;
            }
            Item::Cons(r, Vowel::Inherent) => {
                out.push_str(r);
                let next_has_vowel = items
                    .get(idx + 1)
                    .map(|nx| has_vowel(nx, idx + 2 == n, n))
                    .unwrap_or(false);
                let drop = if is_last {
                    n > 1
                } else {
                    idx > 0 && prev_vowel && next_has_vowel
                };
                if drop {
                    prev_vowel = false;
                } else {
                    out.push('a');
                    prev_vowel = true;
                }
            }
            Item::Nasal => {
                out.push(if next_is_lip(idx) { 'm' } else { 'n' });
                prev_vowel = false;
            }
            Item::Visarga => {
                out.push('h');
                prev_vowel = false;
            }
        }
    }
    out
}

fn romanize_word(word: &[char]) -> String {
    let as_string: String = word.iter().collect();
    if let Some((_, roman)) = COMMON.iter().find(|(dev, _)| *dev == as_string) {
        return (*roman).to_string();
    }
    let items = parse(word);
    let lips = |idx: usize| {
        matches!(
            items.get(idx + 1),
            Some(Item::Cons("p" | "ph" | "b" | "bh" | "m", _))
        )
    };
    render(&items, lips)
}

/// Capitalise the first letter of the text and of each sentence.
fn capitalise_sentences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut start = true;
    for c in text.chars() {
        if start && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            start = false;
        } else {
            out.push(c);
            if matches!(c, '.' | '?' | '!') {
                start = true;
            } else if c.is_alphanumeric() {
                start = false;
            }
        }
    }
    out
}

/// Convert every Devanagari word in `text` to Roman letters. Returns the input
/// unchanged when it holds no Devanagari.
pub fn romanize_devanagari(text: &str) -> String {
    if !text.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c)) {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut run: Vec<char> = Vec::new();
    let flush = |run: &mut Vec<char>, out: &mut String| {
        if !run.is_empty() {
            out.push_str(&romanize_word(run));
            run.clear();
        }
    };
    for c in text.chars() {
        if c == '।' || c == '॥' {
            flush(&mut run, &mut out);
            out.push('.');
        } else if ('०'..='९').contains(&c) {
            flush(&mut run, &mut out);
            out.push(char::from_digit(c as u32 - '०' as u32, 10).unwrap_or(c));
        } else if is_devanagari(c) {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    capitalise_sentences(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_words() {
        assert_eq!(romanize_devanagari("मेरा नाम देव है।"), "Mera naam dev hai.");
        assert_eq!(romanize_devanagari("आप कैसे हैं?"), "Aap kaise hain?");
    }

    #[test]
    fn schwa_rules() {
        assert_eq!(romanize_devanagari("कमल"), "Kamal");
        assert_eq!(romanize_devanagari("समझ"), "Samajh");
        assert_eq!(romanize_devanagari("बहुत"), "Bahut");
        assert_eq!(romanize_devanagari("मीटिंग"), "Meeting");
    }

    #[test]
    fn nasals_and_conjuncts() {
        assert_eq!(romanize_devanagari("संसार"), "Sansaar");
        assert_eq!(romanize_devanagari("हम्पी"), "Hampee");
        assert_eq!(romanize_devanagari("क्षमा"), "Kshama");
        assert_eq!(romanize_devanagari("ज्ञान"), "Gyaan");
    }

    #[test]
    fn mixed_text_keeps_english() {
        assert_eq!(
            romanize_devanagari("कल की meeting ठीक है"),
            "Kal ki meeting theek hai"
        );
        assert_eq!(romanize_devanagari("hello world"), "hello world");
    }

    #[test]
    fn digits_and_danda() {
        assert_eq!(romanize_devanagari("५ बजे। ठीक"), "5 baje. Theek");
    }

    #[test]
    fn nukta_letters() {
        assert_eq!(romanize_devanagari("ज़रूर"), "Zaroor");
        assert_eq!(romanize_devanagari("फ़ोन"), "Fon");
    }
}
