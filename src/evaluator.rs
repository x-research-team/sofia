use crate::ast::{
    BlockStatement, ClassDeclaration, Expression, Identifier, IfExpression, InterfaceDeclaration,
    MethodCallExpression, NewExpression, Node, Program, PropertyAccessExpression, Statement,
    StructDeclaration, ThisExpression,
};
use crate::object::{
    Class, ClassInstance, Environment, Interface, Method, Object, Struct, StructInstance,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Object {
    match node {
        Node::Program(p) => eval_program(p, env),
        Node::Statement(s) => eval_statement(s, env),
        Node::Expression(e) => eval_expression(e, env),
    }
}

fn eval_program(program: Program, env: Rc<RefCell<Environment>>) -> Object {
    let mut result = Object::Null;
    for statement in program.statements {
        result = eval_statement(statement, Rc::clone(&env));
        match result {
            Object::ReturnValue(value) => return *value,
            Object::Error(_) => return result,
            _ => {}
        }
    }
    result
}

fn eval_statement(statement: Statement, env: Rc<RefCell<Environment>>) -> Object {
    match statement {
        Statement::Expression(expr_stmt) => eval_expression(expr_stmt.expression, env),
        Statement::Let(let_stmt) => {
            let val = eval_expression(let_stmt.value, Rc::clone(&env));
            if let Object::Error(_) = val {
                return val;
            }
            env.borrow_mut().set(let_stmt.name.value, val);
            Object::Null
        }
        Statement::Return(ret_stmt) => {
            let val = eval_expression(ret_stmt.return_value, env);
            if let Object::Error(_) = val {
                return val;
            }
            Object::ReturnValue(Box::new(val))
        }
        Statement::Block(block_stmt) => eval_block_statement(block_stmt, env),
        Statement::ClassDeclaration(class_decl) => eval_class_declaration(class_decl, env),
        Statement::StructDeclaration(struct_decl) => eval_struct_declaration(struct_decl, env),
        Statement::InterfaceDeclaration(interface_decl) => {
            eval_interface_declaration(interface_decl, env)
        }
    }
}

fn eval_class_declaration(class_decl: ClassDeclaration, env: Rc<RefCell<Environment>>) -> Object {
    let name = class_decl.name.value.clone();

    let super_class = if let Some(sc) = class_decl.super_class {
        let super_class_obj = eval_expression(Expression::Identifier(sc.clone()), Rc::clone(&env));
        match super_class_obj {
            Object::Class(c) => Some(c),
            Object::Error(_) => return super_class_obj,
            _ => {
                return Object::Error(format!(
                    "super class must be a class, got {}",
                    super_class_obj.type_str()
                ))
            }
        }
    } else {
        None
    };

    let mut properties = HashMap::new();
    for prop_decl in class_decl.properties {
        let value = if let Some(val_expr) = prop_decl.value {
            let val = eval_expression(val_expr, Rc::clone(&env));
            if let Object::Error(_) = val {
                return val;
            }
            val
        } else {
            Object::Null
        };
        properties.insert(prop_decl.name.value, value);
    }

    let mut methods = HashMap::new();
    for method_decl in class_decl.methods {
        let method = Method {
            name: method_decl.name.value.clone(),
            parameters: method_decl.parameters,
            body: method_decl.body,
            env: Rc::clone(&env),
            this: None,
        };
        methods.insert(method_decl.name.value, Rc::new(RefCell::new(method)));
    }

    let class = Rc::new(RefCell::new(Class {
        name: name.clone(),
        super_class,
        interfaces: vec![],
        properties,
        methods,
    }));

    env.borrow_mut().set(name, Object::Class(Rc::clone(&class)));
    Object::Null
}

fn eval_struct_declaration(
    struct_decl: StructDeclaration,
    env: Rc<RefCell<Environment>>,
) -> Object {
    let name = struct_decl.name.value.clone();
    let struct_obj = Rc::new(RefCell::new(Struct {
        name: name.clone(),
        properties: HashMap::new(),
    }));
    env.borrow_mut()
        .set(name, Object::Struct(Rc::clone(&struct_obj)));
    Object::Null
}

fn eval_interface_declaration(
    interface_decl: InterfaceDeclaration,
    env: Rc<RefCell<Environment>>,
) -> Object {
    let name = interface_decl.name.value.clone();
    let interface = Rc::new(RefCell::new(Interface {
        name: name.clone(),
        method_signatures: HashMap::new(),
    }));
    env.borrow_mut()
        .set(name, Object::Interface(Rc::clone(&interface)));
    Object::Null
}

fn eval_expression(expression: Expression, env: Rc<RefCell<Environment>>) -> Object {
    match expression {
        Expression::IntegerLiteral(il) => Object::Integer(il.value),
        Expression::Boolean(b) => Object::Boolean(b.value),
        Expression::Prefix(pe) => {
            let right = eval_expression(*pe.right, env);
            if let Object::Error(_) = right {
                return right;
            }
            eval_prefix_expression(&pe.operator, right)
        }
        Expression::Infix(ie) => {
            let left = eval_expression(*ie.left, Rc::clone(&env));
            if let Object::Error(_) = left {
                return left;
            }
            let right = eval_expression(*ie.right, env);
            if let Object::Error(_) = right {
                return right;
            }
            eval_infix_expression(&ie.operator, left, right)
        }
        Expression::If(ie) => eval_if_expression(ie, env),
        Expression::Identifier(i) => eval_identifier(i, env),
        Expression::FunctionLiteral(fl) => Object::Function(fl.parameters, fl.body, env),
        Expression::Call(ce) => {
            let function = eval_expression(*ce.function, Rc::clone(&env));
            if let Object::Error(_) = function {
                return function;
            }
            let args = eval_expressions(ce.arguments, env);
            if args.len() == 1 {
                if let Object::Error(_) = args[0] {
                    return args[0].clone();
                }
            }
            apply_function(function, args)
        }
        Expression::StringLiteral(s) => Object::String(s.value),
        Expression::ArrayLiteral(al) => {
            let elements = eval_expressions(al.elements, env);
            if elements.len() == 1 {
                if let Object::Error(_) = elements[0] {
                    return elements[0].clone();
                }
            }
            Object::Array(elements)
        }
        Expression::New(ne) => eval_new_expression(ne, env),
        Expression::This(te) => eval_this_expression(te, env),
        Expression::Super(_) => todo!(),
        Expression::PropertyAccess(pae) => eval_property_access_expression(pae, env),
        Expression::MethodCall(mce) => eval_method_call_expression(mce, env),
        // Добавляем обработку match-выражений, чтобы устранить ошибку компиляции.
        Expression::Match(me) => eval_match_expression(me, env),
    }
}

fn eval_block_statement(block: BlockStatement, env: Rc<RefCell<Environment>>) -> Object {
    let mut result = Object::Null;
    for statement in block.statements {
        result = eval_statement(statement, Rc::clone(&env));
        match result {
            Object::ReturnValue(_) | Object::Error(_) => return result,
            _ => {}
        }
    }
    result
}

fn eval_prefix_expression(operator: &str, right: Object) -> Object {
    match operator {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_prefix_operator_expression(right),
        _ => Object::Error(format!(
            "unknown operator: {}{}",
            operator,
            right.type_str()
        )),
    }
}

fn eval_bang_operator_expression(right: Object) -> Object {
    match right {
        Object::Boolean(true) => Object::Boolean(false),
        Object::Boolean(false) => Object::Boolean(true),
        Object::Null => Object::Boolean(true),
        _ => Object::Boolean(false),
    }
}

fn eval_minus_prefix_operator_expression(right: Object) -> Object {
    match right {
        Object::Integer(i) => Object::Integer(-i),
        _ => Object::Error(format!("unknown operator: -{}", right.type_str())),
    }
}

fn eval_infix_expression(operator: &str, left: Object, right: Object) -> Object {
    match (&left, &right) {
        (Object::Integer(l), Object::Integer(r)) => eval_integer_infix_expression(operator, *l, *r),
        (Object::Boolean(l), Object::Boolean(r)) => eval_boolean_infix_expression(operator, *l, *r),
        (Object::String(l), Object::String(r)) => eval_string_infix_expression(operator, l, r),
        (Object::String(l), Object::Integer(r)) if operator == "*" => {
            eval_string_integer_infix_expression(operator, l, *r)
        }
        // Сравнение экземпляров классов
        (Object::ClassInstance(l), Object::ClassInstance(r)) => match operator {
            "==" => Object::Boolean(Rc::ptr_eq(l, r)),
            "!=" => Object::Boolean(!Rc::ptr_eq(l, r)),
            _ => Object::Error(format!(
                "unknown operator: {} {} {}",
                left.type_str(),
                operator,
                right.type_str()
            )),
        },
        // Сравнение экземпляров структур
        (Object::StructInstance(l), Object::StructInstance(r)) => match operator {
            "==" => Object::Boolean(Rc::ptr_eq(l, r)),
            "!=" => Object::Boolean(!Rc::ptr_eq(l, r)),
            _ => Object::Error(format!(
                "unknown operator: {} {} {}",
                left.type_str(),
                operator,
                right.type_str()
            )),
        },
        _ => Object::Error(format!(
            "type mismatch: {} {} {}",
            left.type_str(),
            operator,
            right.type_str()
        )),
    }
}

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Object {
    match operator {
        "+" => Object::Integer(left + right),
        "-" => Object::Integer(left - right),
        "*" => Object::Integer(left * right),
        "/" => {
            if right == 0 {
                return Object::Error("division by zero".to_string());
            }
            Object::Integer(left / right)
        }
        "**" => {
            if right < 0 {
                return Object::Error("negative exponent not supported".to_string());
            }
            Object::Integer(left.pow(right as u32))
        }
        "%" => {
            if right == 0 {
                return Object::Error("modulo by zero".to_string());
            }
            Object::Integer(left % right)
        }
        "<" => Object::Boolean(left < right),
        ">" => Object::Boolean(left > right),
        "==" => Object::Boolean(left == right),
        "!=" => Object::Boolean(left != right),
        _ => Object::Error(format!(
            "unknown operator: {} {} {}",
            "INTEGER", operator, "INTEGER"
        )),
    }
}

fn eval_boolean_infix_expression(operator: &str, left: bool, right: bool) -> Object {
    match operator {
        "==" => Object::Boolean(left == right),
        "!=" => Object::Boolean(left != right),
        "&&" => Object::Boolean(left && right),
        "||" => Object::Boolean(left || right),
        _ => Object::Error(format!(
            "unknown operator: {} {} {}",
            "BOOLEAN", operator, "BOOLEAN"
        )),
    }
}

fn eval_string_infix_expression(operator: &str, left: &str, right: &str) -> Object {
    match operator {
        "+" => Object::String(format!("{}{}", left, right)),
        _ => Object::Error(format!(
            "unknown operator: {} {} {}",
            "STRING", operator, "STRING"
        )),
    }
}

fn eval_string_integer_infix_expression(operator: &str, left: &str, right: i64) -> Object {
    match operator {
        "*" => {
            if right < 0 {
                return Object::Error(
                    "negative multiplier not supported for string multiplication".to_string(),
                );
            }
            Object::String(left.repeat(right as usize))
        }
        _ => Object::Error(format!(
            "unknown operator: {} {} {}",
            "STRING", operator, "INTEGER"
        )),
    }
}

fn eval_if_expression(ie: IfExpression, env: Rc<RefCell<Environment>>) -> Object {
    let condition = eval_expression(*ie.condition, Rc::clone(&env));
    if is_truthy(condition) {
        eval_block_statement(ie.consequence, env)
    } else if let Some(alt) = ie.alternative {
        eval_block_statement(alt, env)
    } else {
        Object::Null
    }
}

fn is_truthy(obj: Object) -> bool {
    match obj {
        Object::Null => false,
        Object::Boolean(true) => true,
        Object::Boolean(false) => false,
        _ => true,
    }
}

fn eval_identifier(ident: Identifier, env: Rc<RefCell<Environment>>) -> Object {
    match env.borrow().get(&ident.value) {
        Some(o) => o,
        None => Object::Error(format!("identifier not found: {}", ident.value)),
    }
}

fn eval_expressions(exps: Vec<Expression>, env: Rc<RefCell<Environment>>) -> Vec<Object> {
    exps.into_iter()
        .map(|e| eval_expression(e, Rc::clone(&env)))
        .collect()
}

fn apply_function(func: Object, args: Vec<Object>) -> Object {
    match func {
        Object::Function(params, body, env) => {
            let extended_env = extend_function_env(&params, args, &env);
            let evaluated = eval_block_statement(body, extended_env);
            unwrap_return_value(evaluated)
        }
        Object::Method(method_rc) => {
            let method = method_rc.borrow();
            let instance = method
                .this
                .as_ref()
                .expect("method.this should be set before calling")
                .clone();
            let mut extended_env = Environment::new_enclosed(Rc::clone(&method.env));
            extended_env.set("this".to_string(), Object::ClassInstance(instance));
            for (i, param) in method.parameters.iter().enumerate() {
                extended_env.set(param.value.clone(), args[i].clone());
            }
            let evaluated =
                eval_block_statement(method.body.clone(), Rc::new(RefCell::new(extended_env)));
            unwrap_return_value(evaluated)
        }
        _ => Object::Error(format!("not a function: {}", func.type_str())),
    }
}

fn extend_function_env(
    params: &[Identifier],
    args: Vec<Object>,
    env: &Rc<RefCell<Environment>>,
) -> Rc<RefCell<Environment>> {
    let mut new_env = Environment::new_enclosed(Rc::clone(env));
    for (i, param) in params.iter().enumerate() {
        new_env.set(param.value.clone(), args[i].clone());
    }
    Rc::new(RefCell::new(new_env))
}

fn unwrap_return_value(obj: Object) -> Object {
    match obj {
        Object::ReturnValue(val) => *val,
        _ => obj,
    }
}

fn eval_new_expression(new_expr: NewExpression, env: Rc<RefCell<Environment>>) -> Object {
    let class_name = &new_expr.class_name.value;
    match env.borrow().get(class_name) {
        Some(Object::Class(class_obj)) => {
            let mut fields = HashMap::new();
            for (name, value) in &class_obj.borrow().properties {
                fields.insert(name.clone(), value.clone());
            }

            let instance = Rc::new(RefCell::new(ClassInstance {
                class: Rc::clone(&class_obj),
                fields,
            }));
            Object::ClassInstance(instance)
        }
        Some(Object::Struct(struct_obj)) => {
            let instance = Rc::new(RefCell::new(StructInstance {
                struct_def: Rc::clone(&struct_obj),
                fields: HashMap::new(),
            }));
            Object::StructInstance(instance)
        }
        Some(_) => Object::Error(format!("not a class or struct: {}", class_name)),
        None => Object::Error(format!("type not found: {}", class_name)),
    }
}

fn eval_property_access_expression(
    pae: PropertyAccessExpression,
    env: Rc<RefCell<Environment>>,
) -> Object {
    let left = eval_expression(*pae.left, Rc::clone(&env));
    if let Object::Error(_) = left {
        return left;
    }

    let property_name = &pae.property.value;

    match left {
        Object::StructInstance(instance_rc) => {
            let instance = instance_rc.borrow();
            if let Some(value) = instance.fields.get(property_name) {
                return value.clone();
            }
            Object::Error(format!(
                "property '{}' not found on struct '{}'",
                property_name,
                instance.struct_def.borrow().name
            ))
        }
        Object::ClassInstance(instance_rc) => {
            let instance = instance_rc.borrow();
            if let Some(value) = instance.fields.get(property_name) {
                return value.clone();
            }

            if let Some(method) = find_method_in_class(Rc::clone(&instance.class), property_name) {
                return bind_method(method, &instance_rc);
            }

            if let Some(value) = instance.class.borrow().properties.get(property_name) {
                return value.clone();
            }

            Object::Error(format!(
                "property '{}' not found on class '{}'",
                property_name,
                instance.class.borrow().name
            ))
        }
        _ => Object::Error(format!(
            "property access not supported for type '{}'",
            left.type_str()
        )),
    }
}

fn eval_this_expression(_this_expr: ThisExpression, env: Rc<RefCell<Environment>>) -> Object {
    match env.borrow().get("this") {
        Some(this_obj) => this_obj,
        None => Object::Error("'this' can only be used inside a method".to_string()),
    }
}

fn eval_method_call_expression(mce: MethodCallExpression, env: Rc<RefCell<Environment>>) -> Object {
    let method = eval_expression(
        Expression::PropertyAccess(PropertyAccessExpression {
            token: mce.token.clone(),
            left: mce.object,
            property: mce.method,
        }),
        Rc::clone(&env),
    );

    if let Object::Error(_) = method {
        return method;
    }

    let args = eval_expressions(mce.arguments, env);
    if args.len() == 1 {
        if let Object::Error(_) = args[0] {
            return args[0].clone();
        }
    }

    apply_function(method, args)
}

fn find_method_in_class(
    class_rc: Rc<RefCell<Class>>,
    method_name: &str,
) -> Option<Rc<RefCell<Method>>> {
    let class = class_rc.borrow();
    if let Some(method) = class.methods.get(method_name) {
        return Some(Rc::clone(method));
    }

    if let Some(super_class_rc) = &class.super_class {
        return find_method_in_class(Rc::clone(super_class_rc), method_name);
    }

    None
}

fn eval_match_expression(
    match_expr: crate::ast::MatchExpression,
    env: Rc<RefCell<Environment>>,
) -> Object {
    // Вычисляем значение, которое сопоставляем
    let value = eval_expression(*match_expr.value, Rc::clone(&env));

    if let Object::Error(_) = value {
        return value;
    }

    // Итерируем по всем ветвям match
    for arm in match_expr.arms {
        // Проверяем, совпадает ли паттерн
        if let Some(bindings) = pattern_matches(&arm.pattern, &value, Rc::clone(&env)) {
            // Если есть гард, проверяем его
            if let Some(guard_expr) = &arm.guard {
                let guard_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(&env))));

                // Применяем привязки из паттерна в окружение гарда
                for (name, obj) in &bindings {
                    guard_env.borrow_mut().set(name.clone(), obj.clone());
                }

                let guard_result = eval_expression(guard_expr.clone(), Rc::clone(&guard_env));

                if let Object::Error(_) = guard_result {
                    return guard_result;
                }

                if !is_truthy(guard_result) {
                    continue; // Гард не прошел, переходим к следующей ветви
                }
            }

            // Создаем новое окружение с привязками из паттерна
            let arm_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(&env))));
            for (name, obj) in bindings {
                arm_env.borrow_mut().set(name, obj);
            }

            // Выполняем consequence для этой ветви
            return eval_block_statement(arm.consequence, arm_env);
        }
    }

    // Если ни один паттерн не совпал, генерируем ошибку (проверка исчерпаемости)
    Object::Error(format!("non-exhaustive match pattern for value: {}", value))
}

