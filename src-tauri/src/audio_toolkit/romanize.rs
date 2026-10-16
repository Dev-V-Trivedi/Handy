//! Indic scripts to Roman letters ("Hinglish", "Tenglish" and so on).
//!
//! Speech models write Hindi in Devanagari. Many people type and read Hindi in
//! English letters ("mera naam Dev hai"). This converts each Devanagari word
//! to the casual spelling people actually use. It is rule based and fully
//! offline. Latin text and punctuation pass through untouched.
//!
//! Bengali, Gurmukhi, Gujarati, Odia, Tamil, Telugu, Kannada and Malayalam
//! sit at the same offsets inside their Unicode blocks as Devanagari, so each
//! letter is folded onto its Devanagari twin and then read by the same rules.
//! Dravidian scripts keep every inherent "a" (Kannada is "Kannada", not
//! "Kannad").

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

/// Sample text handed to Whisper as its initial prompt in Hinglish mode. It is
/// Hindi in English letters with English words kept in English, which steers
/// the model to write the same way.
pub const HINGLISH_PROMPT: &str = "Toh yeh abhi test ke liye hai. What we deployed is live, once everything is fixed, toh ek baar check karte hain. Meeting mein action items aur next steps discuss karenge, phir update bhej dena.";

/// Roman-letter sample prompt for a language code (`hi`, `te`, `kn-IN` ...).
/// `None` when the language has no Roman mode.
pub fn roman_prompt(lang: &str) -> Option<&'static str> {
    Some(match lang.split(['-', '_']).next().unwrap_or("") {
        // Auto-detect: the language is unknown up front, so use the Hindi sample.
        // Anything still written in a native script is converted afterwards.
        "hi" | "ne" | "auto" => HINGLISH_PROMPT,
        "mr" => "Mi aata meeting madhe aahe. Marathi madhe bolto, pan English words pan vaparto. Project update kasa aahe, next steps kay aahet, deploy zala ka?",
        "te" => "Nenu ippudu meeting lo unnanu. Telugu lo matladutunnanu, kani English words kuda vadutanu. Project update ela undi, next steps enti, deploy chesara?",
        "kn" => "Naanu ivattu meeting alli iddene. Kannada alli maatanaaduttiddene, aadre English words kooda ballutteve. Project update hege ide, next steps yenu, deploy maadidira?",
        "gu" => "Hu atyare meeting ma chhu. Gujarati ma vaat karu chhu, pan English words pan vaparu chhu. Project update kem chhe, next steps shu chhe, deploy kari didhu?",
        "ta" => "Naan ippo meeting la irukken. Tamil la pesuren, aana English words um use pannuven. Project update eppadi irukku, next steps enna, deploy pannitingala?",
        "bn" => "Ami ekhon meeting e achhi. Bangla te kotha bolchi, kintu English words o bolbo. Project update kemon achhe, next steps ki, deploy kora hoyeche?",
        "ml" => "Njan ippol meeting il aanu. Malayalam il samsarikkunnu, pakshe English words um upayogikkum. Project update engane und, next steps entha, deploy cheythoo?",
        "pa" => "Main hun meeting vich haan. Punjabi vich gall kardan, par English words vi vartanda haan. Project update kiven aa, next steps ki ne, deploy ho gaya?",
        "ur" => "Main abhi meeting mein hoon. Urdu mein baat kar raha hoon, lekin English words bhi istemal karta hoon. Project update kaisa hai, next steps kya hain, deploy ho gaya?",
        _ => return None,
    })
}

