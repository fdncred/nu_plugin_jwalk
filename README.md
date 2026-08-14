# nu_plugin_jwalk

Nushell plugin that walks a directory tree. The default engine is [`jwalk`](https://crates.io/crates/jwalk). [`dua-core`](https://crates.io/crates/dua-core), [`ignore`](https://crates.io/crates/ignore), and [`walkdir`](https://crates.io/crates/walkdir) are always available via `--engine`. [`zlob`](https://crates.io/crates/zlob) is optional because it needs the Zig toolchain to compile.

Every engine returns a live Nushell list stream. The first path is sent immediately so pipelines can start before the walk finishes. `--sort` never waits on the plugin thread: `jwalk`, `walkdir`, and `ignore` sort each directory while streaming (`ignore` goes serial so it can); `dua` and `zlob` collect on a walker thread, then flush the sorted items into the same stream.

## Install

Default build (jwalk, dua, ignore, walkdir):

```nushell
cargo install --path .
plugin add ~/.cargo/bin/nu_plugin_jwalk
plugin use jwalk
```

Include the `zlob` engine (Zig must be on `PATH`):

```nushell
cargo install --path . --features zlob
plugin add ~/.cargo/bin/nu_plugin_jwalk
plugin use jwalk
```

A build without `--features zlob` rejects `--engine zlob` and tells you to rebuild.

## Fastest walk

```nushell
# Default engine is jwalk; skip hidden names and heavy dirs
jwalk --skip-hidden --skip-dir [target node_modules .git] ~/src

# Same walk on ignore (gitignore filters are off so flags match the other engines)
jwalk --engine ignore --skip-hidden --skip-dir [target node_modules .git] ~/src

# Same walk on walkdir (always serial)
jwalk --engine walkdir --skip-hidden --skip-dir [target node_modules .git] ~/src

# Same walk on zlob (requires a --features zlob build)
jwalk --engine zlob --skip-hidden --skip-dir [target node_modules .git] ~/src

# Same walk on dua (always stats)
jwalk --engine dua --skip-hidden --skip-dir [target node_modules .git] ~/src

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

| | `jwalk` (default) | `dua` | `ignore` | `walkdir` | `zlob` (`--features zlob`) |
|---|---|---|---|---|---|
| Parallelism | Rayon | work-stealing pool (`crossbeam`) | `WalkParallel` unless `--sort` or `--threads 0`/`1` | always serial | zlob worker pool |
| Path listing | no extra `stat` | native dir metadata on macOS/Windows; extra `stat` on Linux | no extra `stat` unless `--metadata` | no extra `stat` unless `--metadata` | no extra `stat` unless `--metadata` |
| `--metadata` | extra syscall per entry | collected during the walk (size + mtime everywhere; atime/ctime/readonly on Linux) | extra syscall per entry | extra syscall per entry | optional, fetched during the walk |
| `--sort` | per-directory sort while streaming | collect on walker thread, then sort by file name | forces serial per-directory sort while streaming | per-directory sort while streaming | collect on walker thread, then sort by file name |
| `--follow-links` | supported | not supported | supported | supported | supported |
| `--custom` | `process_read_dir` demo | not supported | not supported | not supported | not supported |
| `--order` | ignored | `completion` (default) or `parent-first` | ignored | ignored | ignored |
| `--threads` | Rayon pool (`0` = serial) | worker count (`0`/`1` = 1) | `0`/`1` = serial, else parallel | ignored (serial) | `0` = one worker per CPU |

`dua-core` 3.0 still never follows symlinks. On macOS and Windows it reads type, size, and mtime from the directory listing (`getattrlistbulk` / `FileIdBothDirectoryInfo`) instead of a per-entry `stat`. Linux still stats after `readdir`. `--order` is still the dua-only scheduling switch (`completion` vs `parent-first`). Multi-root `walk_roots` is unused because this command walks one path. dua-cli's `--ignore-from` is a CLI feature, not part of `dua-core`.

`dua` can look slower on Linux path-only / `--count` walks because it still reads metadata there. The other engines skip that syscall unless you pass `--metadata`.

The `ignore` engine uses the same walker as ripgrep, but **gitignore / `.ignore` / hidden filters are off** so `--skip-hidden` and `--skip-dir` match the other engines. Hidden names are skipped only when you pass `--skip-hidden`.

`--custom` requires `--engine jwalk`. `--follow-links` works with every engine except `dua`. `--engine zlob` requires a plugin built with `--features zlob` and `zig` on `PATH`.

## Flags

```nushell
jwalk {flags} <path>

  --engine <jwalk|dua|ignore|walkdir|zlob>   walk engine (default jwalk)
  --verbose                   multi-column output without extra metadata syscalls
  --metadata                  include size, times, readonly (implies record output)
  --sort                      sort by file name (does not delay the stream)
  --skip-hidden               skip names that start with '.'
  --skip-dir <list>           yield these directory names but do not descend
  --follow-links              follow symlinks (all engines except dua)
  --custom                    hard-coded process_read_dir demo (jwalk + verbose)
  --min-depth / --max-depth
  --threads <n>               0 = serial
  --order <completion|parent-first>   dua yield order
  --count                     return only the number of entries
  --debug                     print options and elapsed time to stderr
```

## Benchmarks

`scripts/bench.nu` times every compiled engine on the same tree (count, count+skip, paths, verbose+metadata). It probes `--engine zlob` and skips that engine when the plugin was built without the `zlob` feature:

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
Manuall Sorted (* = winner in each category)
```
╭─#──┬─engine──┬─────case─────┬─entries─┬───min_ms──┬──median_ms─┬─entries_per_sec─╮
│ 0  │ dua     │ count        │ 803,868 │ 17,408.54 │  17,817.65 │          45,116 │
│ 4  │ ignore  │ count        │ 803,868 │  1,166.74 │   1,182.68 │         679,700 │
│ 8  │ jwalk   │ count        │ 803,868 │  1,059.02 │   1,069.40 │         751,697 │
│ 12 │ walkdir │ count        │ 803,868 │  1,573.68 │   1,584.64 │         507,288 │
│ 16 │ zlob   *│ count        │ 803,868 │    881.16 │     895.99 │         897,184 │
│ 1  │ dua     │ count+skip   │   2,988 │      5.57 │       5.77 │         517,683 │
│ 5  │ ignore  │ count+skip   │   2,988 │      7.07 │       7.70 │         388,128 │
│ 9  │ jwalk   │ count+skip   │   2,988 │      5.07 │       5.27 │         567,319 │
│ 13 │ walkdir │ count+skip   │   2,988 │     10.01 │      10.03 │         297,936 │
│ 17 │ zlob   *│ count+skip   │   2,988 │      4.90 │       4.99 │         599,323 │
│ 2  │ dua     │ paths        │ 803,868 │ 16,911.64 │  17,586.22 │          45,710 │
│ 6  │ ignore  │ paths        │ 803,868 │  5,444.10 │   5,651.91 │         142,229 │
│ 10 │ jwalk   │ paths        │ 803,868 │  1,669.90 │   1,760.07 │         456,724 │
│ 14 │ walkdir*│ paths        │ 803,868 │  1,565.31 │   1,625.07 │         494,667 │
│ 18 │ zlob    │ paths        │ 803,868 │  3,275.56 │   3,321.28 │         242,036 │
│ 3  │ dua     │ verbose+meta │ 803,868 │ 15,864.32 │  16,130.63 │          49,835 │
│ 7  │ ignore *│ verbose+meta │ 803,868 │  7,392.89 │   7,771.23 │         103,442 │
│ 11 │ jwalk   │ verbose+meta │ 803,868 │ 21,140.78 │  21,542.28 │          37,316 │
│ 15 │ walkdir │ verbose+meta │ 803,868 │ 21,599.72 │  21,796.54 │          36,881 │
│ 19 │ zlob    │ verbose+meta │ 803,868 │ 10,898.72 │  10,930.55 │          73,543 │
╰─#──┴─engine──┴─────case─────┴─entries─┴───min_ms──┴──median_ms─┴─entries_per_sec─╯
```