use std::collections::HashMap;

use oxc::{
    allocator::Allocator,
    ast::{
        ast::{
            AssignmentOperator, AssignmentTarget, BinaryExpression, BinaryOperator,
            BindingPatternKind, Expression, ForStatement, ForStatementInit, NumberBase, Statement,
            TSType, UpdateOperator,
        },
        visit::{walk, walk_mut},
        AstBuilder, Visit, VisitMut,
    },
    semantic::{Semantic, SymbolId},
    span::SPAN,
};

pub struct RunningModuloOptimization<'a> {
    semantic: &'a Semantic<'a>,
    ast: AstBuilder<'a>,
    replacements: HashMap<(String, String), String>,
}

impl<'a> RunningModuloOptimization<'a> {
    pub fn new(semantic: &'a Semantic<'a>, allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        Self {
            semantic,
            ast,
            replacements: HashMap::new(),
        }
    }
}

impl<'a> VisitMut<'a> for RunningModuloOptimization<'a> {
    fn visit_for_statement(&mut self, for_: &mut ForStatement<'a>) {
        let mut simple_incr_variable: Option<SymbolId> = None;
        if let Some(init) = &for_.init {
            if let ForStatementInit::VariableDeclaration(var) = init {
                if let Some(init) = &var.declarations.first() {
                    if let BindingPatternKind::BindingIdentifier(x) = &init.id.kind {
                        if let Some(symbol_id) = x.symbol_id.get() {
                            simple_incr_variable = Some(symbol_id.clone());
                        }
                    }
                }
            };
        }

        if let Some(simple_incr_variable) = simple_incr_variable {
            let mut find_modulo_var =
                FindModuloVar::new(self.semantic.symbols().get_name(simple_incr_variable));
            find_modulo_var.visit_for_statement(for_);

            if let Some(denominator) = find_modulo_var.denominator {
                let modulo_var_name = format!(
                    "{}_modulo",
                    self.semantic.symbols().get_name(simple_incr_variable)
                );

                self.replacements.insert(
                    (
                        self.semantic
                            .symbols()
                            .get_name(simple_incr_variable)
                            .to_string(),
                        denominator.clone(),
                    ),
                    modulo_var_name.clone(),
                );

                if let Some(ForStatementInit::VariableDeclaration(init)) = &mut for_.init {
                    init.declarations.push(
                        self.ast.variable_declarator(
                            SPAN,
                            oxc::ast::ast::VariableDeclarationKind::Let,
                            self.ast.binding_pattern(
                                oxc::ast::ast::BindingPatternKind::BindingIdentifier(
                                    self.ast
                                        .alloc_binding_identifier(SPAN, modulo_var_name.clone()),
                                ),
                                Some(self.ast.ts_type_annotation(
                                    SPAN,
                                    TSType::TSNumberKeyword(self.ast.alloc_ts_number_keyword(SPAN)),
                                )),
                                false,
                            ),
                            Some(Expression::NumericLiteral(self.ast.alloc_numeric_literal(
                                SPAN,
                                0.0f64,
                                None,
                                NumberBase::Decimal,
                            ))),
                            false,
                        ),
                    );
                }

                if let Statement::BlockStatement(block) = &mut for_.body {
                    // modulo_var_name++;
                    block.body.push(Statement::ExpressionStatement(
                        self.ast.alloc_expression_statement(
                            SPAN,
                            Expression::UpdateExpression(self.ast.alloc_update_expression(
                                SPAN,
                                UpdateOperator::Increment,
                                false,
                                self.ast.simple_assignment_target_identifier_reference(
                                    SPAN,
                                    modulo_var_name.clone(),
                                ),
                            )),
                        ),
                    ));

                    // if (modulo_var_name == n) modulo_var_name = 0;
                    block.body.push(Statement::IfStatement(
                        self.ast.alloc_if_statement(
                            SPAN,
                            Expression::BinaryExpression(
                                self.ast.alloc_binary_expression(
                                    SPAN,
                                    Expression::Identifier(
                                        self.ast.alloc_identifier_reference(
                                            SPAN,
                                            modulo_var_name.clone(),
                                        ),
                                    ),
                                    BinaryOperator::Equality,
                                    Expression::Identifier(
                                        self.ast
                                            .alloc_identifier_reference(SPAN, denominator.clone()),
                                    ),
                                ),
                            ),
                            Statement::ExpressionStatement(self.ast.alloc_expression_statement(
                                SPAN,
                                Expression::AssignmentExpression(
                                    self.ast.alloc_assignment_expression(
                                        SPAN,
                                        AssignmentOperator::Assign,
                                        AssignmentTarget::AssignmentTargetIdentifier(
                                            self.ast.alloc_identifier_reference(
                                                SPAN,
                                                modulo_var_name.clone(),
                                            ),
                                        ),
                                        Expression::NumericLiteral(self.ast.alloc_numeric_literal(
                                            SPAN,
                                            0.0f64,
                                            None,
                                            NumberBase::Decimal,
                                        )),
                                    ),
                                ),
                            )),
                            None,
                        ),
                    ));
                }
            }
        }

        walk_mut::walk_for_statement(self, for_);
    }

    fn visit_expression(&mut self, expr_base: &mut Expression<'a>) {
        if let Expression::BinaryExpression(expr) = expr_base {
            if let BinaryOperator::Remainder = expr.operator {
                if let Expression::Identifier(left) = &expr.left {
                    if let Expression::Identifier(right) = &expr.right {
                        if let Some(replacement) = self.replacements.get(&(
                            left.name.as_str().to_string(),
                            right.name.as_str().to_string(),
                        )) {
                            *expr_base = Expression::Identifier(
                                self.ast
                                    .alloc_identifier_reference(SPAN, replacement.clone()),
                            );
                        }
                    }
                }
            }
        }

        walk_mut::walk_expression(self, expr_base);
    }
}

struct FindModuloVar {
    numerator: String,
    denominator: Option<String>,
}

impl FindModuloVar {
    fn new(numerator: &str) -> Self {
        Self {
            numerator: numerator.to_string(),
            denominator: None,
        }
    }
}

impl<'a> Visit<'a> for FindModuloVar {
    fn visit_binary_expression(&mut self, expr: &BinaryExpression<'a>) {
        if let BinaryOperator::Remainder = expr.operator {
            let mut good = false;
            if let Expression::Identifier(id) = &expr.left {
                if id.name == self.numerator {
                    good = true;
                }
            }

            if !good {
                return;
            }

            if let Expression::Identifier(id) = &expr.right {
                self.denominator = Some(id.name.as_str().to_string());
            };
        }

        walk::walk_binary_expression(self, expr);
    }
}
