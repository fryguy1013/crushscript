use oxc::{
    allocator::Allocator,
    ast::{
        ast::{
            BinaryOperator, Expression, ForStatement, ForStatementInit, MemberExpression, Program,
        },
        visit::{walk, walk_mut},
        AstBuilder, Visit, VisitMut,
    },
    span::SPAN,
};

pub struct SlottedArrayReadOptimization<'a> {
    ast: AstBuilder<'a>,
    replacements: Vec<SlottedReadCandidate>,
}

impl<'a> SlottedArrayReadOptimization<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        Self {
            ast,
            replacements: Vec::new(),
        }
    }
}

impl<'a> VisitMut<'a> for SlottedArrayReadOptimization<'a> {
    fn visit_program(&mut self, program: &mut Program<'a>) {
        let mut find_slotted_read = FindSlottedRead::new();
        find_slotted_read.visit_program(program);

        //println!("candidates: {:?}", find_slotted_read.candidates);

        self.replacements = find_slotted_read
            .candidates
            .iter_mut()
            .filter(|candidate| candidate.valid)
            .map(|candidate| candidate.clone())
            .collect();

        walk_mut::walk_program(self, program);
    }

    fn visit_for_statement(&mut self, for_: &mut ForStatement<'a>) {
        let mut loop_var = None;
        if let Some(init) = &for_.init {
            if let ForStatementInit::VariableDeclaration(var) = init {
                if let Some(init) = &var.declarations.first() {
                    if let oxc::ast::ast::BindingPatternKind::BindingIdentifier(x) = &init.id.kind {
                        loop_var = Some(x.name.to_string());
                    }
                }
            };
        }

        if let Some(loop_var) = loop_var {
            let c = self
                .replacements
                .iter_mut()
                .find(|candidate| candidate.loop_index == loop_var);

            if let Some(c) = c {
                if let Some(ForStatementInit::VariableDeclaration(var)) = &mut for_.init {
                    var.declarations.first_mut().unwrap().init = Some(Expression::Identifier(
                        self.ast
                            .alloc_identifier_reference(SPAN, c.read_index.clone().unwrap()),
                    ));
                }

                for_.test = Some(Expression::BinaryExpression(
                    self.ast.alloc_binary_expression(
                        SPAN,
                        Expression::Identifier(
                            self.ast.alloc_identifier_reference(SPAN, loop_var.clone()),
                        ),
                        BinaryOperator::Inequality,
                        Expression::Identifier(
                            self.ast
                                .alloc_identifier_reference(SPAN, c.read_index.clone().unwrap()),
                        ),
                    ),
                ));
            }
        }

        walk_mut::walk_for_statement(self, for_);
    }
}

#[derive(Debug, Clone)]
struct SlottedReadCandidate {
    array: String,
    loop_index: String,
    read_index: Option<String>,
    valid: bool,
}

struct FindSlottedRead {
    current_loop_vars: Vec<String>,
    candidates: Vec<SlottedReadCandidate>,
}

impl FindSlottedRead {
    fn new() -> Self {
        Self {
            current_loop_vars: vec![],
            candidates: vec![],
        }
    }
}

impl<'a> Visit<'a> for FindSlottedRead {
    fn visit_for_statement(&mut self, for_: &ForStatement<'a>) {
        let mut loop_var = None;
        if let Some(init) = &for_.init {
            if let ForStatementInit::VariableDeclaration(var) = init {
                if let Some(init) = &var.declarations.first() {
                    if let oxc::ast::ast::BindingPatternKind::BindingIdentifier(x) = &init.id.kind {
                        loop_var = Some(x.name.to_string());
                    }
                }
            };
        }

        if let Some(loop_var) = &loop_var {
            self.current_loop_vars.push(loop_var.to_string());
        }
        walk::walk_for_statement(self, for_);
        if let Some(_) = &loop_var {
            self.current_loop_vars.pop();
        }
    }

    fn visit_member_expression(&mut self, expr: &MemberExpression<'a>) {
        if let MemberExpression::ComputedMemberExpression(x) = expr {
            if let Expression::Identifier(a) = &x.object {
                if let Expression::Identifier(i) = &x.expression {
                    let c = self
                        .candidates
                        .iter_mut()
                        .find(|candidate| candidate.array == a.name.to_string());

                    if self.current_loop_vars.contains(&i.name.to_string()) {
                        if let Some(c) = c {
                            if c.loop_index != i.name.to_string() {
                                c.valid = false;
                            }
                        } else {
                            self.candidates.push(SlottedReadCandidate {
                                array: a.name.to_string(),
                                loop_index: i.name.to_string(),
                                read_index: None,
                                valid: true,
                            });
                        }
                    } else {
                        if let Some(c) = c {
                            if c.read_index.is_none() {
                                c.read_index = Some(i.name.to_string());
                            } else if *c.read_index.as_ref().unwrap() != i.name.to_string() {
                                c.valid = false;
                            }
                        }
                    }

                    //println!("expr: {}[{}]", a.name, i.name);
                }
            }
        }

        walk::walk_member_expression(self, expr);
    }
}
