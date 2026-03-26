import json


def main():
    with open("quotes.json", "r") as f:
        ds = json.load(f)

    quotes = set()
    for d in ds:
        quote = d['Quote']
        if 150 <= len(quote) <= 200:
            a = d['Author'].split(",")[0]
            q = (quote, a)
            quotes.add(q)

    out = [{'Quote': q, 'Author': a} for (q, a) in quotes]

    print(f"Total: {len(ds)}\nQuote: {len(out)}")

    with open("quotes_clean.json", "w") as f:
        json.dump(out, f, indent=2, ensure_ascii=False)


if __name__ == "__main__":
    main()
