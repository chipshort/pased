# pased

pased stands for **p**osition-**a**ware **sed**.
The name is a variantion on sed, except that this does not do a global search and replace,
but only around certain positions in the file.

Its primary use case is fixing compiler errors caused by refactorings.

Performance is currently not a primary concern, but if you have ideas on how to improve it,
feel free to open an issue or a PR.

# Usage

To use pased, you need to supply a few things:

- the regex to search for
- the text to replace the regex with
- the files and positions to search in
- the number of lines to search before and after the position
