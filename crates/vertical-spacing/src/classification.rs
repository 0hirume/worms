use larvae::syntax::ast::{CallArgs, Expr, IndexKey, Stmt, TableField};
use larvae::syntax::lexer::Tok;

use crate::syntax::span_text;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    Tight,
    Blank,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TopGroup {
    Service,
    Require,
    ReimportedType,
    LocalType,
    ExportedType,
    Class,
    ModuleReturn,
}

pub fn gap_between(
    previous: &Stmt,
    next: &Stmt,
    source: &str,
    tokens: &[Tok],
    is_top: bool,
) -> Option<Gap> {
    if matches!(next, Stmt::Return(_)) || is_block_statement(previous) || is_block_statement(next) {
        return Some(Gap::Blank);
    }

    if has_expanded_expression(previous, source, tokens)
        || has_expanded_expression(next, source, tokens)
    {
        return Some(Gap::Blank);
    }

    if let (Some(declaration), Some(assignment)) = (
        declaration_name(previous, source, tokens),
        indexed_assignment_root(next, source, tokens),
    ) {
        return Some(if declaration == assignment {
            Gap::Tight
        } else {
            Gap::Blank
        });
    }

    if let (Some(previous_root), Some(next_root)) = (
        indexed_assignment_root(previous, source, tokens),
        indexed_assignment_root(next, source, tokens),
    ) {
        return Some(if previous_root == next_root {
            Gap::Tight
        } else {
            Gap::Blank
        });
    }

    if indexed_assignment_root(previous, source, tokens).is_some() && matches!(next, Stmt::Local(_))
    {
        return Some(Gap::Blank);
    }

    if is_multiline_declaration(previous, source, tokens)
        || is_multiline_declaration(next, source, tokens)
    {
        return Some(Gap::Blank);
    }

    if !is_top {
        return None;
    }

    let previous_group = top_group(previous, source, tokens);
    let next_group = top_group(next, source, tokens);

    match (previous_group, next_group) {
        (Some(TopGroup::Require), Some(TopGroup::Require)) => {
            match (
                require_alias(previous, source, tokens),
                require_alias(next, source, tokens),
            ) {
                (Some(left), Some(right)) if left == right => Some(Gap::Tight),
                (Some(_), Some(_)) => Some(Gap::Blank),
                _ => None,
            }
        }
        (Some(TopGroup::LocalType), Some(TopGroup::LocalType)) => Some(Gap::Tight),
        (Some(left), Some(right)) if left == right => Some(Gap::Tight),
        (Some(_), Some(_)) => Some(Gap::Blank),
        _ => None,
    }
}

fn top_group(statement: &Stmt, source: &str, tokens: &[Tok]) -> Option<TopGroup> {
    match statement {
        Stmt::Local(local)
            if local.values.first().is_some_and(|value| {
                let value = span_text(value.span(), source, tokens).trim_start();

                value.starts_with("game:GetService(") || value.starts_with("game:GetService (")
            }) =>
        {
            Some(TopGroup::Service)
        }
        Stmt::Local(_) if require_alias(statement, source, tokens).is_some() => {
            Some(TopGroup::Require)
        }
        Stmt::TypeAlias(alias) if alias.exported => Some(TopGroup::ExportedType),
        Stmt::TypeAlias(alias) if is_reimported_type(alias.span, source, tokens) => {
            Some(TopGroup::ReimportedType)
        }
        Stmt::TypeAlias(_) => Some(TopGroup::LocalType),
        Stmt::Local(_) if class_name(statement, source, tokens).is_some() => Some(TopGroup::Class),
        Stmt::Assign(_)
            if indexed_assignment_root(statement, source, tokens).is_some_and(is_pascal_case) =>
        {
            Some(TopGroup::Class)
        }
        Stmt::Return(_) => Some(TopGroup::ModuleReturn),
        _ => None,
    }
}

fn require_alias<'a>(statement: &Stmt, source: &'a str, tokens: &[Tok]) -> Option<&'a str> {
    let Stmt::Local(local) = statement else {
        return None;
    };
    let value = local.values.first()?;
    let path = require_path(value, source, tokens)?;

    path.split('/').next()
}

fn require_path<'a>(expression: &Expr, source: &'a str, tokens: &[Tok]) -> Option<&'a str> {
    match expression {
        Expr::Call { func, args, .. } if matches!(func.as_ref(), Expr::Name(name) if span_text(*name, source, tokens) == "require") =>
        {
            let span = match args {
                CallArgs::Paren(arguments) => match arguments.first()? {
                    Expr::String(span) => *span,
                    _ => return None,
                },
                CallArgs::Str(span) => *span,
                CallArgs::Table(_) => return None,
            };
            let literal = span_text(span, source, tokens);

            unquote(literal)
        }
        Expr::Paren { inner, .. } | Expr::TypeAssert { expr: inner, .. } => {
            require_path(inner, source, tokens)
        }
        _ => None,
    }
}

fn unquote(literal: &str) -> Option<&str> {
    let quote = literal.as_bytes().first()?;

    if !matches!(quote, b'\'' | b'"') || literal.as_bytes().last() != Some(quote) {
        return None;
    }

    literal.get(1..literal.len().checked_sub(1)?)
}

fn class_name<'a>(statement: &Stmt, source: &'a str, tokens: &[Tok]) -> Option<&'a str> {
    let Stmt::Local(local) = statement else {
        return None;
    };

    if local.names.len() != 1
        || local.values.len() != 1
        || !matches!(local.values.first(), Some(Expr::Table { .. }))
    {
        return None;
    }

    let name = span_text(local.names.first()?.name, source, tokens);

    is_pascal_case(name).then_some(name)
}

