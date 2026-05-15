use criterion::{Criterion, criterion_group, criterion_main};
use etherpad_client::changeset::parser::parse;
use etherpad_client::changeset::serializer::serialize;
use etherpad_client::ot::apply;

fn bench_parse(c: &mut Criterion) {
    let wire = "Z:64>3=10+3=51$abc";
    c.bench_function("parse 3-op changeset", |b| {
        b.iter(|| parse(wire).unwrap());
    });
}

fn bench_apply_1k(c: &mut Criterion) {
    // 1000-char doc, insert "hello" at offset 500.
    // old_len = 1000 = base36 "rs", net = +5, keep 500 ("dw") + insert 5 + keep 500.
    let text = "x".repeat(1000);
    let wire = "Z:rs>5=dw+5=dw$hello".to_string();
    let cs = parse(&wire).unwrap();
    c.bench_function("apply to 1k doc", |b| {
        b.iter(|| apply(&cs, &text).unwrap());
    });
}

fn bench_roundtrip(c: &mut Criterion) {
    let wire = "Z:64>3=10+3=51$abc";
    let cs = parse(wire).unwrap();
    c.bench_function("parse+serialize roundtrip", |b| {
        b.iter(|| serialize(&parse(&serialize(&cs)).unwrap()));
    });
}

criterion_group!(benches, bench_parse, bench_apply_1k, bench_roundtrip);
criterion_main!(benches);
