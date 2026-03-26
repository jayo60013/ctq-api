import json
import string
from spellchecker import SpellChecker
from collections import Counter
from better_profanity import profanity

spell = SpellChecker()
profanity.load_censor_words()
all_unknown = Counter()

CONTRACTIONS = {
    "don't": ["do", "not"],
    "doesn't": ["does", "not"],
    "can't": ["can", "not"],
    "won't": ["will", "not"],
    "i'm": ["i", "am"],
    "that's": ["that", "is"],
    "there's": ["there", "is"],
    "wasn't": ["was", "not"],
    "you're": ["you", "are"],
    "we're": ["we", "are"],
    "they're": ["they", "are"],
    "else's": ["else"],
    "another's": ["another"],
    "other's": ["other"],
    "wellbeing": ["well being"],
    "'til": ["until"],
    "'cause": ["because"],
    "men's": ["men"],
}


def is_ascii(s: str) -> bool:
    """Return True if the string contains only standard ASCII characters."""
    try:
        s.encode("ascii")
        return True
    except UnicodeEncodeError:
        return False


def spellcheck_quote(quote: str) -> bool:
    cleaned = quote.translate(str.maketrans(
        "", "", string.punctuation.replace("'", "")))
    raw_words = cleaned.split()

    words = []
    for original in raw_words:
        lower = original.lower()

        # Contraction handling
        if lower in CONTRACTIONS:
            words.extend(CONTRACTIONS[lower])
        else:
            words.append(lower)

    # Spell-check expanded word list
    unknown = spell.unknown(words)
    all_unknown.update(unknown)

    return len(unknown) == 0


def main():
    with open("quotes.json", "r", encoding="utf-8") as f:
        ds = json.load(f)

    quotes = set()

    for d in ds:
        quote = d["Quote"].strip()

        # Length check
        if not (100 <= len(quote) <= 150):
            continue

        # Number check
        if any(ch.isdigit() for ch in quote):
            continue

        # Profanity check
        if profanity.contains_profanity(quote):
            continue

        # ASCII-only check
        if not is_ascii(quote):
            continue

        # Spell check
        if not spellcheck_quote(quote):
            continue

        # Parse author
        author_field = d.get("Author", "").strip()
        if "," in author_field:
            author, source = author_field.split(",", 1)
            author, source = author.strip(), source.strip()
        else:
            author, source = author_field, None

        quotes.add((quote, author, source))

    out = [{"Quote": q, "Author": a, "Source": s} for (q, a, s) in quotes]

    print(f"Total in source: {len(ds)}")
    print(f"Quotes kept: {len(out)}")

    with open("quotes_clean.json", "w", encoding="utf-8") as f:
        json.dump(out, f, indent=2, ensure_ascii=False)


if __name__ == "__main__":
    main()
