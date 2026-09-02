use crate::ast::{Parameter, Stmt};
use crate::native::NativeValue;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
    Array(std::rc::Rc<std::cell::RefCell<Vec<Value>>>),
    UniqueArray(Vec<Value>), // Unique array (set-like)
    Dictionary(std::rc::Rc<std::cell::RefCell<HashMap<String, Value>>>),
    Collection(std::collections::HashSet<String>),
    Function {
        name: String,
        params: Vec<Parameter>,
        body: Vec<Stmt>,
        closure_env: Environment,
    },
    ModuleFunction {
        name: String,
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
    /// A function provided by a native extension module (e.g. jnum), called
    /// through that module's `NativeModule::call`. Replaces the old approach of
    /// sniffing a `"np_"`/`"gui_"` prefix on a plain `BuiltinFunction` name.
    NativeFunction {
        module: String,
        name: String,
    },
    StructInstance {
        struct_name: String,
        fields: HashMap<String, Value>,
    },
    StructDefinition {
        name: String,
        fields: Vec<String>,
    },
    /// A value backed by native (Rust) code from a native extension module, e.g.
    /// jnum's `NdArray`. See `crate::native`.
    Native(NativeValue),
    Error(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => {
                if n.is_infinite() || n.is_nan() {
                    write!(f, "{}", n)
                } else if n.fract() == 0.0 && n.abs() < 1e15 {
                    // A whole-number float still needs to look like a float --
                    // otherwise there's no way to tell 5.0 (Float) from 5 (Integer)
                    // just by looking at printed output.
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Array(elements) => {
                let elements = elements.borrow();
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Value::UniqueArray(elements) => {
                write!(f, "{{|")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "|}}")
            }
            Value::Dictionary(map) => {
                let map = map.borrow();
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
            Value::Collection(set) => {
                write!(f, "collection{{")?;
                let mut items: Vec<_> = set.iter().collect();
                items.sort(); // Sort for consistent display
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "}}")
            }
            Value::Function { params, .. } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Value::ModuleFunction { params, .. } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Value::Lambda { params, .. } => {
                write!(f, "lambda(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Value::ModuleObject(exports) => {
                write!(f, "module {{ ")?;
                for (i, (name, _)) in exports.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", name)?;
                }
                write!(f, " }}")
            }
            Value::BuiltinFunction(name) => write!(f, "<builtin: {}>", name),
            Value::StructInstance {
                struct_name,
                fields,
            } => {
                write!(f, "{} {{", struct_name)?;
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            Value::StructDefinition { name, fields } => {
                write!(f, "struct {} {{", name)?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", field)?;
                }
                write!(f, "}}")
            }
            Value::Native(native) => write!(f, "{}", native.0.display()),
            Value::NativeFunction { module, name } => write!(f, "<native {}::{}>", module, name),
            Value::Error(msg) => write!(f, "error: {}", msg),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
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
            (Value::Array(a), Value::Array(b)) => a.borrow().partial_cmp(&*b.borrow()),

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
                    Value::Native(_) => 17,
                    Value::NativeFunction { .. } => 18,
                };
                type_order(a).partial_cmp(&type_order(b))
            }
        }
    }
}

impl Value {
    /// Wraps a `Vec<Value>` as an `Array`. Prefer this over
    /// `Value::Array(Rc::new(RefCell::new(v)))` directly -- same result, less noise.
    pub fn array(v: Vec<Value>) -> Value {
        Value::Array(std::rc::Rc::new(std::cell::RefCell::new(v)))
    }

    /// Wraps a dictionary in shared storage so cloning a `Value` remains O(1).
    pub fn dictionary(values: HashMap<String, Value>) -> Value {
        Value::Dictionary(std::rc::Rc::new(std::cell::RefCell::new(values)))
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            Value::Integer(0) => false,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.borrow().is_empty(),
            Value::UniqueArray(arr) => !arr.is_empty(),
            Value::Dictionary(dict) => !dict.borrow().is_empty(),
            Value::Collection(set) => !set.is_empty(),
            _ => true,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            Value::Integer(_) => "integer",
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
            Value::Native(native) => native.0.type_name(),
            Value::NativeFunction { .. } => "builtin",
            Value::Error(_) => "error",
        }
    }

    /// Display value for print() - strings without quotes at top level
    /// But strings inside collections keep their quotes (via to_string())
    pub fn display(&self) -> String {
        match self {
            Value::String(s) => s.clone(), // No quotes for bare strings in print
            _ => self.to_string(),         // Use Display (with quotes) for everything else
        }
    }
}

/// A single lexical scope: a mutable name table shared by reference. Using `Rc<RefCell<_>>`
/// means cloning an `Environment` (e.g. to capture a closure) is O(depth) — it just clones
/// scope pointers — instead of deep-copying every variable in every enclosing scope. It also
/// gives real closure semantics for free: a captured scope stays alive (and mutations to it
/// stay visible to the closure) via the shared reference, even after the scope that declared
/// it has been popped off the live call stack.
pub type Scope = std::rc::Rc<std::cell::RefCell<HashMap<String, Value>>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    scopes: Vec<Scope>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![std::rc::Rc::new(std::cell::RefCell::new(HashMap::new()))],
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last() {
            scope.borrow_mut().insert(name, value);
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.borrow().get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.borrow().contains_key(name) {
                scope.borrow_mut().insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    pub fn push_scope(&mut self) {
        self.scopes
            .push(std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())));
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
}
