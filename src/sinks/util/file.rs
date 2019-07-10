use crate::{
    event::Event,
    sinks::util::encoding::{self, BasicEncoding},
};

use bytes::Bytes;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use futures::{try_ready, Async, AsyncSink, Future, Poll, Sink, StartSend};
use tokio::codec::{BytesCodec, FramedWrite};
use tokio::fs::file::{File, OpenFuture};
use tokio::fs::OpenOptions;

use tracing::field;

pub type EmbeddedFileSink = Box<dyn Sink<SinkItem = Event, SinkError = ()> + 'static + Send>;

pub struct FileSink {
    pub path: PathBuf,
    state: FileSinkState,
}

enum FileSinkState {
    Disconnected,
    OpeningFile(OpenFuture<PathBuf>),
    FileProvided(FramedWrite<File, BytesCodec>),
}

impl FileSinkState {
    fn init(path: PathBuf) -> Self {
        debug!(message = "opening", file = ?path);
        let mut options = OpenOptions::new();
        options.create(true).append(true);

        FileSinkState::OpeningFile(options.open(path))
    }
}

impl FileSink {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path.clone(),
            state: FileSinkState::init(path),
        }
    }

    pub fn new_with_encoding(path: &Path, encoding: Option<BasicEncoding>) -> EmbeddedFileSink {
        let sink = FileSink::new(path.to_path_buf())
            .sink_map_err(|err| error!("Terminating the sink due to error: {}", err))
            .with(move |event| encoding::log_event_as_bytes_with_nl(event, &encoding));

        Box::new(sink)
    }

    pub fn poll_file(&mut self) -> Poll<&mut FramedWrite<File, BytesCodec>, io::Error> {
        loop {
            match self.state {
                FileSinkState::Disconnected => return Err(disconnected()),

                FileSinkState::FileProvided(ref mut sink) => return Ok(Async::Ready(sink)),

                FileSinkState::OpeningFile(ref mut open_future) => match open_future.poll() {
                    Ok(Async::Ready(file)) => {
                        debug!(message = "provided", file = ?file);
                        self.state =
                            FileSinkState::FileProvided(FramedWrite::new(file, BytesCodec::new()));
                    }
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(err) => {
                        self.state = FileSinkState::Disconnected;
                        return Err(err);
                    }
                },
            }
        }
    }
}

impl Sink for FileSink {
    type SinkItem = Bytes;
    type SinkError = io::Error;

    fn start_send(&mut self, line: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        match self.poll_file() {
            Ok(Async::Ready(file)) => {
                debug!(
                    message = "sending event",
                    bytes = &field::display(line.len())
                );
                match file.start_send(line) {
                    Ok(ok) => Ok(ok),

                    Err(err) => {
                        self.state = FileSinkState::Disconnected;
                        Err(err)
                    }
                }
            }
            Ok(Async::NotReady) => Ok(AsyncSink::NotReady(line)),
            Err(err) => Err(err),
        }
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        if let FileSinkState::Disconnected = self.state {
            return Err(disconnected());
        }

        let file = try_ready!(self.poll_file());

        match file.poll_complete() {
            Err(err) => {
                error!("Error while completing {:?}: {}", self.path, err);
                self.state = FileSinkState::Disconnected;
                Ok(Async::Ready(()))
            }
            Ok(ok) => Ok(ok),
        }
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        match self.poll_complete() {
            Ok(Async::Ready(())) => match self.state {
                FileSinkState::Disconnected => Ok(Async::Ready(())),

                FileSinkState::FileProvided(ref mut sink) => sink.close(),

                //this state is eliminated during poll_complete()
                FileSinkState::OpeningFile(_) => unreachable!(),
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => Err(err),
        }
    }
}

fn disconnected() -> io::Error {
    io::Error::new(ErrorKind::NotConnected, "FileSink is in disconnected state")
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        event::Event,
        sinks::util::encoding::BasicEncoding,
        test_util::{lines_from_file, random_lines_with_stream},
    };

    use futures::Stream;
    use tempfile::tempdir;

    #[test]
    fn text_output_is_correct() {
        let (input, events) = random_lines_with_stream(100, 16);
        let output = test_with_encoding(events, BasicEncoding::Text, None);

        for (input, output) in input.into_iter().zip(output) {
            assert_eq!(input, output);
        }
    }

    #[test]
    fn json_output_is_correct() {
        let (input, events) = random_lines_with_stream(100, 16);
        let output = test_with_encoding(events, BasicEncoding::Json, None);

        for (input, output) in input.into_iter().zip(output) {
            let output: serde_json::Value = serde_json::from_str(&output[..]).unwrap();
            let output = output.get("message").and_then(|v| v.as_str()).unwrap();
            assert_eq!(input, output);
        }
    }

    #[test]
    fn file_is_appended_not_truncated() {
        let directory = tempdir().unwrap().into_path();

        let (mut input1, events) = random_lines_with_stream(100, 16);
        test_with_encoding(events, BasicEncoding::Text, Some(directory.clone()));

        let (mut input2, events) = random_lines_with_stream(100, 16);
        let output = test_with_encoding(events, BasicEncoding::Text, Some(directory));

        let mut input = vec![];
        input.append(&mut input1);
        input.append(&mut input2);

        assert_eq!(output.len(), input.len());

        for (input, output) in input.into_iter().zip(output) {
            assert_eq!(input, output);
        }
    }

    fn test_with_encoding<S>(
        events: S,
        encoding: BasicEncoding,
        directory: Option<PathBuf>,
    ) -> Vec<String>
    where
        S: 'static + Stream<Item = Event, Error = ()> + Send,
    {
        let path = directory
            .unwrap_or(tempdir().unwrap().into_path())
            .join("test.out");

        let sink = FileSink::new_with_encoding(&path, Some(encoding));

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let pump = sink.send_all(events);
        let _ = rt.block_on(pump).unwrap();

        lines_from_file(&path)
    }
}