/// Проверяет, совпадает ли паттерн со значением.
/// Возвращает Some(bindings) если совпал, где bindings - это переменные и их значения.
/// Возвращает None если паттерн не совпал.
fn pattern_matches(
    pattern: &crate::ast::Pattern,
    value: &Object,
    env: Rc<RefCell<Environment>>,
) -> Option<Vec<(String, Object)>> {
    match pattern {
        crate::ast::Pattern::Literal(expr) => {
            // Вычисляем литерал из выражения
            let literal_value = eval_expression(expr.clone(), env);
            if literal_value == *value {
                Some(vec![])
            } else {
                None
            }
        }
        crate::ast::Pattern::Identifier(ident) => {
            // Идентификатор всегда совпадает, но привязывает переменную
            Some(vec![(ident.value.clone(), value.clone())])
        }
        crate::ast::Pattern::Wildcard => {
            // Wildcard всегда совпадает, но не создает привязок
            Some(vec![])
        }
        crate::ast::Pattern::Range(range_pattern) => {
            // Проверяем, попадает ли значение в диапазон
            let start_val = eval_expression(*range_pattern.start.clone(), Rc::clone(&env));
            let end_val = eval_expression(*range_pattern.end.clone(), env);

            if let (Object::Integer(start), Object::Integer(end), Object::Integer(n)) =
                (&start_val, &end_val, value)
            {
                let in_range = if range_pattern.inclusive {
                    n >= start && n <= end
                } else {
                    n >= start && n < end
                };

                if in_range {
                    Some(vec![])
                } else {
                    None
                }
            } else {
                None // Типы не совпадают для диапазонного сопоставления
            }
        }
        crate::ast::Pattern::Tuple(patterns) => {
            // Проверяем, если значение - это массив с нужным количеством элементов
            if let Object::Array(elements) = value {
                if patterns.len() != elements.len() {
                    return None;
                }

                let mut all_bindings = vec![];

                // Проверяем каждый элемент кортежа
                for (pattern_elem, value_elem) in patterns.iter().zip(elements.iter()) {
                    if let Some(bindings) =
                        pattern_matches(pattern_elem, value_elem, Rc::clone(&env))
                    {
                        all_bindings.extend(bindings);
                    } else {
                        return None;
                    }
                }

                Some(all_bindings)
            } else {
                None
            }
        }
        crate::ast::Pattern::Struct(struct_pattern) => {
            // Проверяем сопоставление структуры
            if let Object::StructInstance(instance_rc) = value {
                let instance = instance_rc.borrow();
                let struct_def = instance.struct_def.borrow();

                // Проверяем имя структуры
                if struct_def.name != struct_pattern.name.value {
                    return None;
                }

                let mut all_bindings = vec![];

                // Проверяем каждое поле
                for (field_name, field_pattern_opt) in &struct_pattern.fields {
                    if let Some(field_value) = instance.fields.get(&field_name.value) {
                        if let Some(field_pattern) = field_pattern_opt {
                            // Если для поля указан паттерн, проверяем его
                            if let Some(bindings) =
                                pattern_matches(field_pattern, field_value, Rc::clone(&env))
                            {
                                all_bindings.extend(bindings);
                            } else {
                                return None;
                            }
                        } else {
                            // Если паттерн не указан, просто привязываем переменную
                            all_bindings.push((field_name.value.clone(), field_value.clone()));
                        }
                    } else {
                        return None; // Поле не найдено
                    }
                }

                Some(all_bindings)
            } else {
                None
            }
        }
    }
}

