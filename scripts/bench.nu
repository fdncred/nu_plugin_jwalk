#!/usr/bin/env nu

# Compare dua, jwalk, and zlob walk times on the same tree.
#
# Usage:
#   nu scripts/bench.nu ~/src/nushell --threads 8 --runs 5
#
# Fairness: dua-core always stats entries while walking. jwalk and zlob
# --count / path listing do not, so they can look faster. verbose+metadata is
# the closer apples-to-apples comparison.
#
# Optional: purge OS caches yourself before a run. On macOS that is `purge`
# (requires privileges) and is not done here.

def main [
    path: path = ".",
    --threads (-t): int = 8,
    --runs (-n): int = 5,
] {
    let root = ($path | path expand)
    if not ($root | path exists) {
        error make { msg: $"path does not exist: ($root)" }
    }

    mut rows = []
    for engine in [dua jwalk zlob] {
        $rows = ($rows | append (bench-case $root $engine $threads $runs "count" {||
            jwalk --engine $engine --count --threads $threads $root
        }))
        $rows = ($rows | append (bench-case $root $engine $threads $runs "count+skip" {||
            jwalk --engine $engine --count --skip-hidden --skip-dir [target .git] --threads $threads $root
        }))
        $rows = ($rows | append (bench-case $root $engine $threads $runs "paths" {||
            jwalk --engine $engine --threads $threads $root
        }))
        $rows = ($rows | append (bench-case $root $engine $threads $runs "verbose+meta" {||
            jwalk --engine $engine --verbose --metadata --threads $threads $root
        }))
    }

    $rows
}

def bench-case [
    root: path,
    engine: string,
    threads: int,
    runs: int,
    name: string,
    walk: closure,
] {
    print --stderr $"warmup ($engine) ($name)"
    do $walk | ignore

    let times = (
        0..<$runs
        | each {|_|
            timeit { do $walk | ignore } | into int
        }
        | sort
    )
    let min_ns = ($times | first)
    let median_ns = ($times | get (($times | length) // 2))
    let entries = match $name {
        "count+skip" => {
            jwalk --engine $engine --count --skip-hidden --skip-dir [target .git] --threads $threads $root
        }
        _ => {
            jwalk --engine $engine --count --threads $threads $root
        }
    }

    {
        engine: $engine
        case: $name
        entries: $entries
        min_ms: (($min_ns / 1_000_000) | math round --precision 2)
        median_ms: (($median_ns / 1_000_000) | math round --precision 2)
        entries_per_sec: (
            if $median_ns == 0 {
                0
            } else {
                ($entries / ($median_ns / 1_000_000_000)) | math round
            }
        )
    }
}
