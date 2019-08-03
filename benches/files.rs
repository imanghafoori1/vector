use bytes::Bytes;
use criterion::{criterion_group, Benchmark, Criterion, Throughput};
use futures::{sink::Sink, stream::Stream, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};
use std::iter;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::codec::{BytesCodec, FramedWrite};
use tokio::fs::OpenOptions;
use vector::test_util::{random_lines, send_lines};
use vector::{
    sinks, sources,
    topology::{self, config},
};

fn benchmark_files_without_partitions(c: &mut Criterion) {
    let num_lines: usize = 100_000;
    let line_size: usize = 100;

    let directory = tempdir().unwrap();
    let directory = directory.into_path();

    let bench = Benchmark::new("files_without_partitions", move |b| {
        b.iter_with_setup(
            || {
                let directory = directory.to_string_lossy();

                let mut input = directory.to_string();
                input.push_str("/test.in");

                let input = PathBuf::from(input);

                let mut output = directory.to_string();
                output.push_str("/test.out");

                let mut source = sources::file::FileConfig::default();
                source.include.push(input.clone());

                let mut config = config::Config::empty();
                config.add_source("in", source);
                config.add_sink(
                    "out",
                    &["in"],
                    sinks::file::FileSinkConfig {
                        path: output,
                        close_timeout_secs: None,
                        encoding: None,
                    },
                );

                let mut rt = tokio::runtime::Runtime::new().unwrap();
                let (topology, _crash) = topology::start(config, &mut rt, false).unwrap();

                let mut options = OpenOptions::new();
                options.create(true).write(true);

                let input = rt.block_on(options.open(input)).unwrap();
                let input =
                    FramedWrite::new(input, BytesCodec::new()).sink_map_err(|e| panic!("{:?}", e));

                (rt, topology, input)
            },
            |(mut rt, topology, input)| {
                let lines = random_lines(line_size).take(num_lines).map(|mut line| {
                    line.push('\n');
                    Bytes::from(line)
                });

                let lines = futures::stream::iter_ok::<_, ()>(lines);

                let pump = lines.forward(input);
                rt.block_on(pump).unwrap();

                rt.block_on(topology.stop()).unwrap();
                rt.shutdown_now().wait().unwrap();
            },
        )
    })
    .sample_size(10)
    .noise_threshold(0.05)
    .throughput(Throughput::Bytes((num_lines * line_size) as u32));

    c.bench("files", bench);
}

criterion_group!(files, benchmark_files_without_partitions);
