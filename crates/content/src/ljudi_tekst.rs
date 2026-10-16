//! Tekst za pravila banke ljudi: usporedba bez velikih slova i dijakritika (vlastita tablica, bez nove ovisnosti),
//! cijele riječi, abeceda imena, hrvatska množina u porukama i dobni razredi `"18-24"`.

/// Hrvatski oblik imenice uz broj (1 znak, 2 znaka, 5 znakova; 21 znak).
pub(crate) fn mnozina<'a>(n: usize, jedan: &'a str, dva: &'a str, pet: &'a str) -> &'a str {
    let (d, s) = (n % 10, n % 100);
    if d == 1 && s != 11 {
        jedan
    } else if (2..=4).contains(&d) && !(12..=14).contains(&s) {
        dva
    } else {
        pet
    }
}

/// Osnovno slovo bez dijakritika (vlastita tablica, bez nove ovisnosti); `None` = znak ostaje.
fn osnova(c: char) -> Option<&'static str> {
    Some(match c {
        'č' | 'ć' | 'ç' => "c",
        'š' | 'ś' | 'ş' | 'ș' => "s",
        'ž' | 'ź' | 'ż' => "z",
        'đ' => "dj",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ě' | 'ę' | 'ė' => "e",
        'í' | 'ì' | 'î' | 'ï' | 'ī' | 'į' | 'ı' => "i",
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'ő' | 'ō' | 'ø' => "o",
        'ú' | 'ù' | 'û' | 'ü' | 'ű' | 'ū' | 'ů' | 'ų' => "u",
        'ý' | 'ÿ' => "y",
        'ř' => "r",
        'ň' | 'ñ' | 'ń' => "n",
        'ł' | 'ľ' | 'ĺ' => "l",
        'ť' | 'ț' | 'ţ' => "t",
        'ď' => "d",
        'ğ' => "g",
        _ => return None,
    })
}

/// Tekst bez dijakritika, s izvornim velikim slovima („Kovačević" → „Kovacevic", „Đurić" → „Djuric").
pub(crate) fn bez_dijakritika(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let mala = c.to_lowercase().next().unwrap_or(c);
        match osnova(mala) {
            Some(b) if mala != c => {
                let mut z = b.chars();
                if let Some(prvo) = z.next() {
                    out.extend(prvo.to_uppercase());
                    out.push_str(z.as_str());
                }
            }
            Some(b) => out.push_str(b),
            None => out.push(c),
        }
    }
    out
}

/// Cijele riječi teksta, mala slova bez dijakritika (`kasni` ne hvata „kasnije").
pub(crate) fn rijeci(s: &str) -> Vec<String> {
    bez_dijakritika(s)
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

/// Sadrži li tekst (riječi) frazu (riječi) kao uzastopne cijele riječi.
pub(crate) fn sadrzi(tekst: &[String], fraza: &[String]) -> bool {
    !fraza.is_empty() && tekst.windows(fraza.len()).any(|w| w == fraza)
}

/// Ključ punog imena za usporedbu (§3.1): riječi bez dijakritika i velikih slova.
pub(crate) fn kljuc_imena(ime: &str, prezime: &str) -> String {
    rijeci(&format!("{ime} {prezime}")).join(" ")
}

pub(crate) fn latinsko_slovo(c: char) -> bool {
    c.is_ascii_alphabetic()
        || (c.is_alphabetic() && matches!(u32::from(c), 0xC0..=0x24F | 0x1E00..=0x1EFF))
}

pub(crate) fn cirilica(c: char) -> bool {
    matches!(u32::from(c), 0x400..=0x52F)
}

/// Dobni razred `"18-24"` → (18, 24).
pub(crate) fn razred(k: &str) -> Option<(u8, u8)> {
    let (a, b) = k.split_once('-')?;
    let (a, b) = (a.parse().ok()?, b.parse().ok()?);
    (a <= b).then_some((a, b))
}
