use std::collections::{HashMap, HashSet};
use std::fmt;
use crate::ast::{Stmt, Parameter};
use crate::numpy::NdArray;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
    Array(Vec<Value>),
    UniqueArray(Vec<Value>),  // Unique array (set-like)
    Dictionary(std::collections::HashMap<String, Value>),
    Collection(std::collections::HashSet<String>),
    Function {
        params: Vec<Parameter>,
        body: Vec<Stmt>,
    },
    ModuleFunction {
        params: Vec<Parameter>,
        body: Vec<Stmt>,
        closure_env: Environment,
    },
    Lambda {
        params: Vec<String>,
        body: crate::ast::LambdaBody,
        closure_env: Environment,
    },
    ModuleObject(std::collections::HashMap<String, Value>),
    BuiltinFunction(String),
    StructInstance {
        struct_name: String,
        fields: HashMap<String, Value>,
    },
    StructDefinition {
        name: String,
        fields: Vec<String>,
    },
    NdArray(NdArray),
    Error(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Value::UniqueArray(elements) => {
                write!(f, "{{|")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, "|}}")
            }
            Value::Dictionary(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
            Value::Collection(set) => {
                write!(f, "collection{{")?;
                let mut items: Vec<_> = set.iter().collect();
                items.sort(); // Sort for consistent display
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "}}")
            }
            Value::Function { params, .. } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Value::ModuleFunction { params, .. } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Value::Lambda { params, .. } => {
                write!(f, "lambda(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Value::ModuleObject(exports) => {
                write!(f, "module {{ ")?;
                for (i, (name, _)) in exports.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", name)?;
                }
                write!(f, " }}")
            }
            Value::BuiltinFunction(name) => write!(f, "<builtin: {}>", name),
            Value::StructInstance { struct_name, fields } => {
                write!(f, "{} {{", struct_name)?;
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            Value::StructDefinition { name, fields } => {
                write!(f, "struct {} {{", name)?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", field)?;
                }
                write!(f, "}}")
            }
            Value::NdArray(arr) => {
                write!(f, "array(")?;
                match arr {
                    crate::numpy::NdArray::F64(a) => {
                        for (i, val) in a.iter().enumerate() {
                            if i > 0 { write!(f, ", ")?; }
                            write!(f, "{}", val)?;
                        }
                    }
                    crate::numpy::NdArray::I64(a) => {
                        for (i, val) in a.iter().enumerate() {
                            if i > 0 { write!(f, ", ")?; }
                            write!(f, "{}", val)?;
                        }
                    }
                    crate::numpy::NdArray::Bool(a) => {
                        for (i, val) in a.iter().enumerate() {
                            if i > 0 { write!(f, ", ")?; }
                            write!(f, "{}", val)?;
                        }
                    }
                }
                write!(f, ")")
            }
            Value::Error(msg) => write!(f, "error: {}", msg),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match (self, other) {
            // Numbers can be compared
            (Value::Integer(a), Value::Integer(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Integer(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Integer(b)) => a.partial_cmp(&(*b as f64)),
            
            // Strings can be compared
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            
            // Bools can be compared
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            
            // Arrays can be compared lexicographically
            (Value::Array(a), Value::Array(b)) => a.partial_cmp(b),
            
            // For different types, use a consistent ordering
            (a, b) => {
                let type_order = |v: &Value| match v {
                    Value::Nil => 0,
                    Value::Bool(_) => 1,
                    Value::Integer(_) => 2,
                    Value::Float(_) => 3,
                    Value::String(_) => 4,
                    Value::Array(_) => 5,
                    Value::UniqueArray(_) => 6,
                    Value::Dictionary(_) => 7,
                    Value::Collection(_) => 8,
                    Value::Function { .. } => 9,
                    Value::ModuleFunction { .. } => 10,
                    Value::Lambda { .. } => 11,
                    Value::ModuleObject(_) => 12,
                    Value::BuiltinFunction(_) => 13,
                    Value::StructInstance { .. } => 14,
                    Value::StructDefinition { .. } => 15,
                    Value::Error(_) => 16,
                    Value::NdArray(_) => 17,
                };
                type_order(a).partial_cmp(&type_order(b))
            }
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            Value::Integer(0) => false,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Dictionary(dict) => !dict.is_empty(),
            Value::Collection(set) => !set.is_empty(),
            _ => true,
        }
    }
    
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Bool(_) => "boolean",
            Value::Nil => "nil",
            Value::Array(_) => "array",
            Value::UniqueArray(_) => "unique_array",
            Value::Dictionary(_) => "dictionary",
            Value::Collection(_) => "collection",
            Value::Function { .. } => "function",
            Value::ModuleFunction { .. } => "function",
            Value::Lambda { .. } => "lambda",
            Value::ModuleObject(_) => "module",
            Value::BuiltinFunction(_) => "builtin",
            Value::StructInstance { .. } => "struct",
            Value::StructDefinition { .. } => "struct_definition",
            Value::NdArray(_) => "ndarray",
            Value::Error(_) => "error",
        }
    }

    /// Display value for print() - strings without quotes at top level
    /// But strings inside collections keep their quotes (via to_string())
    pub fn display(&self) -> String {
        match self {
            Value::String(s) => s.clone(),  // No quotes for bare strings in print
            _ => self.to_string(),  // Use Display (with quotes) for everything else
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }
    
    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }
    
    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }
    
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }
    
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
}
