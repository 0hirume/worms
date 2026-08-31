use larvae::syntax::ast::{Block, Expr, Stmt};
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
    Constant,
    Function,
    ModuleReturn,
}

pub fn gap_between(
    previous: &Stmt,
    next: &Stmt,
    source: &str,
    tokens: &[Tok],
    is_top: bool,
) -> Option<Gap> {
    if matches!(next, Stmt::Return(_)) || is_guard(previous, source, tokens) {
        return Some(Gap::Blank);
    }

    if !is_top {
        return None;
    }

    let previous_group = top_group(previous, source, tokens);
    let next_group = top_group(next, source, tokens);

    match (previous_group, next_group) {
        (Some(TopGroup::Function), Some(TopGroup::Function)) => Some(Gap::Blank),
        (Some(TopGroup::LocalType), Some(TopGroup::LocalType)) => {
            match (
                is_multiline_type(previous, source, tokens),
                is_multiline_type(next, source, tokens),
            ) {
                (false, false) => Some(Gap::Tight),
                _ => Some(Gap::Blank),
            }
        }
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
        Stmt::Local(local)
            if local.values.first().is_some_and(|value| {
                let value = span_text(value.span(), source, tokens).trim_start();

                value.starts_with("require(") || value.starts_with("require ")
            }) =>
        {
            Some(TopGroup::Require)
        }
        Stmt::TypeAlias(alias) if is_reimported_type(alias.span, source, tokens) => {
            Some(TopGroup::ReimportedType)
        }
        Stmt::TypeAlias(_) => Some(TopGroup::LocalType),
        Stmt::Local(local)
            if local.is_const
                || (!local.names.is_empty()
                    && local.names.iter().all(|binding| {
                        is_screaming_snake(span_text(binding.name, source, tokens))
                    })) =>
        {
            Some(TopGroup::Constant)
        }
        Stmt::Function(_) | Stmt::LocalFunction(_) => Some(TopGroup::Function),
        Stmt::Return(_) => Some(TopGroup::ModuleReturn),
        _ => None,
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

fn is_multiline_type(statement: &Stmt, source: &str, tokens: &[Tok]) -> bool {
    let Stmt::TypeAlias(alias) = statement else {
        return false;
    };

    let text = span_text(alias.span, source, tokens);
    let Some((_, value)) = text.split_once('=') else {
        return false;
    };

    value.contains('\n') && (value.trim_start().starts_with('{') || value.contains('|'))
}

fn is_screaming_snake(name: &str) -> bool {
    let mut has_letter = false;

    for byte in name.bytes() {
        if byte.is_ascii_lowercase() || !(byte.is_ascii_alphanumeric() || byte == b'_') {
            return false;
        }

        has_letter |= byte.is_ascii_uppercase();
    }

    has_letter
}

fn is_guard(statement: &Stmt, source: &str, tokens: &[Tok]) -> bool {
    let Stmt::If(statement) = statement else {
        return false;
    };

    statement.else_block.is_none()
        && !statement.branches.is_empty()
        && statement
            .branches
            .iter()
            .all(|(_, block)| block_terminates(block, source, tokens))
}

fn block_terminates(block: &Block, source: &str, tokens: &[Tok]) -> bool {
    let Some(statement) = block
        .stmts
        .iter()
        .rev()
        .find(|statement| !matches!(statement, Stmt::Empty(_)))
    else {
        return false;
    };

    match statement {
        Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_) => true,
        Stmt::Call(expression, _) => is_error_call(expression, source, tokens),
        _ => false,
    }
}

fn is_error_call(expression: &Expr, source: &str, tokens: &[Tok]) -> bool {
    let Expr::Call { func, .. } = expression else {
        return false;
    };
    let Expr::Name(name) = func.as_ref() else {
        return false;
    };

    span_text(*name, source, tokens) == "error"
}
