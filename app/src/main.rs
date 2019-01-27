
use std::env;
use std::io::{stdout, Write, BufWriter};
use std::process::exit;

use apng_encoder::apng::Delay;
use apng_encoder::apng::errors::{ApngResult, ErrorKind};
use apng_encoder::apng::encoder::Encoder;



#[derive(Debug, Default, Clone)]
struct EntryParameter {
    delay: Option<Delay>,
}

#[derive(Debug, Clone)]
struct Entry {
    filepath: String,
    parameter: EntryParameter,
}


fn main() {
    let entries = match parse_args() {
        Ok(entries) => entries,
        Err(message) => {
            eprintln!("{}", message);
            exit(1);
        },
    };

    let out = stdout();
    let mut out = BufWriter::new(out.lock());
    let mut encoder = Encoder::create(&mut out, meta).unwrap();

    for entry in entries {
    }
}


fn parse_args() -> ApngResult<Vec<Entry>> {
    let mut entries = vec![];

    let mut args = env::args();
    let _ = args.next().unwrap();
    let mut parameter = EntryParameter::default();

    while let Some(arg) = args.next() {
        let mut next = || args.next().ok_or(ErrorKind::NotEnoughArgument);

        match &*arg {
            "-d" | "--delay" => {
                 let value = next()?;
                parameter.delay = Some(parse_delay(&value)?)
            },
            filepath => {
                let entry = Entry {
                    filepath: filepath.to_owned(),
                    parameter: parameter.clone(),
                };
                entries.push(entry);
            }
        }
        println!("arg: {:?}", arg);
    }

    Ok(entries)
}


fn parse_delay(s: &str) -> ApngResult<Delay> {
    if let Some(div) = s.find('/') {
        let (numerator, denominator) = s.split_at(div);
        let numerator = numerator.parse()?;
        let denominator = denominator[1..].parse()?;
        return Ok(Delay { numerator, denominator });
    }

    let numerator = s.parse()?;
    Ok(Delay { numerator, denominator: 1000 })
}
