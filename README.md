# ebnf2pest

# Introduction
This is a utility to parse the ISO 39075:2024 GQL language specification in EBNF format and emit a Pest file.
It worked for my purpose to convert the EBNF grammar from [here](https://github.com/zmajeed/ebnfparser/blob/main/docs/gqlgrammar.quotedliterals.txt) to an almost-working pest file (I needed to fix a couple of things by hand, such as a few character rules that were not quoted correctly, and some of the rules in the original EBNF grammar are left-recursive, which pest really doesn't like).

Overall I could probably have made my life easier by using the XML version of the grammar as input, which seems to be the authoritative version anyways.

## Licensing
The code in this repository is dual-licensed as MIT OR Apache-2.0, with the exception of:
- resources/gqlgrammar.quotedliterals.txt: Licensed under MIT only.