/// English words that models write in Devanagari, mapped back to English.
const ENGLISH_LOANWORDS: &[(&str, &str)] = &[
    // Language names and other proper nouns.
    ("इंग्लिश", "English"),
    ("इंग्लीश", "English"),
    ("इंग्लिष्", "English"),
    ("इंग्लीष्", "English"),
    ("अंग्रेज़ी", "English"),
    ("अंग्रेजी", "English"),
    ("तेलुगु", "Telugu"),
    ("तेलुगू", "Telugu"),
    ("कन्नड", "Kannada"),
    ("कन्नडा", "Kannada"),
    ("गुजराती", "Gujarati"),
    ("हिंदी", "Hindi"),
    ("हिन्दी", "Hindi"),
    ("मराठी", "Marathi"),
    ("तमिल", "Tamil"),
    ("तमिळ", "Tamil"),
    ("मलयालम", "Malayalam"),
    ("बंगाली", "Bengali"),
    ("बांग्ला", "Bangla"),
    ("पंजाबी", "Punjabi"),
    ("उर्दू", "Urdu"),
    ("हिंग्लिश", "Hinglish"),
    ("ओशाबे", "Oshabe"),
    // Everyday English that speech models write in Indic scripts.
    ("एंड", "and"),
    ("बट", "but"),
    ("वर्क", "work"),
    ("प्रॉपर", "proper"),
    ("प्रॉपरली", "properly"),
    ("रीजनल", "regional"),
    ("रीजनल", "regional"),
    ("लैंग्वेज", "language"),
    ("लैंग्वेजेस", "languages"),
    ("मिसिंग", "missing"),
    ("स्पेलिंग", "spelling"),
    ("स्पेलिंग्स", "spellings"),
    ("करेक्ट", "correct"),
    ("ट्रांसक्रिप्ट", "transcript"),
    ("ट्रांसक्रिप्शन", "transcription"),
    ("समरी", "summary"),
    ("नोट्स", "notes"),
    ("एक्शन", "action"),
    ("आइटम्स", "items"),
    ("नेक्स्ट", "next"),
    ("स्टेप्स", "steps"),
    ("डिस्कस", "discuss"),
    ("प्रॉब्लम", "problem"),
    ("इश्यू", "issue"),
    ("रिलीज", "release"),
    ("रिलीज़", "release"),
    ("बिल्ड", "build"),
    ("टीम", "team"),
    ("मैनेजर", "manager"),
    ("स्पीड", "speed"),
    ("फास्ट", "fast"),
    ("स्लो", "slow"),
    ("मॉडल", "model"),
    ("सेटिंग", "setting"),
    ("सेटिंग्स", "settings"),
    ("टेस्ट", "test"),
    ("मीटिंग", "meeting"),
    ("डिप्लॉय", "deploy"),
    ("डिप्लोय", "deploy"),
    ("फिक्स", "fix"),
    ("फिक्स्ड", "fixed"),
    ("चेक", "check"),
    ("अपडेट", "update"),
    ("ऑफिस", "office"),
    ("ऑफ़िस", "office"),
    ("प्रोजेक्ट", "project"),
    ("क्लाइंट", "client"),
    ("कस्टमर", "customer"),
    ("सर्वर", "server"),
    ("डेटा", "data"),
    ("वेबसाइट", "website"),
    ("ऐप", "app"),
    ("एप", "app"),
    ("ऐप्लिकेशन", "application"),
    ("मैसेज", "message"),
    ("मेसेज", "message"),
    ("ईमेल", "email"),
    ("कॉल", "call"),
    ("फोन", "phone"),
    ("फ़ोन", "phone"),
    ("लाइव", "live"),
    ("लाइफ", "life"),
    ("वन्स", "once"),
    ("एवरीथिंग", "everything"),
    ("इज", "is"),
    ("वाट", "what"),
    ("वॉट", "what"),
    ("वी", "we"),
    ("ओके", "okay"),
    ("ओकै", "okay"),
    ("थैंक्स", "thanks"),
    ("थैंक", "thank"),
    ("सॉरी", "sorry"),
    ("प्लीज", "please"),
    ("प्लीज़", "please"),
    ("हेलो", "hello"),
    ("हैलो", "hello"),
    ("हाय", "hi"),
    ("बाय", "bye"),
    ("प्रॉब्लम", "problem"),
    ("इश्यू", "issue"),
    ("बग", "bug"),
    ("कोड", "code"),
    ("फाइल", "file"),
    ("फोल्डर", "folder"),
    ("लिंक", "link"),
    ("पासवर्ड", "password"),
    ("अकाउंट", "account"),
    ("पेमेंट", "payment"),
    ("इनवॉइस", "invoice"),
    ("रिपोर्ट", "report"),
    ("प्लान", "plan"),
    ("टीम", "team"),
    ("मैनेजर", "manager"),
    ("डेडलाइन", "deadline"),
    ("टास्क", "task"),
    ("शेड्यूल", "schedule"),
    ("रिमाइंडर", "reminder"),
    ("नोट्स", "notes"),
    ("सेटिंग", "setting"),
    ("सेटिंग्स", "settings"),
    ("इंस्टॉल", "install"),
    ("डाउनलोड", "download"),
    ("अपलोड", "upload"),
    ("लॉगिन", "login"),
    ("सपोर्ट", "support"),
    ("सर्विस", "service"),
    ("टाइम", "time"),
    ("फर्स्ट", "first"),
    ("नेक्स्ट", "next"),
    ("स्टेप", "step"),
    ("अप्रूव", "approve"),
    ("सेंड", "send"),
    ("शेयर", "share"),
    ("स्क्रीन", "screen"),
    ("कंप्यूटर", "computer"),
    ("लैपटॉप", "laptop"),
    ("मोबाइल", "mobile"),
    ("इंटरनेट", "internet"),
    ("नेटवर्क", "network"),
    ("ऑनलाइन", "online"),
    ("ऑफलाइन", "offline"),
];

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
    ("अभी", "abhi"),
    ("कभी", "kabhi"),
    ("सब", "sab"),
    ("जब", "jab"),
    ("तब", "tab"),
    ("यही", "yahi"),
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
        '\u{0929}' => "n",
        '\u{0931}' => "r",
        '\u{0934}' => "zh",
        '\u{095F}' => "y",
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
        '\u{090D}' => "e",
        '\u{090E}' => "e",
        '\u{0912}' => "o",
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
        '\u{0946}' => "e",
        '\u{094A}' => "o",
        _ => return None,
    })
}

