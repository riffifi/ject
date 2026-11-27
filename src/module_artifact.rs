//! Decoder for the source-free module artifacts produced by the Ject bootstrap.
use crate::{ast::*, lexer::InterpolationPart};
use serde_json::Value;
use std::{collections::BTreeMap, io::Read};

type Decoded<T> = Result<T, String>;

pub enum Artifact {
    Module(Vec<Stmt>),
    Program(Program),
}

pub struct Program {
    pub(crate) entry: String,
    pub(crate) modules: BTreeMap<String, CompiledModule>,
}

impl Program {
    pub fn requires_native_project(&self) -> bool {
        self.modules.values().any(|module| module.native.is_some())
    }

    pub fn validate_native_bindings(
        &self,
        checksums: &std::collections::HashMap<String, String>,
    ) -> Result<(), String> {
        for module in self.modules.values() {
            if let Some((backend, expected)) = &module.native {
                match checksums.get(backend) {
                    Some(actual) if actual == expected => {}
                    Some(_) => {
                        return Err(format!(
                            "native package {backend:?} differs from the compiled package"
                        ))
                    }
                    None => {
                        return Err(format!(
                            "native package {backend:?} is absent from the authorized project"
                        ))
                    }
                }
            }
        }
        Ok(())
    }
}

pub(crate) struct CompiledModule {
    pub(crate) statements: Vec<Stmt>,
    pub(crate) stem: Option<String>,
    pub(crate) native: Option<(String, String)>,
    pub(crate) imports: BTreeMap<String, String>,
}

pub fn decode(reader: impl Read) -> Decoded<Artifact> {
    let artifact: Value = serde_json::from_reader(reader).map_err(|error| error.to_string())?;
    if artifact["version"] != 1 {
        return Err("expected artifact version 1".into());
    }
    match artifact["format"].as_str() {
        Some("ject-module-ast") => list(&artifact["statements"], statement).map(Artifact::Module),
        Some("ject-program-ast") => decode_program(&artifact).map(Artifact::Program),
        _ => Err("unknown Ject artifact format".into()),
    }
}

fn decode_program(artifact: &Value) -> Decoded<Program> {
    let entry = text(&artifact["entry"])?;
    let modules = artifact["modules"]
        .as_object()
        .ok_or("expected module table")?
        .iter()
        .map(|(name, module)| {
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
            {
                return Err(format!("invalid compiled module identifier {name:?}"));
            }
            let imports = module["imports"]
                .as_object()
                .ok_or("expected import table")?
                .iter()
                .map(|(specifier, destination)| Ok((specifier.clone(), text(destination)?)))
                .collect::<Decoded<BTreeMap<_, _>>>()?;
            Ok((
                name.clone(),
                CompiledModule {
                    statements: list(&module["statements"], statement)?,
                    stem: optional(&module["stem"], text)?,
                    native: optional(&module["native"], |native| {
                        Ok((text(&native["backend"])?, text(&native["checksum"])?))
                    })?,
                    imports,
                },
            ))
        })
        .collect::<Decoded<BTreeMap<_, _>>>()?;
    if !modules.contains_key(&entry) {
        return Err(format!("missing entry module {entry:?}"));
    }
    for (name, module) in &modules {
        for (specifier, destination) in &module.imports {
            if !modules.contains_key(destination) {
                return Err(format!(
                    "module {name:?} maps {specifier:?} to missing module {destination:?}"
                ));
            }
        }
    }
    Ok(Program { entry, modules })
}

fn kind(node: &Value) -> Decoded<&str> {
    node["kind"]
        .as_str()
        .ok_or_else(|| "missing node kind".into())
}

fn text(value: &Value) -> Decoded<String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "expected text".into())
}

fn boolean(value: &Value) -> Decoded<bool> {
    value.as_bool().ok_or_else(|| "expected boolean".into())
}

fn list<T>(value: &Value, read: impl Fn(&Value) -> Decoded<T>) -> Decoded<Vec<T>> {
    value
        .as_array()
        .ok_or("expected array")?
        .iter()
        .map(read)
        .collect()
}

fn optional<T>(value: &Value, read: impl FnOnce(&Value) -> Decoded<T>) -> Decoded<Option<T>> {
    if value.is_null() {
        Ok(None)
    } else {
        read(value).map(Some)
    }
}

fn boxed(value: &Value) -> Decoded<Box<Expr>> {
    expression(value).map(Box::new)
}

fn operator(value: &Value) -> Decoded<BinaryOp> {
    Ok(match text(value)?.as_str() {
        "Add" => BinaryOp::Add,
        "Subtract" => BinaryOp::Subtract,
        "Multiply" => BinaryOp::Multiply,
        "Divide" => BinaryOp::Divide,
        "Modulo" => BinaryOp::Modulo,
        "Equal" => BinaryOp::Equal,
        "NotEqual" => BinaryOp::NotEqual,
        "Less" => BinaryOp::Less,
        "LessEqual" => BinaryOp::LessEqual,
        "Greater" => BinaryOp::Greater,
        "GreaterEqual" => BinaryOp::GreaterEqual,
        "And" => BinaryOp::And,
        "Or" => BinaryOp::Or,
        "In" => BinaryOp::In,
        name => return Err(format!("unknown binary operator {name}")),
    })
}

