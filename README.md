# nu_plugin_jwalk

Nushell plugin that walks a directory tree. The default engine is [`dua-core`](https://crates.io/crates/dua-core). [`jwalk`](https://crates.io/crates/jwalk) and [`zlob`](https://crates.io/crates/zlob) are available for comparison via `--engine`.

## Install

```nushell
cargo install --path .
plugin add ~/.cargo/bin/nu_plugin_jwalk
plugin use jwalk
```

## Fastest walk

```nushell
# zlob can list paths without an extra stat; skip hidden names and heavy dirs
jwalk --engine zlob --skip-hidden --skip-dir [target node_modules .git] ~/src

# Default engine is dua (always stats)
jwalk --skip-hidden --skip-dir [target node_modules .git] ~/src

# Same walk on jwalk
jwalk --engine jwalk --skip-hidden --skip-dir [target node_modules .git] ~/src

# Fastest comparable number (no per-path plugin messages)
jwalk --engine zlob --count --skip-hidden --skip-dir [target node_modules .git] ~/src

# Shallow listing (serial)
jwalk --engine zlob --skip-hidden --max-depth 1 --threads 0 (pwd)

# Verbose columns without extra stat syscalls
jwalk --engine zlob --verbose --skip-hidden --skip-dir [target] (pwd)

# Verbose columns including size and times
jwalk --engine zlob --verbose --metadata --skip-hidden (pwd)
```

Avoid `--sort`, `--follow-links`, `--custom`, and `--verbose --metadata` when you only need paths.

## Engines

| | `dua` (default) | `jwalk` | `zlob` |
|---|---|---|---|
| Parallelism | work-stealing pool (`crossbeam`) | Rayon | zlob worker pool |
| Path listing | always `stat`s | no extra `stat` | no extra `stat` unless `--metadata` |
| `--metadata` | already collected during the walk | extra syscall per entry | optional, fetched during the walk |
| `--sort` | collect, then sort by file name | per-directory sort while streaming | sort collected results by path |
| `--follow-links` | not supported | supported | supported |
| `--custom` | not supported | `process_read_dir` demo | not supported |
| `--order` | `completion` (default) or `parent-first` | ignored | ignored |

`dua` can look slower on path-only / `--count` walks because it always reads metadata. `zlob` and `jwalk` skip that syscall unless you pass `--metadata`.

`--custom` requires `--engine jwalk`. `--follow-links` requires `--engine jwalk` or `--engine zlob`.

## Flags

```
jwalk {flags} <path>

  --engine <dua|jwalk|zlob>   walk engine (default dua)
  --verbose                   multi-column output without extra metadata syscalls
  --metadata                  include size, times, readonly (implies record output)
  --sort                      sort by file name
  --skip-hidden               skip names that start with '.'
  --skip-dir <list>           yield these directory names but do not descend
  --follow-links              follow symlinks (jwalk and zlob)
  --custom                    hard-coded process_read_dir demo (jwalk + verbose)
  --min-depth / --max-depth
  --threads <n>               0 = serial
  --order <completion|parent-first>   dua yield order
  --count                     return only the number of entries
  --debug                     print options and elapsed time to stderr
```

## Benchmarks

`scripts/bench.nu` times all three engines on the same tree (count, count+skip, paths, verbose+metadata):

```nushell
run scripts/bench.nu ~/src/nushell --threads 8 --runs 5
```

The plugin must already be registered (`plugin use jwalk`). Use an explicit `--threads` so the engines are given the same worker count.

## Examples

Walk one level with debug output:

```nushell
jwalk --debug --max-depth 1 --threads 2 (pwd)
```

Verbose records with metadata:

```nushell
jwalk --engine zlob --verbose --metadata --max-depth 1 (pwd)
```
## Benchmarks
#### count
```nushell
jwalk --engine $engine --count --threads $threads $root
```
#### counts+skip
```nushell
jwalk --engine $engine --count --skip-hidden --skip-dir [target .git] --threads $threads $root
```
#### paths
```nushell
jwalk --engine $engine --threads $threads $root
```
#### verbose+meta
```nushell
jwalk --engine $engine --verbose --metadata --threads $threads $root
```

## Real-world results using the bench.nu on the nushell repo
```
╭─#──┬─engine─┬─────case─────┬──entries─┬───min_ms──┬──median_ms─┬─entries_per_sec─╮
│ 0  │ dua    │ count        │  788,709 │  4,395.34 │   4,479.83 │         176,058 │
│ 1  │ dua    │ count+skip   │    2,988 │      5.82 │       6.02 │         496,105 │
│ 2  │ dua    │ paths        │  788,709 │  4,699.32 │   4,803.71 │         164,188 │
│ 3  │ dua    │ verbose+meta │  788,709 │  5,306.36 │   5,388.16 │         146,378 │
│ 4  │ jwalk  │ count        │  788,709 │    892.60 │     940.86 │         838,287 │
│ 5  │ jwalk  │ count+skip   │    2,988 │      4.49 │       4.53 │         659,748 │
│ 6  │ jwalk  │ paths        │  788,709 │  1,449.15 │   1,488.25 │         529,956 │
│ 7  │ jwalk  │ verbose+meta │  788,709 │ 19,071.52 │  19,269.86 │          40,930 │
│ 8  │ zlob   │ count        │  788,709 │    753.78 │     757.12 │       1,041,721 │
│ 9  │ zlob   │ count+skip   │    2,988 │      4.69 │       4.87 │         612,934 │
│ 10 │ zlob   │ paths        │  788,709 │  2,478.71 │   2,587.93 │         304,765 │
│ 11 │ zlob   │ verbose+meta │  788,709 │ 10,157.12 │  10,257.68 │          76,890 │
╰─#──┴─engine─┴─────case─────┴──entries─┴───min_ms──┴──median_ms─┴─entries_per_sec─╯
```
## Manuall Sorted (* = winner in each category)
```
╭─#──┬─engine─┬─────case─────┬──entries─┬───min_ms──┬──median_ms─┬─entries_per_sec─╮
│ 0  │ dua    │ count        │  788,709 │  4,395.34 │   4,479.83 │         176,058 │
│ 4  │ jwalk  │ count        │  788,709 │    892.60 │     940.86 │         838,287 │
│ 8  │ zlob  *│ count        │  788,709 │    753.78 │     757.12 │       1,041,721 │
│ 1  │ dua    │ count+skip   │    2,988 │      5.82 │       6.02 │         496,105 │
│ 5  │ jwalk *│ count+skip   │    2,988 │      4.49 │       4.53 │         659,748 │
│ 9  │ zlob   │ count+skip   │    2,988 │      4.69 │       4.87 │         612,934 │
│ 2  │ dua    │ paths        │  788,709 │  4,699.32 │   4,803.71 │         164,188 │
│ 6  │ jwalk *│ paths        │  788,709 │  1,449.15 │   1,488.25 │         529,956 │
│ 10 │ zlob   │ paths        │  788,709 │  2,478.71 │   2,587.93 │         304,765 │
│ 3  │ dua   *│ verbose+meta │  788,709 │  5,306.36 │   5,388.16 │         146,378 │
│ 7  │ jwalk  │ verbose+meta │  788,709 │ 19,071.52 │  19,269.86 │          40,930 │
│ 11 │ zlob   │ verbose+meta │  788,709 │ 10,157.12 │  10,257.68 │          76,890 │
╰─#──┴─engine─┴─────case─────┴──entries─┴───min_ms──┴──median_ms─┴─entries_per_sec─╯
```
