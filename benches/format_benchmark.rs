use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nginxfmt::{format_str, Config};

fn large_config() -> String {
    let mut config = String::from(
        "worker_processes auto;\n\nevents {\n    worker_connections 1024;\n}\n\nhttp {\n",
    );

    for server in 0..50 {
        config.push_str(&format!(
            "    server {{\n        listen {port};\n",
            port = 8000 + server
        ));
        for location in 0..20 {
            config.push_str(&format!(
                "        location /app{location} {{\n            proxy_pass http://backend{server};\n            proxy_set_header Host $host;\n        }}\n"
            ));
        }
        config.push_str("    }\n");
    }

    config.push_str("}\n");
    config
}

fn bench_format_large_config(c: &mut Criterion) {
    let input = large_config();
    let cfg = Config::default();

    c.bench_function("format_large_config", |b| {
        b.iter(|| format_str(black_box(&input), black_box(&cfg)).unwrap());
    });
}

criterion_group!(benches, bench_format_large_config);
criterion_main!(benches);
