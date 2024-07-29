# nu_plugin_jwalk

`jwalk` is an experimental nushell plugin that uses the `jwalk` crate.

## Usage:
```nushell
❯ jwalk --help
View jwalk results of walking the path.

Usage:
  > jwalk {flags} <path>

Flags:
  -h, --help - Display the help message for this command
  -v, --verbose - run in verbose mode with multi-column output
  -s, --sort - sort by file name
  -c, --custom - custom hard-coded walker with process_read_dir
  -k, --skip-hidden - skip hidden files
  -f, --follow-links - follow symbolic links
  -d, --debug - print performance metrics at the end of the table
  -m, --min-depth <Int> - minimum depth to search
  -x, --max-depth <Int> - maximum depth to search
  -t, --threads <Int> - number of rayon threads to use

Parameters:
  path <string>: path to jwalk

Examples:
  Walk the process working directory in debug mode with 2 threads and max depth of 1
  > jwalk --debug --max-depth 1 --threads 2 (pwd)

  Walk the process working directory in debug mode with 2 threads and max depth of 1 using verbose
  > jwalk --debug --verbose --max-depth 1 --threads 2 (pwd)
```

## Examples
### Example 1
jwalk with one column output, with debug info using 4 threads and a max depth of 1
```
❯ jwalk c:\Users\username\source\repos\forks\nushell --debug --threads 4 --max-depth 1
╭────┬──────────────────────────────────────────────────────────────────╮
│  0 │ c:\Users\username\source\repos\forks\nushell                     │
│  1 │ c:\Users\username\source\repos\forks\nushell\.cargo              │
│  2 │ c:\Users\username\source\repos\forks\nushell\.git                │
│  3 │ c:\Users\username\source\repos\forks\nushell\.gitattributes      │
│  4 │ c:\Users\username\source\repos\forks\nushell\.githooks           │
│  5 │ c:\Users\username\source\repos\forks\nushell\.github             │
│  6 │ c:\Users\username\source\repos\forks\nushell\.gitignore          │
│  7 │ c:\Users\username\source\repos\forks\nushell\.vscode             │
│  8 │ c:\Users\username\source\repos\forks\nushell\assets              │
│  9 │ c:\Users\username\source\repos\forks\nushell\benches             │
│ 10 │ c:\Users\username\source\repos\forks\nushell\Cargo.lock          │
│ 11 │ c:\Users\username\source\repos\forks\nushell\Cargo.toml          │
│ 12 │ c:\Users\username\source\repos\forks\nushell\CITATION.cff        │
│ 13 │ c:\Users\username\source\repos\forks\nushell\CODE_OF_CONDUCT.md  │
│ 14 │ c:\Users\username\source\repos\forks\nushell\CONTRIBUTING.md     │
│ 15 │ c:\Users\username\source\repos\forks\nushell\crates              │
│ 16 │ c:\Users\username\source\repos\forks\nushell\Cross.toml          │
│ 17 │ c:\Users\username\source\repos\forks\nushell\devdocs             │
│ 18 │ c:\Users\username\source\repos\forks\nushell\docker              │
│ 19 │ c:\Users\username\source\repos\forks\nushell\LICENSE             │
│ 20 │ c:\Users\username\source\repos\forks\nushell\README.md           │
│ 21 │ c:\Users\username\source\repos\forks\nushell\rust-toolchain.toml │
│ 22 │ c:\Users\username\source\repos\forks\nushell\scripts             │
│ 23 │ c:\Users\username\source\repos\forks\nushell\src                 │
│ 24 │ c:\Users\username\source\repos\forks\nushell\target              │
│ 25 │ c:\Users\username\source\repos\forks\nushell\tests               │
│ 26 │ c:\Users\username\source\repos\forks\nushell\toolkit.nu          │
│ 27 │ c:\Users\username\source\repos\forks\nushell\typos.toml          │
│ 28 │ c:\Users\username\source\repos\forks\nushell\wix                 │
│ 29 │ Running with these options:                                      │
│    │   sort: false                                                    │
│    │   skip_hidden: false                                             │
│    │   follow_links: false                                            │
│    │   min_depth: 0                                                   │
│    │   max_depth: 1                                                   │
│    │   threads: Some(4)                                               │
│    │ Time: 1.0024ms                                                   │
╰────┴──────────────────────────────────────────────────────────────────╯
```
### Example 2
jwalk with multi-column output and debug info with 4 threads and a max depth of 1
```nushell
❯ jwalk /Users/fdncred/src/nushell  --debug --verbose --threads 4 --max-depth 1
╭─#──┬─────depth─────┬─client_state─┬─file_name─┬─full_path─┬─is_dir─┬─is_file─┬─is_symlink─┬─parent_path─┬path_is_symlink┬─accessed─┬─created─┬─modified─┬─size─┬readonly┬─path─╮
│ 0  │             0 │ false        │ nushell   │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ now      │ a year  │ now      │ 1056 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src     │               │          │ ago     │          │      │        │      │
│    │               │              │           │ /nushell  │        │         │            │             │               │          │         │          │      │        │      │
│ 1  │             1 │ false        │ CODE_OF_C │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ a year   │ a year  │ a year   │ 3444 │ false  │   ❎ │
│    │               │              │ ONDUCT.md │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ CODE_OF_C │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ ONDUCT.md │        │         │            │             │               │          │         │          │      │        │      │
│ 2  │             1 │ false        │ Cargo.tom │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ a day    │ a day   │ a day    │ 9040 │ false  │   ❎ │
│    │               │              │ l         │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ Cargo.tom │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ l         │        │         │            │             │               │          │         │          │      │        │      │
│ 3  │             1 │ false        │ toolkit.n │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 2 months │ 2       │ 2 months │ 1964 │ false  │   ❎ │
│    │               │              │ u         │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │ 4    │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ toolkit.n │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ u         │        │         │            │             │               │          │         │          │      │        │      │
│ 4  │             1 │ false        │ .githooks │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 6 months │ 6       │ 6 months │  128 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ .githooks │        │         │            │             │               │          │         │          │      │        │      │
│ 5  │             1 │ false        │ typos.tom │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 3 weeks  │ 3 weeks │ 3 weeks  │  499 │ false  │   ❎ │
│    │               │              │ l         │ ncred/src │        │         │            │ red/src/nus │               │ ago      │  ago    │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ typos.tom │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ l         │        │         │            │             │               │          │         │          │      │        │      │
│ 6  │             1 │ false        │ .fleet    │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ a year   │ a year  │ a year   │   96 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ .fleet    │        │         │            │             │               │          │         │          │      │        │      │
│ 7  │             1 │ false        │ crates    │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ a month  │ a year  │ a month  │ 1344 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ crates    │        │         │            │             │               │          │         │          │      │        │      │
│ 8  │             1 │ false        │ docker    │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 3 weeks  │ a year  │ 3 weeks  │   96 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ docker    │        │         │            │             │               │          │         │          │      │        │      │
│ 9  │             1 │ false        │ .DS_Store │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 2 weeks  │ 6       │ now      │ 6148 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ months  │          │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ .DS_Store │        │         │            │             │               │          │         │          │      │        │      │
│ 10 │             1 │ false        │ LICENSE   │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 6 months │ 6       │ 6 months │ 1094 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ LICENSE   │        │         │            │             │               │          │         │          │      │        │      │
│ 11 │             1 │ false        │ CITATION. │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 2 months │ 2       │ 2 months │  812 │ false  │   ❎ │
│    │               │              │ cff       │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ CITATION. │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ cff       │        │         │            │             │               │          │         │          │      │        │      │
│ 12 │             1 │ false        │ target    │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 2 weeks  │ 2 weeks │ 2 weeks  │  224 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │  ago    │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ target    │        │         │            │             │               │          │         │          │      │        │      │
│ 13 │             1 │ false        │ Cross.tom │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 5 months │ 5       │ 5 months │  666 │ false  │   ❎ │
│    │               │              │ l         │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ Cross.tom │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ l         │        │         │            │             │               │          │         │          │      │        │      │
│ 14 │             1 │ false        │ devdocs   │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 2 months │ 4       │ 2 months │  224 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ devdocs   │        │         │            │             │               │          │         │          │      │        │      │
│ 15 │             1 │ false        │ tests     │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 2 months │ a year  │ 2 months │  544 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ ago     │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ tests     │        │         │            │             │               │          │         │          │      │        │      │
│ 16 │             1 │ false        │ Cargo.loc │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ a day    │ a day   │ a day    │ 1755 │ false  │   ❎ │
│    │               │              │ k         │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │ 80   │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ Cargo.loc │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ k         │        │         │            │             │               │          │         │          │      │        │      │
│ 17 │             1 │ false        │ README.md │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ a month  │ a month │ a month  │ 1228 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │  ago    │ ago      │ 3    │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ README.md │        │         │            │             │               │          │         │          │      │        │      │
│ 18 │             1 │ false        │ .cargo    │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ a month  │ a year  │ a month  │   96 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ .cargo    │        │         │            │             │               │          │         │          │      │        │      │
│ 19 │             1 │ false        │ .gitignor │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 6 months │ 6       │ 6 months │  660 │ false  │   ❎ │
│    │               │              │ e         │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ .gitignor │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ e         │        │         │            │             │               │          │         │          │      │        │      │
│ 20 │             1 │ false        │ CONTRIBUT │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 2 months │ 2       │ 2 months │ 1122 │ false  │   ❎ │
│    │               │              │ ING.md    │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │ 4    │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ CONTRIBUT │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ ING.md    │        │         │            │             │               │          │         │          │      │        │      │
│ 21 │             1 │ false        │ scripts   │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 2 weeks  │ 6       │ 2 weeks  │  416 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ months  │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ scripts   │        │         │            │             │               │          │         │          │      │        │      │
│ 22 │             1 │ false        │ .github   │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 2 weeks  │ a year  │ 2 weeks  │  224 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ .github   │        │         │            │             │               │          │         │          │      │        │      │
│ 23 │             1 │ false        │ .gitattri │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 6 months │ 6       │ 6 months │  111 │ false  │   ❎ │
│    │               │              │ butes     │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ .gitattri │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ butes     │        │         │            │             │               │          │         │          │      │        │      │
│ 24 │             1 │ false        │ benches   │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 2 weeks  │ 6       │ 2 weeks  │  128 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ months  │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ benches   │        │         │            │             │               │          │         │          │      │        │      │
│ 25 │             1 │ false        │ wix       │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 2 months │ a year  │ 2 months │  160 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ ago     │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ wix       │        │         │            │             │               │          │         │          │      │        │      │
│ 26 │             1 │ false        │ rust-tool │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 2 months │ 2       │ 2 months │ 1106 │ false  │   ❎ │
│    │               │              │ chain.tom │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ months  │  ago     │      │        │      │
│    │               │              │ l         │ /nushell/ │        │         │            │ hell        │               │          │ ago     │          │      │        │      │
│    │               │              │           │ rust-tool │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ chain.tom │        │         │            │             │               │          │         │          │      │        │      │
│    │               │              │           │ l         │        │         │            │             │               │          │         │          │      │        │      │
│ 27 │             1 │ false        │ .mailmap  │ /Users/fd │ false  │ true    │ false      │ /Users/fdnc │ false         │ 4 months │ 2 years │ 4 months │ 2036 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │  ago     │  ago    │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ .mailmap  │        │         │            │             │               │          │         │          │      │        │      │
│ 28 │             1 │ false        │ .git      │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ an hour  │ a year  │ 15       │  608 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ minutes  │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │ ago      │      │        │      │
│    │               │              │           │ .git      │        │         │            │             │               │          │         │          │      │        │      │
│ 29 │             1 │ false        │ .vscode   │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ a year   │ a year  │ a year   │  128 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ .vscode   │        │         │            │             │               │          │         │          │      │        │      │
│ 30 │             1 │ false        │ assets    │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ 6 months │ a year  │ 6 months │  160 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │  ago     │ ago     │  ago     │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ assets    │        │         │            │             │               │          │         │          │      │        │      │
│ 31 │             1 │ false        │ src       │ /Users/fd │ true   │ false   │ false      │ /Users/fdnc │ false         │ a week   │ a year  │ a week   │  384 │ false  │   ❎ │
│    │               │              │           │ ncred/src │        │         │            │ red/src/nus │               │ ago      │ ago     │ ago      │      │        │      │
│    │               │              │           │ /nushell/ │        │         │            │ hell        │               │          │         │          │      │        │      │
│    │               │              │           │ src       │        │         │            │             │               │          │         │          │      │        │      │
│ 32 │ skip_hidden:  │ follow_links │ min_depth │        ❎ │ max_de │ threads │ time:      │          ❎ │            ❎ │       ❎ │      ❎ │       ❎ │   ❎ │     ❎ │ sort │
│    │ false         │ : false      │ : 0       │           │ pth: 1 │ : 4     │ 1.727ms    │             │               │          │         │          │      │        │ : fa │
│    │               │              │           │           │        │         │            │             │               │          │         │          │      │        │ lse  │
╰─#──┴─────depth─────┴─client_state─┴─file_name─┴─full_path─┴─is_dir─┴─is_file─┴─is_symlink─┴─parent_path─┴─path_is_symli─┴─accessed─┴─created─┴─modified─┴─size─┴─readon─┴─path─╯
```