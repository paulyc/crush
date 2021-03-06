use crate::lang::scope::Scope;
use crate::lang::errors::{CrushResult, argument_error, mandate};
use crate::lang::value::{Value, ValueType};
use crate::lang::execution_context::ExecutionContext;
use crate::lang::table::{ColumnType, Row};
use ordered_map::OrderedMap;
use crate::lang::command::OutputType::{Known, Unknown};

pub fn r#let(context: ExecutionContext) -> CrushResult<()> {
    for arg in context.arguments {
        context.env.declare(mandate(arg.argument_type, "Missing variable name")?.as_ref(), arg.value)?;
    }
    context.output.send(Value::Empty())
}

pub fn set(context: ExecutionContext) -> CrushResult<()> {
    for arg in context.arguments {
        context.env.set(mandate(arg.argument_type, "Missing variable name")?.as_ref(), arg.value)?;
    }
    context.output.send(Value::Empty())
}

pub fn unset(context: ExecutionContext) -> CrushResult<()> {
    for arg in context.arguments {
        if let Value::String(s) = &arg.value {
            if s.len() == 0 {
                return argument_error("Illegal variable name");
            } else {
                context.env.remove_str(s)?;
            }
        } else {
            return argument_error("Illegal variable name");
        }
    }
    context.output.send(Value::Empty())
}

pub fn r#use(context: ExecutionContext) -> CrushResult<()> {
    for arg in context.arguments.iter() {
        match (arg.argument_type.is_none(), &arg.value) {
            (true, Value::Scope(e)) => context.env.r#use(e),
            _ => return argument_error("Expected all arguments to be scopes"),
        }
    }
    context.output.send(Value::Empty())
}

pub fn env(context: ExecutionContext) -> CrushResult<()> {
    let output = context.output.initialize(vec![
        ColumnType::new("name", ValueType::String),
        ColumnType::new("type", ValueType::String),
    ])?;

    let mut values: OrderedMap<String, ValueType> = OrderedMap::new();
    context.env.dump(&mut values)?;

    let mut keys = values.keys().collect::<Vec<&String>>();
    keys.sort();

    for k in keys {
        context.printer.handle_error(output.send(Row::new(vec![
            Value::String(k.clone()),
            Value::String(values[k].to_string())
        ])));
    }

    Ok(())
}

pub fn declare(root: &Scope) -> CrushResult<()> {
    root.create_lazy_namespace(
        "var",
        Box::new(move |ns| {
            ns.declare_command(
                "let", r#let, false,
                "name := value", "Declare a new variable", None, Known(ValueType::Empty))?;
            ns.declare_command(
                "set", set, false,
                "name = value", "Assign a new value to an already existing variable", None, Known(ValueType::Empty))?;
            ns.declare_command(
                "unset", unset, false,
                "scope name:string",
                "Removes a variable from the namespace",
                None, Known(ValueType::Empty))?;
            ns.declare_command(
                "env", env, false,
                "env", "Returns a table containing the current namespace",
                Some(r#"    The columns of the table are the name, and the type of the value."#), Unknown)?;
            ns.declare_command(
                "use", r#use, false,
                "use scope:scope",
                "Puts the specified scope into the list of scopes to search in by default during scope lookups",
                Some(r#"    Example:

    use math
    sqrt 1.0"#), Known(ValueType::Empty))?;
            Ok(())
        }))?;
    Ok(())
}