fn declaration_name<'a>(statement: &Stmt, source: &'a str, tokens: &[Tok]) -> Option<&'a str> {
    let Stmt::Local(local) = statement else {
        return None;
    };

    (local.names.len() == 1).then(|| span_text(local.names[0].name, source, tokens))
}

fn indexed_assignment_root<'a>(
    statement: &Stmt,
    source: &'a str,
    tokens: &[Tok],
) -> Option<&'a str> {
    let Stmt::Assign(assignment) = statement else {
        return None;
    };
    let mut targets = assignment.targets.iter();
    let root = indexed_root_name(targets.next()?, source, tokens)?;

    targets
        .all(|target| indexed_root_name(target, source, tokens) == Some(root))
        .then_some(root)
}

fn indexed_root_name<'a>(expression: &Expr, source: &'a str, tokens: &[Tok]) -> Option<&'a str> {
    let Expr::Index { object, .. } = expression else {
        return None;
    };

    root_name(object, source, tokens)
}

fn root_name<'a>(expression: &Expr, source: &'a str, tokens: &[Tok]) -> Option<&'a str> {
    match expression {
        Expr::Name(name) => Some(span_text(*name, source, tokens)),
        Expr::Index { object, .. } => root_name(object, source, tokens),
        Expr::Paren { inner, .. } | Expr::TypeAssert { expr: inner, .. } => {
            root_name(inner, source, tokens)
        }
        Expr::Nil(_)
        | Expr::True(_)
        | Expr::False(_)
        | Expr::Vararg(_)
        | Expr::Number(_)
        | Expr::String(_)
        | Expr::InterpString(_)
        | Expr::Function { .. }
        | Expr::Table { .. }
        | Expr::Binary { .. }
        | Expr::Unary { .. }
        | Expr::Call { .. }
        | Expr::IfElse { .. } => None,
    }
}

fn is_multiline_declaration(statement: &Stmt, source: &str, tokens: &[Tok]) -> bool {
    matches!(statement, Stmt::Local(_) | Stmt::TypeAlias(_))
        && span_text(statement.span(), source, tokens).contains('\n')
}

fn has_expanded_expression(statement: &Stmt, source: &str, tokens: &[Tok]) -> bool {
    let expressions: &[Expr] = match statement {
        Stmt::Assign(assignment) => &assignment.values,
        Stmt::Return(statement) => &statement.values,
        Stmt::Call(expression, _) => std::slice::from_ref(expression),
        _ => return false,
    };

    expressions
        .iter()
        .any(|expression| is_expanded_expression(expression, source, tokens))
}

fn is_expanded_expression(expression: &Expr, source: &str, tokens: &[Tok]) -> bool {
    if matches!(
        expression,
        Expr::Call { .. } | Expr::Function { .. } | Expr::IfElse { .. } | Expr::Table { .. }
    ) && span_text(expression.span(), source, tokens).contains('\n')
    {
        return true;
    }

    match expression {
        Expr::Table { fields, .. } => fields.iter().any(|field| match field {
            TableField::Positional(value) | TableField::Named { value, .. } => {
                is_expanded_expression(value, source, tokens)
            }
            TableField::Computed { key, value } => {
                is_expanded_expression(key, source, tokens)
                    || is_expanded_expression(value, source, tokens)
            }
        }),
        Expr::Binary { lhs, rhs, .. } => {
            is_expanded_expression(lhs, source, tokens)
                || is_expanded_expression(rhs, source, tokens)
        }
        Expr::Unary { operand, .. } => is_expanded_expression(operand, source, tokens),
        Expr::Paren { inner, .. } | Expr::TypeAssert { expr: inner, .. } => {
            is_expanded_expression(inner, source, tokens)
        }
        Expr::Index { object, key, .. } => {
            is_expanded_expression(object, source, tokens)
                || matches!(key, IndexKey::Computed(key) if is_expanded_expression(key, source, tokens))
        }
        Expr::Call { func, args, .. } => {
            is_expanded_expression(func, source, tokens)
                || match args {
                    CallArgs::Paren(arguments) => arguments
                        .iter()
                        .any(|argument| is_expanded_expression(argument, source, tokens)),
                    CallArgs::Table(table) => is_expanded_expression(table, source, tokens),
                    CallArgs::Str(_) => false,
                }
        }
        Expr::IfElse {
            branches,
            else_value,
            ..
        } => {
            branches.iter().any(|(condition, value)| {
                is_expanded_expression(condition, source, tokens)
                    || is_expanded_expression(value, source, tokens)
            }) || is_expanded_expression(else_value, source, tokens)
        }
        Expr::Nil(_)
        | Expr::True(_)
        | Expr::False(_)
        | Expr::Vararg(_)
        | Expr::Number(_)
        | Expr::String(_)
        | Expr::InterpString(_)
        | Expr::Function { .. }
        | Expr::Name(_) => false,
    }
}

fn is_reimported_type(span: larvae::syntax::ast::TokSpan, source: &str, tokens: &[Tok]) -> bool {
    let text = span_text(span, source, tokens);
    let Some((_, value)) = text.split_once('=') else {
        return false;
    };

    let value = value.trim_start();
    let name_length = value
        .bytes()
        .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        .count();

    name_length > 0 && value.as_bytes().get(name_length) == Some(&b'.')
}

fn is_pascal_case(name: &str) -> bool {
    name.bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_uppercase())
        && name.bytes().any(|byte| byte.is_ascii_lowercase())
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

const fn is_block_statement(statement: &Stmt) -> bool {
    matches!(
        statement,
        Stmt::Do(_)
            | Stmt::While(_)
            | Stmt::Repeat(_)
            | Stmt::If(_)
            | Stmt::NumericFor(_)
            | Stmt::GenericFor(_)
            | Stmt::Function(_)
            | Stmt::LocalFunction(_)
            | Stmt::Class(_)
    )
}
