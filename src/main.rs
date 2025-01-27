#![allow(clippy::print_stdout)]
#[allow(unused_imports)]
use std::fs::File;
use std::process::Command;
use std::rc::Rc;
use std::{fs, path::Path};

use clap::arg;
use clap::Parser as ClapParser;
use codegen::Codegen;
use oxc::{
    allocator::Allocator,
    ast::VisitMut,
    parser::{ParseOptions, Parser, ParserReturn},
    semantic::{SemanticBuilder, SemanticBuilderReturn},
    span::SourceType,
};
use running_modulo_optimization::RunningModuloOptimization;
use slotted_array_read_optimization::SlottedArrayReadOptimization;

mod codegen;
mod running_modulo_optimization;
mod slotted_array_read_optimization;

#[derive(ClapParser, Debug)]
struct Args {
    #[arg(required = true)]
    input: String,

    #[arg(short, long, default_value_t = String::from("out.exe"))]
    output: String,
}

#[allow(unused)]
fn main() -> Result<(), String> {
    let args = Args::parse();

    let name = args.input;
    let (source_text, source_type) = read_file(name)?;

    let allocator = Rc::new(Allocator::default());
    let mut crush = CrushScript::new(&allocator, &source_text, source_type);

    SlottedArrayReadOptimization::new(&allocator).visit_program(&mut crush.program);
    RunningModuloOptimization::new(&crush.semantic, &allocator).visit_program(&mut crush.program);

    let mut writer = File::create("tmp/out.cpp").unwrap();
    let mut codegen = Codegen::new(&mut writer, &crush.semantic);
    codegen.print_program(&crush.program);

    crush.build_program(&args.output);

    Ok(())
}

fn read_file(name: String) -> Result<(String, SourceType), String> {
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();
    Ok((source_text, source_type))
}

struct CrushScript<'a> {
    program: oxc::ast::ast::Program<'a>,
    semantic: oxc::semantic::Semantic<'a>,
}

impl<'a> CrushScript<'a> {
    fn new(allocator: &'a Allocator, source_text: &'a String, source_type: SourceType) -> Self {
        let mut errors = Vec::new();

        let ParserReturn {
            program,
            errors: parser_errors,
            panicked,
            ..
        } = Parser::new(&allocator, &source_text, source_type.clone())
            .with_options(ParseOptions {
                parse_regular_expression: true,
                ..ParseOptions::default()
            })
            .parse();
        errors.extend(parser_errors);

        let SemanticBuilderReturn {
            semantic,
            errors: semantic_errors,
        } = SemanticBuilder::new()
            .with_check_syntax_error(true)
            .with_build_jsdoc(true)
            .with_cfg(true)
            .build(&program);

        errors.extend(semantic_errors);

        if panicked {
            for error in &errors {
                eprintln!("{error:?}");
                panic!("Parsing failed");
            }
        }

        Self { program, semantic }
    }

    pub fn build_program(&self, output: &str) {
        let hello = Command::new("cl")
            .args([
                "tmp/out.cpp",
                "/O2",
                "/arch:SSE2",
                "/link",
                &format!("/out:{}", output),
            ])
            .output()
            .expect("failed to execute process");
        println!("{}", String::from_utf8_lossy(&hello.stdout));
    }
}
