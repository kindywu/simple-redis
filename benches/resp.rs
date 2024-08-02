use anyhow::Result;
use bytes::BytesMut;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simple_redis::RespFrame;

// resp frames covers all kinds of real-world redis requests and responses
// cmd 1: set key value
// cmd 1 response: OK
// cmd 2: get key
// cmd 2 response: value
// cmd 3: hset key field value
// cmd 3 response: ERR
// cmd 4: hget key field
// cmd 4 response: value
// cmd 5: sadd key member
// cmd 5 response: 1
// const DATA: &str = "*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*1\r\n+OK\r\n*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*4\r\n$4\r\nHSET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*1\r\n-ERR\r\n*3\r\n$4\r\nHGET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*3\r\n$4\r\nSADD\r\n$3\r\nkey\r\n$6\r\nmember\r\n:1\r\n";

const DATA: &str = "*3\r\n$4\r\necho\r\n$5\r\nhello\r\n+OK\r\n";

fn v1_decode(buf: &mut BytesMut) -> Result<Vec<RespFrame>> {
    use simple_redis::RespDecode;
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = RespFrame::decode(buf)?;
        frames.push(frame);
    }
    Ok(frames)
}

fn v2_decode(buf: &mut BytesMut) -> Result<Vec<RespFrame>> {
    use simple_redis::RespDecodeV2;
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = RespFrame::decode(buf)?;
        frames.push(frame);
    }
    Ok(frames)
}

fn v3_decode(buf: &mut BytesMut) -> Result<Vec<RespFrame>> {
    use simple_redis::RespDecodeV3;
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = RespFrame::decode(buf)?;
        frames.push(frame);
    }
    Ok(frames)
}

fn criterion_benchmark(c: &mut Criterion) {
    let buf = BytesMut::from(DATA);

    c.bench_function("v1_decode", |b| {
        b.iter(|| black_box(v1_decode(&mut buf.clone())))
    });

    c.bench_function("v2_decode", |b| {
        b.iter(|| black_box(v2_decode(&mut buf.clone())))
    });

    c.bench_function("v3_decode", |b| {
        b.iter(|| black_box(v3_decode(&mut buf.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