fn bind_method(method_rc: Rc<RefCell<Method>>, instance_rc: &Rc<RefCell<ClassInstance>>) -> Object {
    let mut bound_method = method_rc.borrow().clone();
    bound_method.this = Some(Rc::clone(instance_rc));
    Object::Method(Rc::new(RefCell::new(bound_method)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::object::{Environment, Object};
    use crate::parser::Parser;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn test_eval(input: &str) -> Object {
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = match parser.parse_program() {
            Ok(prog) => prog,
            Err(errors) => {
                return Object::Error(format!(
                    "Parse errors: {:?}",
                    errors
                        .iter()
                        .map(|e| format!("{:?}", e))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        };
        let env = Rc::new(RefCell::new(Environment::new()));
        eval(Node::Program(program), env)
    }

    #[test]
    fn test_integer_literal_expression() {
        let tests = vec![("5", 5), ("10", 10), ("-5", -5), ("-10", -10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, Object::Integer(expected));
        }
    }

    #[test]
    fn test_boolean_literal_expression() {
        let tests = vec![("true", true), ("false", false)];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, Object::Boolean(expected));
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = vec![
            ("!true", false),
            ("!false", true),
            ("!5", false),
            ("!!true", true),
            ("!!false", false),
            ("!!5", true),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, Object::Boolean(expected));
        }
    }

    #[test]
    fn test_infix_expressions() {
        let tests = vec![
            ("5 + 5", Object::Integer(10)),
            ("5 - 5", Object::Integer(0)),
            ("5 * 5", Object::Integer(25)),
            ("10 / 5", Object::Integer(2)),
            ("5 > 5", Object::Boolean(false)),
            ("5 < 5", Object::Boolean(false)),
            ("5 == 5", Object::Boolean(true)),
            ("5 != 5", Object::Boolean(false)),
            ("5 > 4", Object::Boolean(true)),
            ("5 < 6", Object::Boolean(true)),
            ("5 == 6", Object::Boolean(false)),
            ("5 != 4", Object::Boolean(true)),
            ("true == true", Object::Boolean(true)),
            ("false == false", Object::Boolean(true)),
            ("true == false", Object::Boolean(false)),
            ("true != false", Object::Boolean(true)),
            ("false != true", Object::Boolean(true)),
            ("(1 + 2) * 3", Object::Integer(9)),
            ("1 + (2 * 3)", Object::Integer(7)),
            (
                "\"hello\" + \" world\"",
                Object::String("hello world".to_string()),
            ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_power_operator() {
        let tests = vec![
            ("2 ** 3", Object::Integer(8)),
            ("5 ** 0", Object::Integer(1)),
            ("5 ** 1", Object::Integer(5)),
            ("0 ** 5", Object::Integer(0)),
            ("1 ** 5", Object::Integer(1)),
            (
                "2 ** -3",
                Object::Error("negative exponent not supported".to_string()),
            ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_string_multiplication() {
        let tests = vec![
            ("\"abc\" * 2", Object::String("abcabc".to_string())),
            ("\"abc\" * 0", Object::String("".to_string())),
            (
                "\"abc\" * -1",
                Object::Error(
                    "negative multiplier not supported for string multiplication".to_string(),
                ),
            ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_logical_operators() {
        let tests = vec![
            ("true && true", Object::Boolean(true)),
            ("true && false", Object::Boolean(false)),
            ("false && true", Object::Boolean(false)),
            ("false && false", Object::Boolean(false)),
            ("true || true", Object::Boolean(true)),
            ("true || false", Object::Boolean(true)),
            ("false || true", Object::Boolean(true)),
            ("false || false", Object::Boolean(false)),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_modulo_operator() {
        let tests = vec![
            ("10 % 3", Object::Integer(1)),
            ("-10 % 3", Object::Integer(-1)),
            ("10 % -3", Object::Integer(1)),
            ("-10 % -3", Object::Integer(-1)),
            ("10 % 0", Object::Error("modulo by zero".to_string())),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_operator_precedence() {
        let tests = vec![
            ("2 + 3 ** 2", Object::Integer(11)),             // 2 + 9
            ("true && 1 + 2 == 3", Object::Boolean(true)),   // true && (3 == 3)
            ("false || 10 % 3 == 1", Object::Boolean(true)), // false || (1 == 1)
            ("10 * 2 ** 2", Object::Integer(40)),            // 10 * 4
            ("10 / 2 % 2", Object::Integer(1)),              // 5 % 2
            ("5 + 5 * 2", Object::Integer(15)),              // 5 + 10
            ("(5 + 5) * 2", Object::Integer(20)),            // 10 * 2
            ("!true && false", Object::Boolean(false)),      // false && false
            ("!false || true", Object::Boolean(true)),       // true || true
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_if_else_expressions() {
        let tests = vec![
            ("if (true) { 10 }", Object::Integer(10)),
            ("if (false) { 10 }", Object::Null),
            ("if (1) { 10 }", Object::Integer(10)),
            ("if (1 < 2) { 10 }", Object::Integer(10)),
            ("if (1 > 2) { 10 }", Object::Null),
            ("if (1 > 2) { 10 } else { 20 }", Object::Integer(20)),
            ("if (1 < 2) { 10 } else { 20 }", Object::Integer(10)),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_return_statements() {
        let tests = vec![
            ("return 10;", Object::Integer(10)),
            ("return 10; 9;", Object::Integer(10)),
            ("return 2 * 5; 9;", Object::Integer(10)),
            ("9; return 2 * 5; 9;", Object::Integer(10)),
            (
                "if (true) { if (true) { return 10; } return 1; }",
                Object::Integer(10),
            ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = vec![
            ("let a = 5; a;", Object::Integer(5)),
            ("let a = 5 * 5; a;", Object::Integer(25)),
            ("let a = 5; let b = a; b;", Object::Integer(5)),
            (
                "let a = 5; let b = a; let c = a + b + 5; c;",
                Object::Integer(15),
            ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_function_application() {
        let tests = vec![
            (
                "let identity = fn(x) { x; }; identity(5);",
                Object::Integer(5),
            ),
            (
                "let identity = fn(x) { return x; }; identity(5);",
                Object::Integer(5),
            ),
            (
                "let double = fn(x) { x * 2; }; double(5);",
                Object::Integer(10),
            ),
            (
                "let add = fn(x, y) { x + y; }; add(5, 5);",
                Object::Integer(10),
            ),
            (
                "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
                Object::Integer(20),
            ),
            ("fn(x) { x; }(5)", Object::Integer(5)),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_closures() {
        let input = "
            let newAdder = fn(x) {
                fn(y) { x + y };
            };
            let addTwo = newAdder(2);
            addTwo(2);
        ";
        assert_eq!(test_eval(input), Object::Integer(4));
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            ("5 + true;", "type mismatch: INTEGER + BOOLEAN"),
            ("5 + true; 5;", "type mismatch: INTEGER + BOOLEAN"),
            ("-true", "unknown operator: -BOOLEAN"),
            ("true + false", "unknown operator: BOOLEAN + BOOLEAN"),
            ("5; true + false; 5", "unknown operator: BOOLEAN + BOOLEAN"),
            (
                "if (10 > 1) { true + false; }",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            (
                "if (10 > 1) { if (10 > 1) { return true + false; } return 1; }",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            ("foobar", "identifier not found: foobar"),
            ("let foo = 10; foo();", "not a function: INTEGER"),
            ("\"abc\" + 1;", "type mismatch: STRING + INTEGER"),
            ("1 + \"abc\";", "type mismatch: INTEGER + STRING"),
        ];

        for (input, expected_message) in tests {
            let evaluated = test_eval(input);
            if let Object::Error(msg) = evaluated {
                assert_eq!(msg, expected_message);
            } else {
                panic!("expected error, got {:?}", evaluated);
            }
        }
    }

    #[test]
    fn test_class_declaration() {
        let input = "class A {}; A;";
        let evaluated = test_eval(input);
        match evaluated {
            Object::Class(class_obj) => {
                assert_eq!(class_obj.borrow().name, "A");
            }
            _ => panic!("expected class object, got {:?}", evaluated),
        }
    }

    #[test]
    fn test_struct_declaration() {
        let input = "struct B {}; B;";
        let evaluated = test_eval(input);
        match evaluated {
            Object::Struct(struct_obj) => {
                assert_eq!(struct_obj.borrow().name, "B");
            }
            _ => panic!("expected struct object, got {:?}", evaluated),
        }
    }

    #[test]
    fn test_new_expression() {
        let tests = vec![
            ("class MyClass {}; new MyClass();", "instance of MyClass"),
            (
                "struct MyStruct {}; new MyStruct();",
                "instance of struct MyStruct",
            ),
            ("let a = 10; new a();", "ERROR: not a class or struct: a"),
            ("new NonExistent();", "ERROR: type not found: NonExistent"),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated.to_string(), expected);
        }
    }

    #[test]
    fn test_interface_declaration() {
        let input = "interface C {}; C;";
        let evaluated = test_eval(input);
        match evaluated {
            Object::Interface(interface_obj) => {
                assert_eq!(interface_obj.borrow().name, "C");
            }
            _ => panic!("expected interface object, got {:?}", evaluated),
        }
    }

    #[test]
    fn test_class_member_evaluation() {
        let tests = vec![
            (
                r#"
                class Point {
                    public x = 10;
                    public y;
                    public getX() {
                        return this.x;
                    }
                }
                let p = new Point();
                p.x;
                "#,
                Object::Integer(10),
            ),
            (
                r#"
                class Point {
                    public x = 10;
                    public y;
                    public getX() {
                        return this.x;
                    }
                }
                let p = new Point();
                p.y;
                "#,
                Object::Null,
            ),
            (
                r#"
                class Point {
                    public x = 10;
                    public y = 20;
                    public getX() {
                        return this.x;
                    }
                }
                let p = new Point();
                p.getX();
                "#,
                Object::Integer(10),
            ),
            (
                r#"
                class Adder {
                    public a = 1;
                    public b = 2;
                    public sum() {
                        return this.a + this.b;
                    }
                }
                let adder = new Adder();
                adder.sum();
                "#,
                Object::Integer(3),
            ),
            (
                r#"
                class Greeter {
                    public message = "hello";
                    public greet() {
                        return this.message;
                    }
                }
                let g = new Greeter();
                g.greet();
                "#,
                Object::String("hello".to_string()),
            ),
            (
                r#"
                class Counter {
                    public count = 0;
                    public increment() {
                        // Note: This doesn't modify the state in our current implementation,
                        // as we don't have assignment to properties yet.
                        // It just tests access and return.
                        return this.count + 1;
                    }
                }
                let c = new Counter();
                c.increment();
                "#,
                Object::Integer(1),
            ),
            (
                r#"
                class Test {
                    public a = 5;
                }
                let t = new Test();
                t.nonexistent;
                "#,
                Object::Error("property 'nonexistent' not found on class 'Test'".to_string()),
            ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(
                evaluated, expected,
                "Failed on input:\n{}\nExpected: {:?}, Got: {:?}",
                input, expected, evaluated
            );
        }
    }

    #[test]
    fn test_this_expression() {
        let input = r#"
        class Person {
            public getThis() {
                return this;
            }
        }
        let p = new Person();
        let p2 = p.getThis();
        p == p2;
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Boolean(true));
    }

    #[test]
    fn test_this_outside_class_error() {
        let input = "this;";
        let evaluated = test_eval(input);
        assert_eq!(
            evaluated,
            Object::Error("'this' can only be used inside a method".to_string())
        );
    }

    #[test]
    fn test_inheritance() {
        let tests = vec![
            (
                r#"
                class Animal {
                    public speak() {
                        return "animal sound";
                    }
                }
                class Dog extends Animal {
                    public speak() {
                        return "woof";
                    }
                }
                let d = new Dog();
                d.speak();
                "#,
                Object::String("woof".to_string()),
            ),
            (
                r#"
                class Animal {
                    public speak() {
                        return "animal sound";
                    }
                }
                class Dog extends Animal {}
                let d = new Dog();
                d.speak();
                "#,
                Object::String("animal sound".to_string()),
            ),
            (
                r#"
                let NotAClass = 10;
                class B extends NotAClass {}
                "#,
                Object::Error("super class must be a class, got INTEGER".to_string()),
            ),
            (
                r#"
                class B extends NonExistent {}
                "#,
                Object::Error("identifier not found: NonExistent".to_string()),
            ),
            (
                r#"
                class A { public methodA() { return 1; } }
                class B extends A { public methodB() { return 2; } }
                class C extends B { public methodC() { return 3; } }
                let c = new C();
                c.methodA();
                "#,
                Object::Integer(1),
            ),
        ];
        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(
                evaluated, expected,
                "Failed on input:\n{}\nExpected: {:?}, Got: {:?}",
                input, expected, evaluated
            );
        }
    }

    #[test]
    fn test_match_literal_patterns() {
        // Тест сопоставления с литеральными паттернами (целые числа)
        let input = r#"
            let x = 2;
            match x {
                1 => 10,
                2 => 20,
                3 => 30,
            }
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(20));
    }

    #[test]
    fn test_match_with_identifier_pattern() {
        // Тест сопоставления с идентификаторным паттерном (привязка переменной)
        let input = r#"
            let x = 5;
            match x {
                n => n * 2,
            }
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(10));
    }

    #[test]
    fn test_match_with_wildcard() {
        // Тест сопоставления с wildcard паттерном (_)
        let input = r#"
            let x = 100;
            match x {
                1 => 10,
                2 => 20,
                _ => 99,
            }
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(99));
    }

    #[test]
    fn test_match_with_guard() {
        // Тест сопоставления с гардом
        let input = r#"
            let x = 5;
            match x {
                n if n < 3 => 10,
                n if n < 7 => 20,
                _ => 30,
            }
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(20));
    }

    #[test]
    fn test_match_non_exhaustive() {
        // Тест проверки исчерпаемости паттернов (non-exhaustive match)
        let input = r#"
            let x = 99;
            match x {
                1 => 10,
                2 => 20,
            }
        "#;
        let evaluated = test_eval(input);
        if let Object::Error(msg) = evaluated {
            assert!(msg.contains("non-exhaustive match"));
        } else {
            panic!(
                "expected error for non-exhaustive match, got {:?}",
                evaluated
            );
        }
    }

    #[test]
    fn test_match_with_range_pattern() {
        // Тест сопоставления с диапазоном
        let input = r#"
            let x = 5;
            match x {
                1..3 => 10,
                3..7 => 20,
                7..10 => 30,
                _ => 99,
            }
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(20));
    }

    #[test]
    fn test_match_boolean_patterns() {
        // Тест сопоставления с булевыми паттернами
        let input = r#"
            let x = true;
            match x {
                true => 1,
                false => 0,
            }
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(1));
    }

    #[test]
    fn test_match_string_patterns() {
        // Тест сопоставления со строковыми паттернами
        let input = r#"
            let x = "hello";
            match x {
                "hello" => 1,
                "world" => 2,
                _ => 0,
            }
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(1));
    }

    #[test]
    fn test_match_tuple_pattern() {
        // Тест сопоставления с кортежным паттерном
        let input = r#"
            let x = [1, 2];
            match x {
                [a, b] => a + b,
                _ => 0,
            }
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(3));
    }

    #[test]
    fn test_match_nested_guards() {
        // Тест вложенных гардов
        let input = r#"
            let x = 15;
            match x {
                n if n < 10 => 1,
                n if n < 20 => 2,
                n if n < 30 => 3,
                _ => 4,
            }
        "#;
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(2));
    }
}
