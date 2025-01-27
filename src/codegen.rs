use oxc::ast::ast::{AssignmentOperator, Program};
use oxc::{
    ast::ast::{
        AssignmentTarget, BinaryOperator, BindingPatternKind, Expression, ForStatementInit,
        SimpleAssignmentTarget, Statement, UpdateOperator,
    },
    semantic::Semantic,
};

pub struct Codegen<'a, T>
where
    T: std::io::Write,
{
    writer: &'a mut T,
    semantic: &'a Semantic<'a>,
}

impl<'a, T> Codegen<'a, T>
where
    T: std::io::Write,
{
    pub fn new(writer: &'a mut T, semantic: &'a Semantic<'a>) -> Self {
        Self { writer, semantic }
    }

    fn print_operator(&mut self, op: BinaryOperator) -> Result<(), std::io::Error> {
        match op {
            BinaryOperator::Equality => write!(self.writer, "==")?,
            BinaryOperator::Inequality => write!(self.writer, "!=")?,
            BinaryOperator::LessThan => write!(self.writer, "<")?,
            BinaryOperator::LessEqualThan => write!(self.writer, "<=")?,
            BinaryOperator::GreaterThan => write!(self.writer, ">")?,
            BinaryOperator::GreaterEqualThan => write!(self.writer, ">=")?,
            BinaryOperator::In => write!(self.writer, "in")?,
            BinaryOperator::Instanceof => write!(self.writer, "instanceof")?,
            BinaryOperator::ShiftLeft => write!(self.writer, "<<")?,
            BinaryOperator::ShiftRight => write!(self.writer, ">>")?,
            BinaryOperator::ShiftRightZeroFill => write!(self.writer, ">>>")?,
            BinaryOperator::Addition => write!(self.writer, "+")?,
            BinaryOperator::Subtraction => write!(self.writer, "-")?,
            BinaryOperator::Multiplication => write!(self.writer, "*")?,
            BinaryOperator::Division => write!(self.writer, "/")?,
            BinaryOperator::Remainder => write!(self.writer, "%")?,
            BinaryOperator::BitwiseAnd => write!(self.writer, "&")?,
            BinaryOperator::BitwiseOR => write!(self.writer, "|")?,
            BinaryOperator::BitwiseXOR => write!(self.writer, "^")?,
            BinaryOperator::StrictEquality => write!(self.writer, "==")?,
            BinaryOperator::StrictInequality => write!(self.writer, "!=")?,
            BinaryOperator::Exponential => write!(self.writer, "**")?,
        }

        Ok(())
    }

    //fn print_identifier(&mut self, _identifier: IdentifierReference) {}

    fn print_expression(&mut self, node: &Expression) -> Result<(), std::io::Error> {
        match node {
            Expression::NumericLiteral(x) => {
                write!(self.writer, "{}", x.value)?;
            }
            Expression::BinaryExpression(bexp) => {
                write!(self.writer, "(")?;
                self.print_expression(&bexp.left)?;
                self.print_operator(bexp.operator)?;
                self.print_expression(&bexp.right)?;
                write!(self.writer, ")")?;
            }
            Expression::Identifier(x) => {
                write!(self.writer, "{}", x.name)?;
            }
            Expression::AssignmentExpression(x) => {
                match &x.left {
                    AssignmentTarget::ComputedMemberExpression(target) => {
                        // this is also a[i]
                        //target.object;
                        //target.expression

                        self.print_expression(&target.object)?;
                        write!(self.writer, "[")?;
                        self.print_expression(&target.expression)?;
                        write!(self.writer, "]")?;
                    }
                    AssignmentTarget::AssignmentTargetIdentifier(id) => {
                        write!(self.writer, "{}", id.name)?;
                    }
                    _ => {
                        panic!("Missing AssignmentExpression {:?}", x.left);
                    }
                }
                match x.operator {
                    AssignmentOperator::Assign => {
                        write!(self.writer, " = ")?;
                    }
                    AssignmentOperator::Addition => {
                        write!(self.writer, " += ")?;
                    }
                    _ => panic!("TODO: AssignmentOperator {:#?}", x.operator),
                }
                self.print_expression(&x.right)?;
            }
            Expression::ComputedMemberExpression(expr) => {
                self.print_expression(&expr.object)?;
                write!(self.writer, "[")?;
                self.print_expression(&expr.expression)?;
                write!(self.writer, "]")?;
            }
            Expression::CallExpression(expr) => {
                self.print_expression(&expr.callee)?;
                write!(self.writer, "(")?;
                for (i, arg) in expr.arguments.iter().enumerate() {
                    if i > 0 {
                        write!(self.writer, ", ")?;
                    }
                    self.print_expression(arg.to_expression())?;
                }
                write!(self.writer, ")")?;
            }
            Expression::StaticMemberExpression(expr) => {
                self.print_expression(&expr.object)?;
                write!(self.writer, "::")?;
                write!(self.writer, "{}", expr.property.name)?;
            }
            Expression::NewExpression(expr) => {
                //println!("{:#?}", node);
                write!(self.writer, "js_constructor_")?;
                self.print_expression(&expr.callee)?;
                write!(self.writer, "(")?;
                for (i, arg) in expr.arguments.iter().enumerate() {
                    if i > 0 {
                        write!(self.writer, ", ")?;
                    }
                    self.print_expression(arg.to_expression())?;
                }
                write!(self.writer, ")")?;
            }
            Expression::UpdateExpression(expr) => {
                match &expr.argument {
                    SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                        write!(self.writer, "{}", id.name)?;
                    }
                    _ => panic!("TODO: UpdateOperator {:#?}", expr.argument),
                }
                match expr.operator {
                    UpdateOperator::Increment => {
                        write!(self.writer, "++")?;
                    }
                    _ => panic!("TODO: UpdateOperator {:#?}", expr.operator),
                }
            }
            _ => {
                panic!("TODO: expression {:#?}", node);
            }
        };

        Ok(())
    }

    fn print_statement(&mut self, node: &Statement, indent: usize) -> Result<(), std::io::Error> {
        let indent_str = " ".repeat(indent * 4);
        let semantic = self.semantic;
        match node {
            Statement::BlockStatement(block) => {
                block
                    .body
                    .iter()
                    .for_each(|node| self.print_statement(&node, indent + 1).unwrap());
            }
            Statement::ExpressionStatement(expr) => {
                //println!("{:#?}", expr);
                //println!("expression statement");
                write!(self.writer, "{}", indent_str)?;
                self.print_expression(&expr.expression)?;
                writeln!(self.writer, ";")?;
            }
            Statement::EmptyStatement(_empty) => {
                println!("empty statement");
            }
            Statement::DebuggerStatement(_debugger) => {
                println!("debugger statement");
            }
            Statement::WithStatement(_with) => {
                println!("with statement");
            }
            Statement::ReturnStatement(_return_) => {
                println!("return statement");
            }
            Statement::LabeledStatement(_labeled) => {
                println!("labeled statement");
            }
            Statement::BreakStatement(_break_) => {
                println!("break statement");
            }
            Statement::ContinueStatement(_continue_) => {
                println!("continue statement");
            }
            Statement::IfStatement(if_) => {
                write!(self.writer, "{}if (", indent_str)?;
                self.print_expression(&if_.test)?;
                writeln!(self.writer, ") {{")?;
                self.print_statement(&if_.consequent, indent + 1)?;
                writeln!(self.writer, "{}}}", indent_str)?;
            }
            Statement::SwitchStatement(_switch) => {
                println!("switch statement");
            }
            Statement::ThrowStatement(_throw) => {
                println!("throw statement");
            }
            Statement::TryStatement(_try_) => {
                println!("try statement");
            }
            Statement::WhileStatement(_while_) => {
                println!("while statement");
            }
            Statement::DoWhileStatement(_do_while) => {
                println!("do while statement");
            }
            Statement::ForStatement(for_) => {
                writeln!(self.writer, "{}{{", indent_str)?;

                if let Some(init) = &for_.init {
                    if let ForStatementInit::VariableDeclaration(var) = init {
                        //print_statement(&init);
                        for decl in &var.declarations {
                            if let BindingPatternKind::BindingIdentifier(x) = &decl.id.kind {
                                //println!("init {:#?}", decl);
                                // if let Some(init) = x.symbol_id.get().unwrap().into() {
                                //     let name = semantic.symbols().get_name(init);
                                //     write!(self.writer, "{}auto {} = ", indent_str, name)?;
                                //     self.print_expression(decl.init.as_ref().unwrap())?;
                                //     writeln!(self.writer, ";")?;
                                // }
                                let name = x.name.as_str();
                                write!(self.writer, "{}auto {} = ", indent_str, name)?;
                                self.print_expression(decl.init.as_ref().unwrap())?;
                                writeln!(self.writer, ";")?;
                            }
                        }
                    }
                }

                write!(self.writer, "{}while (", indent_str)?;

                if let Some(test) = &for_.test {
                    self.print_expression(test)?;
                } else {
                    write!(self.writer, "true")?;
                }

                writeln!(self.writer, ") {{")?;

                //print_statement(&for_.test);
                //print_statement(&for_.update);
                self.print_statement(&for_.body, indent + 1)?;

                if let Some(update) = &for_.update {
                    self.print_expression(update)?;
                    writeln!(self.writer, ";")?;
                }

                writeln!(self.writer, "{}}}", indent_str)?;

                writeln!(self.writer, "{}}}", indent_str)?;
            }
            Statement::ForInStatement(_for_in) => {
                println!("for in statement");
            }
            Statement::ForOfStatement(_for_of) => {
                println!("for of statement");
            }
            Statement::VariableDeclaration(var) => {
                for decl in &var.declarations {
                    if let BindingPatternKind::BindingIdentifier(x) = &decl.id.kind {
                        //println!("variable declaration {:?}", x.symbol_id);
                        write!(
                            self.writer,
                            "auto {}",
                            semantic
                                .symbols()
                                .get_name(x.symbol_id.get().unwrap().into())
                        )?;
                    }
                    if let Some(init) = &decl.init {
                        write!(self.writer, " = ")?;
                        self.print_expression(init)?;
                    }
                    writeln!(self.writer, ";")?;
                }
            }
            Statement::ClassDeclaration(_class) => {
                println!("class declaration");
            }
            Statement::FunctionDeclaration(_function) => {
                println!("function declaration");
            }
            Statement::TSTypeAliasDeclaration(_type_alias) => {
                println!("ts type alias declaration");
            }
            _ => println!("other node"),
        }

        Ok(())
    }

    fn print_node(&mut self, node: &oxc::semantic::AstNode) {
        match node.kind() {
            oxc::ast::AstKind::Program(program) => {
                program
                    .body
                    .iter()
                    .for_each(|node| self.print_statement(&node, 0).unwrap());
            }
            _ => {} //println!("{:#?}", node.kind());
        }
    }

    pub fn print_program(&mut self, _program: &Program) -> Result<(), std::io::Error> {
        writeln!(self.writer, "#include <stdio.h>")?;
        writeln!(self.writer, "#include <stdlib.h>")?;
        writeln!(self.writer, "#include <math.h>")?;
        writeln!(self.writer, "#include <string.h>")?;
        writeln!(self.writer, "#include <stdbool.h>")?;
        writeln!(self.writer, "#include <stdint.h>")?;
        writeln!(self.writer, "#include \"js.h\"")?;

        writeln!(self.writer, "int main(int argc, char** argv) {{")?;
        writeln!(self.writer, "    process::setargs(argc, argv);")?;
        self.print_node(self.semantic.nodes().root_node().unwrap());

        writeln!(self.writer, "return 0;")?;
        writeln!(self.writer, "}}")?;
        writeln!(self.writer, "")?;

        Ok(())
    }
}
