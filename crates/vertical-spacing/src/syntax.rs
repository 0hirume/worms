use larvae::syntax::ast::{Block, CallArgs, Expr, IndexKey, Stmt, TableField, TokSpan};
use larvae::syntax::lexer::{Lexed, Tok};

use crate::classification::{Gap, gap_between};

#[derive(Clone, Copy)]
pub struct Boundary {
    pub start: u32,
    pub end: u32,
    pub gap: Gap,
}

pub fn collect_boundaries(
    block: &Block,
    source: &str,
    lexed: &Lexed,
    is_top: bool,
    boundaries: &mut Vec<Boundary>,
) {
    let statements: Vec<&Stmt> = block
        .stmts
        .iter()
        .filter(|statement| !matches!(statement, Stmt::Empty(_)))
        .collect();

    for (index, pair) in statements.windows(2).enumerate() {
        let previous = pair[0];
        let next = pair[1];
        let following = statements.get(index + 2).copied();
        let gap = gap_between(previous, next, following, source, &lexed.toks, is_top);

        if let Some(gap) = gap
            && let (Some((_, start)), Some((end, _))) = (
                byte_span(previous.span(), &lexed.toks),
                byte_span(next.span(), &lexed.toks),
            )
        {
            boundaries.push(Boundary { start, end, gap });
        }
    }

    for statement in statements {
        collect_nested(statement, source, lexed, boundaries);
    }
}

fn collect_nested(statement: &Stmt, source: &str, lexed: &Lexed, boundaries: &mut Vec<Boundary>) {
    match statement {
        Stmt::Local(local) => {
            for value in &local.values {
                collect_expression(value, source, lexed, boundaries);
            }
        }
        Stmt::Assign(assign) => {
            for expression in assign.targets.iter().chain(&assign.values) {
                collect_expression(expression, source, lexed, boundaries);
            }
        }
        Stmt::Call(expression, _) => collect_expression(expression, source, lexed, boundaries),
        Stmt::Do(statement) => {
            collect_boundaries(&statement.block, source, lexed, false, boundaries);
        }
        Stmt::While(statement) => {
            collect_expression(&statement.cond, source, lexed, boundaries);
            collect_boundaries(&statement.block, source, lexed, false, boundaries);
        }
        Stmt::Repeat(statement) => {
            collect_boundaries(&statement.block, source, lexed, false, boundaries);
            collect_expression(&statement.cond, source, lexed, boundaries);
        }
        Stmt::If(statement) => {
            for (condition, block) in &statement.branches {
                collect_expression(condition, source, lexed, boundaries);
                collect_boundaries(block, source, lexed, false, boundaries);
            }

            if let Some(block) = &statement.else_block {
                collect_boundaries(block, source, lexed, false, boundaries);
            }
        }
        Stmt::NumericFor(statement) => {
            collect_expression(&statement.start, source, lexed, boundaries);
            collect_expression(&statement.limit, source, lexed, boundaries);

            if let Some(step) = &statement.step {
                collect_expression(step, source, lexed, boundaries);
            }

            collect_boundaries(&statement.block, source, lexed, false, boundaries);
        }
        Stmt::GenericFor(statement) => {
            for expression in &statement.exprs {
                collect_expression(expression, source, lexed, boundaries);
            }

            collect_boundaries(&statement.block, source, lexed, false, boundaries);
        }
        Stmt::Function(statement) => {
            collect_boundaries(&statement.body.block, source, lexed, false, boundaries);
        }
        Stmt::LocalFunction(statement) => {
            collect_boundaries(&statement.body.block, source, lexed, false, boundaries);
        }
        Stmt::Return(statement) => {
            for value in &statement.values {
                collect_expression(value, source, lexed, boundaries);
            }
        }
        Stmt::Class(class) => {
            for member in &class.members {
                if let larvae::syntax::ast::ClassMember::Method(method) = member {
                    collect_boundaries(&method.body.block, source, lexed, false, boundaries);
                }
            }
        }
        Stmt::Empty(_)
        | Stmt::Break(_)
        | Stmt::Continue(_)
        | Stmt::TypeAlias(_)
        | Stmt::Declare(_) => {}
    }
}

fn collect_expression(
    expression: &Expr,
    source: &str,
    lexed: &Lexed,
    boundaries: &mut Vec<Boundary>,
) {
    match expression {
        Expr::Function { body, .. } => {
            collect_boundaries(&body.block, source, lexed, false, boundaries);
        }
        Expr::Table { fields, .. } => {
            for field in fields {
                match field {
                    TableField::Positional(value) | TableField::Named { value, .. } => {
                        collect_expression(value, source, lexed, boundaries);
                    }
                    TableField::Computed { key, value } => {
                        collect_expression(key, source, lexed, boundaries);
                        collect_expression(value, source, lexed, boundaries);
                    }
                }
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            collect_expression(lhs, source, lexed, boundaries);
            collect_expression(rhs, source, lexed, boundaries);
        }
        Expr::Unary { operand, .. } => {
            collect_expression(operand, source, lexed, boundaries);
        }
        Expr::Paren { inner, .. } => collect_expression(inner, source, lexed, boundaries),
        Expr::Index { object, key, .. } => {
            collect_expression(object, source, lexed, boundaries);

            if let IndexKey::Computed(key) = key {
                collect_expression(key, source, lexed, boundaries);
            }
        }
        Expr::Call { func, args, .. } => {
            collect_expression(func, source, lexed, boundaries);

            match args {
                CallArgs::Paren(arguments) => {
                    for argument in arguments {
                        collect_expression(argument, source, lexed, boundaries);
                    }
                }
                CallArgs::Table(table) => collect_expression(table, source, lexed, boundaries),
                CallArgs::Str(_) => {}
            }
        }
        Expr::IfElse {
            branches,
            else_value,
            ..
        } => {
            for (condition, value) in branches {
                collect_expression(condition, source, lexed, boundaries);
                collect_expression(value, source, lexed, boundaries);
            }

            collect_expression(else_value, source, lexed, boundaries);
        }
        Expr::TypeAssert { expr, .. } => collect_expression(expr, source, lexed, boundaries),
        Expr::Nil(_)
        | Expr::True(_)
        | Expr::False(_)
        | Expr::Vararg(_)
        | Expr::Number(_)
        | Expr::String(_)
        | Expr::InterpString(_)
        | Expr::Name(_) => {}
    }
}

pub fn byte_span(span: TokSpan, tokens: &[Tok]) -> Option<(u32, u32)> {
    if span.is_empty() {
        return None;
    }

    let start = tokens.get(span.start as usize)?.start;
    let end = tokens.get(span.end.checked_sub(1)? as usize)?.end;

    Some((start, end))
}

pub fn span_text<'a>(span: TokSpan, source: &'a str, tokens: &[Tok]) -> &'a str {
    let Some((start, end)) = byte_span(span, tokens) else {
        return "";
    };

    &source[start as usize..end as usize]
}
