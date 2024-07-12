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
  -o, --original - run the original jwalk, 1 column
  -s, --sort - sort by file name
  -c, --custom - custom walker with process_read_dir
  -k, --skip-hidden - skip hidden files
  -f, --follow-links - follow symbolic links
  -d, --debug - print performance metrics at the end of the table
  -m, --min-depth <Int> - minimum depth to search
  -x, --max-depth <Int> - maximum depth to search
  -t, --threads <Int> - number of rayon threads to use

Parameters:
  path <string>: path to jwalk

Examples:
  This is the example descripion
  > some pipeline involving jwalk
```

## Examples
### Example 1
jwalk with one column output, with debug info using 4 threads and a max depth of 1
```
❯ jwalk c:\Users\username\source\repos\forks\nushell -o -d --threads 4 --max-depth 1
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
❯ jwalk c:\Users\username\source\repos\forks\nushell -d --threads 4 --max-depth 1
╭─#──┬───────────────────────────────path───────────────────────────────┬───depth────┬─client_state─┬─file_name─┬─is_dir─┬─is_file─┬─is_symlink─┬─parent_path─┬─accessed─┬─created─┬─modified─┬─size─┬readonly╮
│ 0  │ c:\Users\username\source\repos\forks\nushell                     │          0 │ false        │ nushell   │ true   │ false   │ false      │ c:\Users\us │ now      │ a year  │ 21 hours │ 1228 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │          │ ago     │  ago     │ 8    │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks         │          │         │          │      │        │
│ 1  │ c:\Users\username\source\repos\forks\nushell\.cargo              │          1 │ false        │ .cargo    │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ 9 months │    0 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │  ago     │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 2  │ c:\Users\username\source\repos\forks\nushell\.git                │          1 │ false        │ .git      │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ 16 hours │ 4096 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │  ago     │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 3  │ c:\Users\username\source\repos\forks\nushell\.gitattributes      │          1 │ false        │ .gitattri │ false  │ true    │ false      │ c:\Users\us │ 5        │ a year  │ a year   │  113 │ false  │
│    │                                                                  │            │              │ butes     │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 4  │ c:\Users\username\source\repos\forks\nushell\.githooks           │          1 │ false        │ .githooks │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ a year   │    0 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 5  │ c:\Users\username\source\repos\forks\nushell\.github             │          1 │ false        │ .github   │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ a day    │ 4096 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 6  │ c:\Users\username\source\repos\forks\nushell\.gitignore          │          1 │ false        │ .gitignor │ false  │ true    │ false      │ c:\Users\us │ 5        │ a year  │ a year   │  712 │ false  │
│    │                                                                  │            │              │ e         │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 7  │ c:\Users\username\source\repos\forks\nushell\.vscode             │          1 │ false        │ .vscode   │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ a year   │    0 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 8  │ c:\Users\username\source\repos\forks\nushell\assets              │          1 │ false        │ assets    │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ a year   │    0 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 9  │ c:\Users\username\source\repos\forks\nushell\benches             │          1 │ false        │ benches   │ true   │ false   │ false      │ c:\Users\us │ 2        │ a year  │ a day    │    0 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 10 │ c:\Users\username\source\repos\forks\nushell\Cargo.lock          │          1 │ false        │ Cargo.loc │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ a year  │ 21 hours │ 1826 │ false  │
│    │                                                                  │            │              │ k         │        │         │            │ ername\sour │  ago     │ ago     │  ago     │ 51   │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 11 │ c:\Users\username\source\repos\forks\nushell\Cargo.toml          │          1 │ false        │ Cargo.tom │ false  │ true    │ false      │ c:\Users\us │ 2        │ a year  │ 21 hours │ 9327 │ false  │
│    │                                                                  │            │              │ l         │        │         │            │ ername\sour │ minutes  │ ago     │  ago     │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 12 │ c:\Users\username\source\repos\forks\nushell\CITATION.cff        │          1 │ false        │ CITATION. │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ a month │ a month  │  838 │ false  │
│    │                                                                  │            │              │ cff       │        │         │            │ ername\sour │  ago     │  ago    │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 13 │ c:\Users\username\source\repos\forks\nushell\CODE_OF_CONDUCT.md  │          1 │ false        │ CODE_OF_C │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ a year  │ a year   │ 3520 │ false  │
│    │                                                                  │            │              │ ONDUCT.md │        │         │            │ ername\sour │  ago     │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 14 │ c:\Users\username\source\repos\forks\nushell\CONTRIBUTING.md     │          1 │ false        │ CONTRIBUT │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ a year  │ a month  │ 1146 │ false  │
│    │                                                                  │            │              │ ING.md    │        │         │            │ ername\sour │  ago     │ ago     │ ago      │ 0    │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 15 │ c:\Users\username\source\repos\forks\nushell\crates              │          1 │ false        │ crates    │ true   │ false   │ false      │ c:\Users\us │ 2        │ a year  │ 3 weeks  │ 8192 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 16 │ c:\Users\username\source\repos\forks\nushell\Cross.toml          │          1 │ false        │ Cross.tom │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ a year  │ 11       │  684 │ false  │
│    │                                                                  │            │              │ l         │        │         │            │ ername\sour │  ago     │ ago     │ months   │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │         │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 17 │ c:\Users\username\source\repos\forks\nushell\devdocs             │          1 │ false        │ devdocs   │ true   │ false   │ false      │ c:\Users\us │ 5        │ 7       │ a month  │ 4096 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ months  │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │ ago     │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 18 │ c:\Users\username\source\repos\forks\nushell\docker              │          1 │ false        │ docker    │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ 3 days   │    0 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 19 │ c:\Users\username\source\repos\forks\nushell\LICENSE             │          1 │ false        │ LICENSE   │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ a year  │ a year   │ 1115 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │  ago     │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 20 │ c:\Users\username\source\repos\forks\nushell\README.md           │          1 │ false        │ README.md │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ a year  │ 3 weeks  │ 1252 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │  ago     │ ago     │ ago      │ 0    │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 21 │ c:\Users\username\source\repos\forks\nushell\rust-toolchain.toml │          1 │ false        │ rust-tool │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ a year  │ 3 months │ 1125 │ false  │
│    │                                                                  │            │              │ chain.tom │        │         │            │ ername\sour │  ago     │ ago     │  ago     │      │        │
│    │                                                                  │            │              │ l         │        │         │            │ ce\repos\fo │          │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 22 │ c:\Users\username\source\repos\forks\nushell\scripts             │          1 │ false        │ scripts   │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ a day    │ 4096 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 23 │ c:\Users\username\source\repos\forks\nushell\src                 │          1 │ false        │ src       │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ a day    │ 4096 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 24 │ c:\Users\username\source\repos\forks\nushell\target              │          1 │ false        │ target    │ true   │ false   │ false      │ c:\Users\us │ 16 hours │ 2       │ 2 months │    0 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │  ago     │ months  │  ago     │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │ ago     │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 25 │ c:\Users\username\source\repos\forks\nushell\tests               │          1 │ false        │ tests     │ true   │ false   │ false      │ c:\Users\us │ 2        │ a year  │ a month  │ 4096 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 26 │ c:\Users\username\source\repos\forks\nushell\toolkit.nu          │          1 │ false        │ toolkit.n │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ a year  │ a month  │ 2023 │ false  │
│    │                                                                  │            │              │ u         │        │         │            │ ername\sour │  ago     │ ago     │ ago      │ 8    │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 27 │ c:\Users\username\source\repos\forks\nushell\typos.toml          │          1 │ false        │ typos.tom │ false  │ true    │ false      │ c:\Users\us │ 16 hours │ 4       │ 2 weeks  │  527 │ false  │
│    │                                                                  │            │              │ l         │        │         │            │ ername\sour │  ago     │ months  │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │          │ ago     │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 28 │ c:\Users\username\source\repos\forks\nushell\wix                 │          1 │ false        │ wix       │ true   │ false   │ false      │ c:\Users\us │ 5        │ a year  │ a month  │    0 │ false  │
│    │                                                                  │            │              │           │        │         │            │ ername\sour │ minutes  │ ago     │ ago      │      │        │
│    │                                                                  │            │              │           │        │         │            │ ce\repos\fo │ ago      │         │          │      │        │
│    │                                                                  │            │              │           │        │         │            │ rks\nushell │          │         │          │      │        │
│ 29 │ sort: false                                                      │ skip_hidde │ follow_links │ min_depth │ max_de │ threads │ time:      │          ❎ │       ❎ │      ❎ │       ❎ │   ❎ │     ❎ │
│    │                                                                  │ n: false   │ : false      │ : 0       │ pth: 1 │ : 4     │ 4.8537ms   │             │          │         │          │      │        │
╰─#──┴───────────────────────────────path───────────────────────────────┴───depth────┴─client_state─┴─file_name─┴─is_dir─┴─is_file─┴─is_symlink─┴─parent_path─┴─accessed─┴─created─┴─modified─┴─size─┴─readon─╯
```