fn fields(value: &Value) -> Decoded<Vec<(String, Expr)>> {
    list(value, |field| {
        Ok((text(&field["key"])?, expression(&field["value"])?))
    })
}

fn span_component(value: &Value) -> Decoded<usize> {
    value
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| "expected positive position component".into())
}

fn expression(node: &Value) -> Decoded<Expr> {
    Ok(match kind(node)? {
        "Located" => Expr::Located {
            expression: boxed(&node["value"])?,
            span: crate::diagnostic::SourceSpan::new(
                span_component(&node["line"])?,
                span_component(&node["column"])?,
                span_component(&node["length"])?,
            ),
        },
        "Integer" => Expr::Integer(node["value"].as_i64().ok_or("expected i64")?),
        "Float" => Expr::Float(node["value"].as_f64().ok_or("expected float")?),
        "String" => Expr::String(text(&node["value"])?),
        "Identifier" => Expr::Identifier(text(&node["value"])?),
        "Bool" => Expr::Bool(boolean(&node["value"])?),
        "Nil" => Expr::Nil,
        "CompiledInterpolation" => Expr::CompiledInterpolation(list(&node["value"], expression)?),
        "InterpolatedString" => Expr::InterpolatedString(list(&node["value"], |part| {
            let content = text(&part["text"])?;
            match kind(part)? {
                "Text" => Ok(InterpolationPart::Text(content)),
                "Expression" => Ok(InterpolationPart::Expression(content)),
                name => Err(format!("unknown interpolation part {name}")),
            }
        })?),
        "Binary" => Expr::Binary {
            operator: operator(&node["operator"])?,
            left: boxed(&node["left"])?,
            right: boxed(&node["right"])?,
        },
        "Unary" => Expr::Unary {
            operator: match text(&node["operator"])?.as_str() {
                "Not" => UnaryOp::Not,
                "Negate" => UnaryOp::Negate,
                name => return Err(format!("unknown unary operator {name}")),
            },
            operand: boxed(&node["operand"])?,
        },
        "Increment" | "Decrement" => {
            let target = boxed(&node["target"])?;
            let prefix = boolean(&node["prefix"])?;
            if kind(node)? == "Increment" {
                Expr::Increment { target, prefix }
            } else {
                Expr::Decrement { target, prefix }
            }
        }
        "Call" => Expr::Call {
            callee: boxed(&node["callee"])?,
            args: list(&node["args"], |arg| {
                let value = expression(&arg["value"])?;
                match kind(arg)? {
                    "Positional" => Ok(Argument::Positional(value)),
                    "Keyword" => Ok(Argument::Keyword {
                        name: text(&arg["name"])?,
                        value,
                    }),
                    name => Err(format!("unknown argument kind {name}")),
                }
            })?,
        },
        "Array" => Expr::Array(list(&node["items"], expression)?),
        "UniqueArray" => Expr::UniqueArray(list(&node["items"], expression)?),
        "Dictionary" => Expr::Dictionary(fields(&node["fields"])?),
        "StructInit" => Expr::StructInit {
            struct_name: text(&node["struct_name"])?,
            fields: fields(&node["fields"])?,
        },
        "StructAccess" => Expr::StructAccess {
            object: boxed(&node["object"])?,
            field: text(&node["field"])?,
        },
        "Index" => Expr::Index {
            object: boxed(&node["object"])?,
            index: boxed(&node["index"])?,
        },
        "Slice" => Expr::Slice {
            object: boxed(&node["object"])?,
            from: optional(&node["from"], boxed)?,
            to: optional(&node["to"], boxed)?,
            step: optional(&node["step"], boxed)?,
        },
        "Range" => Expr::Range {
            start: boxed(&node["start"])?,
            end: boxed(&node["finish"])?,
            step: optional(&node["step"], boxed)?,
        },
        "ListComprehension" => Expr::ListComprehension {
            expr: boxed(&node["expr"])?,
            var: text(&node["var"])?,
            iterable: boxed(&node["iterable"])?,
            condition: optional(&node["condition"], boxed)?,
        },
        "Lambda" => Expr::Lambda {
            params: list(&node["params"], text)?,
            body: match kind(&node["body"])? {
                "Expression" => LambdaBody::Expression(boxed(&node["body"]["value"])?),
                "Block" => LambdaBody::Block(list(&node["body"]["statements"], statement)?),
                name => return Err(format!("unknown lambda body {name}")),
            },
        },
        "ConditionalExpr" => Expr::ConditionalExpr {
            condition: boxed(&node["condition"])?,
            then_expr: boxed(&node["then_expr"])?,
            elseif_branches: list(&node["elseif_branches"], |branch| {
                Ok(ConditionalElseIfBranch {
                    condition: expression(&branch["condition"])?,
                    then_expr: expression(&branch["then_expr"])?,
                })
            })?,
            else_expr: optional(&node["else_expr"], boxed)?,
        },
        "Match" => Expr::Match {
            expr: boxed(&node["expr"])?,
            arms: list(&node["arms"], match_arm)?,
        },
        "Print" => Expr::Print {
            values: list(&node["values"], expression)?,
            sep: optional(&node["sep"], boxed)?,
            end: optional(&node["end"], boxed)?,
        },
        name => return Err(format!("unknown expression kind {name}")),
    })
}

