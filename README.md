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

The syntax is as follows:

```bash
cargo check |& pased --lines 2 --rust warnings 'regex' 'replacement'
```

## Example

Here is the example that motivated me to write this tool:
Let's say you want to make the following change:

```diff
- pub fn mock_env() -> Env {
+ pub fn mock_env(api: &MockApi) -> Env {
```

The problem is that this breaks all your tests. Those tests already have a `MockApi` available as `deps.api`,
but because of the structure of the code, you cannot just replace `mock_env()` with `mock_env(&deps.api)`
because the borrow checker will complain.
Your fixed tests should look something like this:

```diff
- call_something(deps.as_mut(), mock_env(), /* ... */);
+ let env = mock_env(&deps.api);
+ call_something(deps.as_mut(), env, /* ... */);
```

To achive this, your call could look something like this:

```bash
cargo check --tests |& pased --lines 2 --rust error "(?ms)(let (?:mut )?[\w_]+ = [\w_]+\(\s*[\w_.()]*,\s*)mock_env\(\)," "let env = mock_env(&deps.api); \$1 env,"
```
