use crate::ast::{BlockStatement, Identifier};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Null,
    ReturnValue(Box<Object>),
    Error(String),
    Function(Vec<Identifier>, BlockStatement, Rc<RefCell<Environment>>),
    String(String),
    Array(Vec<Object>),
    Class(Rc<RefCell<Class>>),
    ClassInstance(Rc<RefCell<ClassInstance>>),
    Struct(Rc<RefCell<Struct>>),
    StructInstance(Rc<RefCell<StructInstance>>),
    Interface(Rc<RefCell<Interface>>),
    Method(Rc<RefCell<Method>>),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Integer(value) => write!(f, "{}", value),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::Null => write!(f, "null"),
            Object::ReturnValue(value) => write!(f, "{}", value),
            Object::Error(message) => write!(f, "ERROR: {}", message),
            Object::Function(parameters, body, _) => {
                let params: Vec<String> = parameters.iter().map(|p| p.value.clone()).collect();
                write!(f, "fn({}) {{\n{}\n}}", params.join(", "), body)
            }
            Object::String(value) => write!(f, "{}", value),
            Object::Array(elements) => {
                let elements: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            Object::Class(c) => write!(f, "class {}", c.borrow().name),
            Object::ClassInstance(i) => write!(f, "instance of {}", i.borrow().class.borrow().name),
            Object::Struct(s) => write!(f, "struct {}", s.borrow().name),
            Object::StructInstance(i) => write!(
                f,
                "instance of struct {}",
                i.borrow().struct_def.borrow().name
            ),
            Object::Interface(i) => write!(f, "interface {}", i.borrow().name),
            Object::Method(m) => write!(f, "method {}", m.borrow().name),
        }
    }
}

#[allow(dead_code)]
const INTEGER: &str = "INTEGER";
#[allow(dead_code)]
const BOOLEAN: &str = "BOOLEAN";
#[allow(dead_code)]
const NULL: &str = "NULL";
#[allow(dead_code)]
const RETURN_VALUE: &str = "RETURN_VALUE";
#[allow(dead_code)]
const ERROR: &str = "ERROR";
#[allow(dead_code)]
const FUNCTION: &str = "FUNCTION";
#[allow(dead_code)]
const STRING: &str = "STRING";
#[allow(dead_code)]
const ARRAY: &str = "ARRAY";
#[allow(dead_code)]
const CLASS: &str = "CLASS";
#[allow(dead_code)]
const CLASS_INSTANCE: &str = "CLASS_INSTANCE";
#[allow(dead_code)]
const STRUCT: &str = "STRUCT";
#[allow(dead_code)]
const STRUCT_INSTANCE: &str = "STRUCT_INSTANCE";
#[allow(dead_code)]
const INTERFACE: &str = "INTERFACE";
#[allow(dead_code)]
const METHOD: &str = "METHOD";

impl Object {
    pub fn type_str(&self) -> &str {
        match self {
            Object::Integer(_) => INTEGER,
            Object::Boolean(_) => BOOLEAN,
            Object::Null => NULL,
            Object::ReturnValue(_) => RETURN_VALUE,
            Object::Error(_) => ERROR,
            Object::Function(_, _, _) => FUNCTION,
            Object::String(_) => STRING,
            Object::Array(_) => ARRAY,
            Object::Class(_) => "CLASS",
            Object::ClassInstance(_) => "CLASS_INSTANCE",
            Object::Struct(_) => "STRUCT",
            Object::StructInstance(_) => "STRUCT_INSTANCE",
            Object::Interface(_) => "INTERFACE",
            Object::Method(_) => "METHOD",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Class {
    pub name: String,
    pub super_class: Option<Rc<RefCell<Class>>>,
    pub interfaces: Vec<Rc<RefCell<Interface>>>,
    pub properties: HashMap<String, Object>,
    pub methods: HashMap<String, Rc<RefCell<Method>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassInstance {
    pub class: Rc<RefCell<Class>>,
    pub fields: HashMap<String, Object>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Struct {
    pub name: String,
    pub properties: HashMap<String, Object>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructInstance {
    pub struct_def: Rc<RefCell<Struct>>,
    pub fields: HashMap<String, Object>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interface {
    pub name: String,
    pub method_signatures: HashMap<String, MethodSignature>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method {
    pub name: String,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
    pub env: Rc<RefCell<Environment>>,
    pub this: Option<Rc<RefCell<ClassInstance>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<Identifier>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Self {
        let mut env = Environment::new();
        env.outer = Some(outer);
        env
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(obj) => Some(obj.clone()),
            None => self.outer.as_ref().and_then(|o| o.borrow().get(name)),
        }
    }

    pub fn set(&mut self, name: String, val: Object) {
        self.store.insert(name, val);
    }
}
