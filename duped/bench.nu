cargo build --release

def avg_dur [times: list<duration>] {
    ($times | each { into int } | math avg) * 1ns
}

def stddev_dur [times: list<duration>] {
    ($times | each { into int } | math stddev) * 1ns
}

def bench_find [] {
    let count = (^find /data/data/com.termux -type f -not -type l | lines | length)
    let times = [
        (timeit { ^find /data/data/com.termux -type f -not -type l | lines | length }),
        (timeit { ^find /data/data/com.termux -type f -not -type l | lines | length }),
        (timeit { ^find /data/data/com.termux -type f -not -type l | lines | length }),
    ]
    {
        strategy: "find (ref)",
        count: $count,
        avg: (avg_dur $times),
        delta: (stddev_dur $times),
    }
}

def bench [strategy: string] {
    let count = (~/build/release/duped /data/data/com.termux $"-s=($strategy)" --silent
        | lines
        | where {|l| $l =~ "Found"}
        | parse "Found {n} files:"
        | get n.0
        | into int)
    let times = [
        (timeit { ~/build/release/duped /data/data/com.termux $"-s=($strategy)" --silent }),
        (timeit { ~/build/release/duped /data/data/com.termux $"-s=($strategy)" --silent }),
        (timeit { ~/build/release/duped /data/data/com.termux $"-s=($strategy)" --silent }),
    ]
    {
        strategy: $strategy,
        count: $count,
        avg: (avg_dur $times),
        delta: (stddev_dur $times),
    }
}

[
    (bench_find),
    (bench "std-fs"),
    (bench "walkdir"),
    (bench "ignore"),
    (bench "jwalk"),
] | sort-by avg | table 
