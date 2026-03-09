use std::fmt;
use crate::lexer::InterpolationPart;

#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    /// Simple variable: x = value
    Identifier(String),
    /// Array/dictionary index: arr[i] = value (object must be identifier)
    Index {
        object: String,
        index: Box<Expr>,
    },
    /// Struct/dictionary field: obj.field = value (object must be identifier)
    Field {
        object: String,
        field: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    String(String),
    InterpolatedString(Vec<InterpolationPart>),
    Bool(bool),
    Nil,
    Identifier(String),
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },
    Increment {
        target: Box<Expr>,
        prefix: bool,  // true for ++x, false for x++
    },
    Decrement {
        target: Box<Expr>,
        prefix: bool,  // true for --x, false for x--
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Argument>,
    },
    Array(Vec<Expr>),
    UniqueArray(Vec<Expr>),  // {|1, 2, 3|} - array with unique values only
    ListComprehension {
        expr: Box<Expr>,
        var: String,
        iterable: Box<Expr>,
        condition: Option<Box<Expr>>,
    },
    Generator {
        expr: Box<Expr>,
        var: String,
        iterable: Box<Expr>,
        condition: Option<Box<Expr>>,
    },
    Dictionary(Vec<(String, Expr)>),
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Slice {
        object: Box<Expr>,
        from: Option<Box<Expr>>,
        to: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
    },
    Member {
        object: Box<Expr>,
        property: String,
    },
    StructAccess {
        object: Box<Expr>,
        field: String,
    },
    StructInit {
        struct_name: String,
        fields: Vec<(String, Expr)>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
    },
    Lambda {
        params: Vec<String>,
        body: LambdaBody,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    ConditionalExpr {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        elseif_branches: Vec<ConditionalElseIfBranch>,
        else_expr: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    In,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaBody {
    Expression(Box<Expr>),
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElseIfBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalElseIfBranch {
    pub condition: Expr,
    pub then_expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub default_value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Positional(Expr),
    Keyword { name: String, value: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Expr),
    Identifier(String),
    Wildcard, // _
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Let {
        name: String,
        value: Expr,
    },
    Assign {
        target: AssignTarget,
        value: Expr,
    },
    Function {
        name: String,
        params: Vec<Parameter>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elseif_branches: Vec<ElseIfBranch>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Import {
        module_path: String,
        items: Option<Vec<String>>, // None for import all, Some(vec) for specific items
        alias: Option<String>,      // For "as" aliases
    },
    Export {
        name: String,
        value: Expr,
    },
    ExportFunction {
        name: String,
        params: Vec<Parameter>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Print {
        values: Vec<Expr>,
        sep: Option<Expr>,
        end: Option<Expr>,
    },
    Struct {
        name: String,
        fields: Vec<String>,
    },
    Try {
        body: Vec<Stmt>,
        catch_var: Option<String>,
        catch_body: Vec<Stmt>,
    },
    Throw(Expr),
    Break,
    Continue,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Integer(n) => write!(f, "{}", n),
            Expr::Float(n) => write!(f, "{}", n),
            Expr::String(s) => write!(f, "\"{}\"", s),
            Expr::InterpolatedString(parts) => {
                write!(f, "\"")?;
                for part in parts {
                    match part {
                        InterpolationPart::Text(text) => write!(f, "{}", text)?,
                        InterpolationPart::Expression(expr) => write!(f, "${{{}}}", expr)?,
                    }
                }
                write!(f, "\"")
            }
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Nil => write!(f, "nil"),
            Expr::Identifier(name) => write!(f, "{}", name),
            Expr::Binary { left, operator, right } => {
                write!(f, "({} {} {})", left, operator, right)
            }
            Expr::Unary { operator, operand } => {
                write!(f, "({}{})", operator, operand)
            }
            Expr::Increment { target, prefix } => {
                if *prefix {
                    write!(f, "++{}", target)
                } else {
                    write!(f, "{}++", target)
                }
            }
            Expr::Decrement { target, prefix } => {
                if *prefix {
                    write!(f, "--{}", target)
                } else {
                    write!(f, "{}--", target)
                }
            }
            Expr::Call { callee, args } => {
                write!(f, "{}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Expr::UniqueArray(elements) => {
                write!(f, "{{|")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, "|}}")
            }
            Expr::ListComprehension { expr, var, iterable, condition } => {
                write!(f, "[{} for {} in {}", expr, var, iterable)?;
                if let Some(cond) = condition {
                    write!(f, " if {}", cond)?;
                }
                write!(f, "]")
            }
            Expr::Generator { expr, var, iterable, condition } => {
                write!(f, "<{} for {} in {}", expr, var, iterable)?;
                if let Some(cond) = condition {
                    write!(f, " if {}", cond)?;
                }
                write!(f, ">")
            }
            Expr::Dictionary(pairs) => {
                write!(f, "{{")?;
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            Expr::Index { object, index } => {
                write!(f, "{}[{}]", object, index)
            }
            Expr::Slice { object, from, to, step } => {
                write!(f, "{}[", object)?;
                let mut parts = Vec::new();
                if let Some(from_expr) = from {
                    parts.push(format!("from:{}", from_expr));
                }
                if let Some(to_expr) = to {
                    parts.push(format!("to:{}", to_expr));
                }
                if let Some(step_expr) = step {
                    parts.push(format!("step:{}", step_expr));
                }
                write!(f, "{}", parts.join(" "))?;
                write!(f, "]")
            }
            Expr::Member { object, property } => {
                write!(f, "{}.{}", object, property)
            }
            Expr::StructAccess { object, field } => {
                write!(f, "{}.{}", object, field)
            }
            Expr::StructInit { struct_name, fields } => {
                write!(f, "new {} {{", struct_name)?;
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            Expr::Range { start, end, step } => {
                match step {
                    Some(step) => write!(f, "{}..{}:{}", start, end, step),
                    None => write!(f, "{}..{}", start, end),
                }
            }
            Expr::Lambda { params, body } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")?;
                match body {
                    LambdaBody::Expression(expr) => write!(f, " -> {}", expr),
                    LambdaBody::Block(_) => write!(f, " {{ ... }}"),
                }
            }
            Expr::Match { expr, arms } => {
                write!(f, "match {} {{ ", expr)?;
                for (i, arm) in arms.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{:?} => {:?}", arm.pattern, arm.body)?;
                }
                write!(f, " }}")
            }
            Expr::ConditionalExpr { condition, then_expr, elseif_branches, else_expr } => {
                write!(f, "if {} then {}", condition, then_expr)?;
                for branch in elseif_branches {
                    write!(f, " elseif {} then {}", branch.condition, branch.then_expr)?;
                }
                if let Some(else_expr) = else_expr {
                    write!(f, " else {}", else_expr)?;
                }
                write!(f, " end")
            }
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op = match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::Greater => ">",
            BinaryOp::LessEqual => "<=",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
            BinaryOp::In => "in",
        };
        write!(f, "{}", op)
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op = match self {
            UnaryOp::Negate => "-",
            UnaryOp::Not => "!",
        };
        write!(f, "{}", op)
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(default) = &self.default_value {
            write!(f, "{}={}", self.name, default)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Argument::Positional(expr) => write!(f, "{}", expr),
            Argument::Keyword { name, value } => write!(f, "{}={}", name, value),
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{}", expr),
            Stmt::Let { name, value } => write!(f, "let {} = {}", name, value),
            Stmt::Assign { target, value } => {
                match target {
                    AssignTarget::Identifier(name) => write!(f, "{} = {}", name, value),
                    AssignTarget::Index { object, index } => write!(f, "{}[{}] = {}", object, index, value),
                    AssignTarget::Field { object, field } => write!(f, "{}.{} = {}", object, field, value),
                }
            }
            Stmt::Function { name, params, .. } => {
                write!(f, "fn {}(", name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Stmt::If { condition, .. } => write!(f, "if {}", condition),
            Stmt::While { condition, .. } => write!(f, "while {}", condition),
            Stmt::For { var, iterable, .. } => write!(f, "for {} in {}", var, iterable),
            Stmt::Import { module_path, items, alias } => {
                write!(f, "import")?;
                if let Some(items) = items {
                    write!(f, " {{")?;
                    for (i, item) in items.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", item)?;
                    }
                    write!(f, "}} from")?;
                }
                write!(f, " \"{}\"", module_path)?;
                if let Some(alias) = alias {
                    write!(f, " as {}", alias)?;
                }
                Ok(())
            }
            Stmt::Export { name, value } => write!(f, "export {} = {}", name, value),
            Stmt::ExportFunction { name, params, .. } => {
                write!(f, "export fn {}(", name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Stmt::Return(Some(expr)) => write!(f, "return {}", expr),
            Stmt::Return(None) => write!(f, "return"),
            Stmt::Print { values, sep, end } => {
                write!(f, "print(")?;
                for (i, val) in values.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", val)?;
                }
                if sep.is_some() || end.is_some() {
                    if !values.is_empty() { write!(f, ", ")?; }
                    if let Some(s) = sep { write!(f, "sep={}, ", s)?; }
                    if let Some(e) = end { write!(f, "end={}", e)?; }
                }
                write!(f, ")")
            },
            Stmt::Struct { name, fields } => {
                write!(f, "struct {} {{", name)?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", field)?;
                }
                write!(f, "}}")
            }
            Stmt::Try { body: _, catch_var, catch_body: _ } => {
                write!(f, "try")?;
                if let Some(var) = catch_var {
                    write!(f, " catch {}", var)?;
                } else {
                    write!(f, " catch")?;
                }
                Ok(())
            }
            Stmt::Throw(expr) => write!(f, "throw {}", expr),
            Stmt::Break => write!(f, "break"),
            Stmt::Continue => write!(f, "continue"),
        }
    }
}
