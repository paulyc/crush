use crate::stream::{OutputStream, InputStream};
use crate::cell::{Argument, CellType, Cell, Row};
use crate::commands::{Call, Exec};
use crate::errors::{JobError, argument_error};
use std::iter::Iterator;

pub fn get_line_count(arguments: &Vec<Argument>) -> Result<i128, JobError> {
    return match arguments.len() {
        0 => Ok(10),
        1 => match arguments[0].cell {
            Cell::Integer(v) => Ok(v),
            _ => Err(argument_error("Expected a number"))
        }
        _ => Err(argument_error("Too many arguments"))
    }
}

fn run(
    input_type: Vec<CellType>,
    arguments: Vec<Argument>,
    input: InputStream,
    output: OutputStream) -> Result<(), JobError> {
    let mut count = 0;
    let mut tot = get_line_count(&arguments)?;
    loop {
        match input.recv() {
            Ok(row) => {
                if count >= tot {
                    break;
                }
                output.send(row)?;
                count+=1;
            }
            Err(_) => break,
        }
    }
    return Ok(());
}

pub fn head(input_type: Vec<CellType>, arguments: Vec<Argument>) -> Result<Call, JobError> {
    get_line_count(&arguments)?;
    return Ok(Call {
        name: String::from("head"),
        output_type: input_type.clone(),
        input_type,
        arguments: arguments,
        exec: Exec::Run(run),
    });
}