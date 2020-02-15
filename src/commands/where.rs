use std::cmp::Ordering;

use crate::{
    data::{
        Value,
        Row,
    },
    stream::{OutputStream}
};
use crate::commands::CompileContext;
use crate::errors::{error, CrushResult, argument_error};
use crate::printer::Printer;
use crate::stream::{Readable, empty_channel, channels};
use crate::data::{RowsReader, ColumnType, Argument};
use crate::closure::Closure;
use crate::stream_printer::spawn_print_thread;
use crate::env::Env;

pub struct Config<T: Readable> {
    condition: Closure,
    input: T,
    output: OutputStream,
}

fn evaluate(condition: &Closure, row: &Row, input_type: &Vec<ColumnType>, env: &Env, printer: &Printer) -> CrushResult<bool> {
    let arguments = row.clone().into_vec()
        .drain(..)
        .zip(input_type.iter())
        .map(|(c, t)| {
            match &t.name {
                None => Argument::unnamed(c.clone()),
                Some(name) => Argument::named(name.as_ref(), c),
            }
        })
        .collect();

    let (sender, reciever) = channels();

    condition.spawn_and_execute(CompileContext{
        input: empty_channel(),
        output: sender,
        arguments,
        env: env.clone(),
        printer: printer.clone(),
    });

    match reciever.recv()? {
        Value::Bool(b) => Ok(b),
        _ => error("Expected a boolean result")
    }
}

pub fn run<T: Readable>(mut config: Config<T>, env: Env, printer: Printer) -> CrushResult<()> {
    loop {
        match config.input.read() {
            Ok(row) => {
                match evaluate(&config.condition, &row, config.input.get_type(), &env, &printer) {
                    Ok(val) => if val { if config.output.send(row).is_err() { break }},
                    Err(e) => printer.job_error(e),
                }
            }
            Err(_) => break,
        }
    }
    return Ok(());
}

pub fn parse(_input_type: &Vec<ColumnType>,
             arguments: &mut Vec<Argument>) -> CrushResult<Closure> {
    match arguments.remove(0).value {
        Value::Closure(c) => Ok(c),
        _ => argument_error("Expected a closure"),
    }
}

pub fn perform(mut context: CompileContext) -> CrushResult<()> {
    match context.input.recv()? {
        Value::Stream(input) => {
            let output = context.output.initialize(input.stream.get_type().clone())?;
            let config = Config {
                condition: parse(input.stream.get_type(), context.arguments.as_mut())?,
                input: input.stream,
                output: output,
            };
            run(config, context.env, context.printer)
        }
        Value::Rows(r) => {
            let input = RowsReader::new(r);
            let output = context.output.initialize(input.get_type().clone())?;
            let config = Config {
                condition: parse(input.get_type(), context.arguments.as_mut())?,
                input: input,
                output: output,
            };
            run(config, context.env, context.printer)
        }
        _ => error("Expected a stream"),
    }
}