fn match_arm(node: &Value) -> Decoded<MatchArm> {
    Ok(MatchArm {
        patterns: list(&node["patterns"], |pattern| {
            Ok(match kind(pattern)? {
                "Wildcard" => Pattern::Wildcard,
                "Identifier" => Pattern::Identifier(text(&pattern["name"])?),
                "Literal" => Pattern::Literal(expression(&pattern["value"])?),
                "Range" => Pattern::Range(
                    expression(&pattern["start"])?,
                    expression(&pattern["finish"])?,
                ),
                "Relational" => Pattern::Relational(
                    operator(&pattern["operator"])?,
                    expression(&pattern["value"])?,
                ),
                name => return Err(format!("unknown pattern kind {name}")),
            })
        })?,
        guard: optional(&node["guard"], expression)?,
        body: match kind(&node["body"])? {
            "Expression" => MatchArmBody::Expression(expression(&node["body"]["value"])?),
            "Block" => MatchArmBody::Block(list(&node["body"]["statements"], statement)?),
            name => return Err(format!("unknown match body {name}")),
        },
    })
}

fn target(node: &Value) -> Decoded<AssignTarget> {
    Ok(match kind(node)? {
        "Identifier" => AssignTarget::Identifier(text(&node["name"])?),
        "Field" => AssignTarget::Field {
            object: text(&node["object"])?,
            field: text(&node["field"])?,
        },
        "Index" => AssignTarget::Index {
            object: text(&node["object"])?,
            index: boxed(&node["index"])?,
        },
        "IndexChain" => AssignTarget::IndexChain {
            object: text(&node["object"])?,
            indices: list(&node["indices"], expression)?,
        },
        name => return Err(format!("unknown assignment target {name}")),
    })
}

fn statement(node: &Value) -> Decoded<Stmt> {
    let body = |field: &str| list(&node[field], statement);
    let value = || expression(&node["value"]);
    let name = || text(&node["name"]);
    Ok(match kind(node)? {
        "Expression" => Stmt::Expression(value()?),
        "Let" => Stmt::Let {
            name: name()?,
            value: value()?,
        },
        "Const" => Stmt::Const {
            name: name()?,
            value: value()?,
        },
        "Export" => Stmt::Export {
            name: name()?,
            value: value()?,
            mutable: boolean(&node["mutable"])?,
        },
        "Assign" => Stmt::Assign {
            target: target(&node["target"])?,
            value: value()?,
        },
        "Function" | "ExportFunction" => {
            let name = name()?;
            let body = body("body")?;
            let params = list(&node["params"], |param| {
                Ok(Parameter {
                    name: text(&param["name"])?,
                    default_value: optional(&param["default_value"], expression)?,
                })
            })?;
            if kind(node)? == "Function" {
                Stmt::Function { name, params, body }
            } else {
                Stmt::ExportFunction { name, params, body }
            }
        }
        "Import" => Stmt::Import {
            module_path: text(&node["module_path"])?,
            items: optional(&node["items"], |items| list(items, text))?,
            alias: optional(&node["alias"], text)?,
        },
        "If" => Stmt::If {
            condition: expression(&node["condition"])?,
            then_branch: body("then_branch")?,
            elseif_branches: list(&node["elseif_branches"], |branch| {
                Ok(ElseIfBranch {
                    condition: expression(&branch["condition"])?,
                    body: list(&branch["body"], statement)?,
                })
            })?,
            else_branch: optional(&node["else_branch"], |branch| list(branch, statement))?,
        },
        "While" => Stmt::While {
            condition: expression(&node["condition"])?,
            body: body("body")?,
        },
        "For" => Stmt::For {
            var: text(&node["var"])?,
            iterable: expression(&node["iterable"])?,
            body: body("body")?,
        },
        "Return" => Stmt::Return(optional(&node["value"], expression)?),
        "Print" => Stmt::Print {
            values: list(&node["values"], expression)?,
            sep: optional(&node["sep"], expression)?,
            end: optional(&node["end"], expression)?,
        },
        "Struct" => Stmt::Struct {
            name: name()?,
            fields: list(&node["fields"], text)?,
        },
        "Try" => Stmt::Try {
            body: body("body")?,
            catch_var: optional(&node["catch_var"], text)?,
            catch_body: body("catch_body")?,
        },
        "Throw" => Stmt::Throw(value()?),
        "Break" => Stmt::Break,
        "Continue" => Stmt::Continue,
        name => return Err(format!("unknown statement kind {name}")),
    })
}
