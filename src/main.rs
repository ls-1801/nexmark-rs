// Copyright 2023 Singularity Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::{Parser, ValueEnum};
use csv::Writer;
use nexmark::event::{Event, EventType};
use nexmark::{BinaryBid, BinaryWriter, EventGenerator};
use std::fs;
use std::io::stdout;
use std::time::{Duration, Instant};

/// Nexmark event generator.
#[derive(Debug, Parser)]
pub struct Args {
    /// The type of events to generate.
    #[clap(short, long = "type", value_enum, default_value = "all")]
    type_: Type,

    /// The number of events to generate.
    /// If not specified, generate events forever.
    #[clap(short, long)]
    number: Option<usize>,

    /// The start event offset.
    #[clap(long, default_value = "0")]
    offset: u64,

    /// The step for each iteration.
    #[clap(long, default_value = "1")]
    step: u64,

    /// Print format.
    #[clap(long, value_enum, default_value = "json")]
    format: Format,

    /// Generate all events immediately.
    #[clap(long)]
    no_wait: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Type {
    All,
    Person,
    Auction,
    Bid,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    /// CSV
    CSV,
    /// JSON format.
    Json,
    /// Rust debug format.
    Rust,
    /// Binary
    Binary,
}

fn main() {
    let opts = Args::parse();
    let number = opts.number.unwrap_or(usize::MAX);

    let iter = EventGenerator::default()
        .with_offset(opts.offset)
        .with_step(opts.step);
    let iter = match opts.type_ {
        Type::All => iter,
        Type::Person => iter.with_type_filter(EventType::Person),
        Type::Auction => iter.with_type_filter(EventType::Auction),
        Type::Bid => iter.with_type_filter(EventType::Bid),
    };
    let start_time = Instant::now();
    let start_ts = iter.timestamp();
    let mut csv_writer = None;
    let mut binary_writer = None;
    let mut file = None;
    match opts.format {
        Format::CSV => csv_writer = Some(Writer::from_writer(stdout())),
        Format::Binary => {
            file = Some(fs::File::create("bid.bin").unwrap());
            binary_writer = Some(BinaryWriter::new(file.as_mut().unwrap(), 8192))
        }
        _ => {}
    };

    for event in iter.take(number) {
        if !opts.no_wait {
            let emit_time = start_time + Duration::from_millis(event.timestamp() - start_ts);
            // sleep until the timestamp of the event
            if let Some(t) = emit_time.checked_duration_since(Instant::now()) {
                std::thread::sleep(t);
            }
        }

        match opts.format {
            Format::CSV => csv_writer.as_mut().unwrap().serialize(&event).unwrap(),
            Format::Json => println!("{}", serde_json::to_string(&event).unwrap()),
            Format::Binary => binary_writer
                .as_mut()
                .unwrap()
                .write_buffer(&BinaryBid::from(event))
                .unwrap(),
            Format::Rust => match &event {
                Event::Person(e) => println!("{e:?}"),
                Event::Auction(e) => println!("{e:?}"),
                Event::Bid(e) => println!("{e:?}"),
            },
        }
    }
}