fn is_joiner(c: char) -> bool {
    c == '\u{200D}' || c == '\u{200C}'
}

/// Any Indic letter this module can read (Devanagari through Malayalam).
fn is_indic(c: char) -> bool {
    ('\u{0900}'..='\u{0D7F}').contains(&c) || is_joiner(c)
}

/// Dravidian scripts (Tamil, Telugu, Kannada, Malayalam) say every written "a".
fn is_dravidian(c: char) -> bool {
    ('\u{0B80}'..='\u{0D7F}').contains(&c)
}

/// Fold a letter from another Indic script onto its Devanagari twin. The
/// blocks share one layout, so the position inside the block is the same.
fn fold(c: char) -> char {
    match c {
        // Malayalam chillu letters (bare n, r, l, l, n, k).
        '\u{0D7A}' | '\u{0D7B}' => 'ण',
        '\u{0D7C}' => 'र',
        '\u{0D7D}' => 'ल',
        '\u{0D7E}' => 'ळ',
        '\u{0D7F}' => 'क',
        // Gurmukhi tippi is the nasal sign.
        '\u{0A70}' => 'ं',
        '\u{0980}'..='\u{0D79}' => char::from_u32(0x0900 + (c as u32 & 0x7F)).unwrap_or(c),
        _ => c,
    }
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

fn render(items: &[Item], keep_schwa: bool, next_is_lip: impl Fn(usize) -> bool) -> String {
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
                let drop = if keep_schwa {
                    false
                } else if is_last {
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
                // South Indian words end in "m": Namaskaaram, Malayaalam.
                let word_final_m = keep_schwa && is_last;
                out.push(if word_final_m || next_is_lip(idx) {
                    'm'
                } else {
                    'n'
                });
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

fn romanize_word(word: &[char], keep_schwa: bool) -> String {
    // A word may end in a bare halant (Telugu writes "meeting" as మీటింగ్).
    let trimmed: &[char] = match word.split_last() {
        Some(('\u{094D}', rest)) if !rest.is_empty() => rest,
        _ => word,
    };
    let as_string: String = word.iter().collect();
    if let Some((_, roman)) = COMMON.iter().find(|(dev, _)| *dev == as_string) {
        return (*roman).to_string();
    }
    if let Some((_, english)) = ENGLISH_LOANWORDS.iter().find(|(dev, _)| *dev == as_string) {
        return (*english).to_string();
    }
    let trimmed_string: String = trimmed.iter().collect();
    if let Some((_, english)) = ENGLISH_LOANWORDS
        .iter()
        .find(|(dev, _)| *dev == trimmed_string)
    {
        return (*english).to_string();
    }
    let items = parse(word);
    let lips = |idx: usize| {
        matches!(
            items.get(idx + 1),
            Some(Item::Cons("p" | "ph" | "b" | "bh" | "m", _))
        )
    };
    render(&items, keep_schwa, lips)
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

/// Convert every Indic-script word in `text` to Roman letters. Returns the
/// input unchanged when it holds none.
pub fn romanize_indic(text: &str) -> String {
    if !text.chars().any(|c| is_indic(c) && !is_joiner(c)) {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut run: Vec<char> = Vec::new();
    let mut dravidian = false;
    let flush = |run: &mut Vec<char>, dravidian: &mut bool, out: &mut String| {
        if !run.is_empty() {
            out.push_str(&romanize_word(run, *dravidian));
            run.clear();
        }
        *dravidian = false;
    };
    for c in text.chars() {
        if c == '।' || c == '॥' {
            flush(&mut run, &mut dravidian, &mut out);
            out.push('.');
        } else if is_indic(c) && !is_joiner(c) && is_digit_of_block(c) {
            flush(&mut run, &mut dravidian, &mut out);
            let digit = (c as u32 & 0x7F) - 0x66;
            out.push(char::from_digit(digit, 10).unwrap_or(c));
        } else if is_indic(c) {
            if run.is_empty() && is_dravidian(c) {
                dravidian = true;
            }
            run.push(fold(c));
        } else {
            flush(&mut run, &mut dravidian, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut dravidian, &mut out);
    capitalise_sentences(&out)
}

/// Digit positions (0x66..=0x6F) inside an Indic block.
fn is_digit_of_block(c: char) -> bool {
    ('\u{0900}'..='\u{0D7F}').contains(&c) && (0x66..=0x6F).contains(&(c as u32 & 0x7F))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_words() {
        assert_eq!(romanize_indic("मेरा नाम देव है।"), "Mera naam dev hai.");
        assert_eq!(romanize_indic("आप कैसे हैं?"), "Aap kaise hain?");
    }

    #[test]
    fn schwa_rules() {
        assert_eq!(romanize_indic("कमल"), "Kamal");
        assert_eq!(romanize_indic("समझ"), "Samajh");
        assert_eq!(romanize_indic("बहुत"), "Bahut");
        assert_eq!(romanize_indic("मीटिंग"), "Meeting");
    }

    #[test]
    fn nasals_and_conjuncts() {
        assert_eq!(romanize_indic("संसार"), "Sansaar");
        assert_eq!(romanize_indic("हम्पी"), "Hampee");
        assert_eq!(romanize_indic("क्षमा"), "Kshama");
        assert_eq!(romanize_indic("ज्ञान"), "Gyaan");
    }

    #[test]
    fn mixed_text_keeps_english() {
        assert_eq!(
            romanize_indic("कल की meeting ठीक है"),
            "Kal ki meeting theek hai"
        );
        assert_eq!(romanize_indic("hello world"), "hello world");
    }

    #[test]
    fn english_words_written_in_devanagari_come_back_as_english() {
        assert_eq!(
            romanize_indic("तो यह अभी टेस्ट के लिए है वाट वी डिप्लॉय"),
            "Toh yeh abhi test ke liye hai what we deploy"
        );
    }

    #[test]
    fn prompts_are_roman_script() {
        assert!(HINGLISH_PROMPT.is_ascii());
        for lang in [
            "hi", "mr", "ne", "te", "kn", "gu", "ta", "bn", "ml", "pa", "ur",
        ] {
            assert!(roman_prompt(lang).expect(lang).is_ascii(), "{lang}");
        }
        assert!(roman_prompt("kn-IN").is_some());
        assert!(roman_prompt("auto").is_some());
        assert!(roman_prompt("en").is_none());
    }

    #[test]
    fn language_names_are_spelled_properly() {
        assert_eq!(romanize_indic("इंग्लिश"), "English");
        assert_eq!(romanize_indic("गुजराती"), "Gujarati");
        assert_eq!(romanize_indic("कन्नड"), "Kannada");
        // Same words written in their own scripts.
        assert_eq!(romanize_indic("ಕನ್ನಡ"), "Kannada");
        assert_eq!(romanize_indic("తెలుగు"), "Telugu");
        assert_eq!(romanize_indic("ગુજરાતી"), "Gujarati");
        assert_eq!(romanize_indic("ఇంగ్లీష్"), "English");
    }

    #[test]
    fn south_indian_scripts_keep_every_a() {
        assert_eq!(romanize_indic("మీరు"), "Meeru");
        assert_eq!(romanize_indic("నమస్కారం"), "Namaskaaram");
        assert_eq!(romanize_indic("ನಮಸ್ಕಾರ"), "Namaskaara");
        assert_eq!(romanize_indic("மலர்"), "Malar");
        assert_eq!(romanize_indic("മലയാളം"), "Malayaalam");
    }

    #[test]
    fn english_inside_south_indian_text() {
        assert_eq!(romanize_indic("మీటింగ్ రేపు"), "Meeting repu");
        assert_eq!(romanize_indic("నేను 5 గంటలకు"), "Nenu 5 gantalaku");
    }

    #[test]
    fn bengali_and_punjabi() {
        assert_eq!(romanize_indic("আমি"), "Aami");
        assert_eq!(romanize_indic("ਪੰਜਾਬੀ"), "Punjabi");
    }

    #[test]
    fn digits_and_danda() {
        assert_eq!(romanize_indic("५ बजे। ठीक"), "5 baje. Theek");
    }

    #[test]
    fn nukta_letters() {
        assert_eq!(romanize_indic("ज़रूर"), "Zaroor");
        assert_eq!(romanize_indic("फ़र्क"), "Fark");
    }
}
