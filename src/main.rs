#![allow(clippy::print_stdout)]
#[allow(unused_imports)]
use std::fs::File;
use std::process::Command;
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

static OUTPUT_FILE: &str = "tmp/out.cpp";

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

    // read source file
    let name = args.input;
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();

    // parse source code into AST
    let allocator = Allocator::default();
    let mut errors = Vec::new();

    let ParserReturn {
        mut program,
        errors: parser_errors,
        panicked,
        ..
    } = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions {
            parse_regular_expression: true,
            ..ParseOptions::default()
        })
        .parse();
    errors.extend(parser_errors);

    let SemanticBuilderReturn {
        mut semantic,
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

    // apply optimizations
    SlottedArrayReadOptimization::new(&allocator).visit_program(&mut program);
    RunningModuloOptimization::new(&semantic, &allocator).visit_program(&mut program);

    // output source code
    let mut writer = File::create(OUTPUT_FILE).unwrap();
    let mut codegen = Codegen::new(&mut writer, &semantic);
    codegen.print_program(&program);
    drop(writer);

    // build program to executable
    build_program(&args.output);

    Ok(())
}

fn build_program(output: &str) {
    let hello = Command::new("cl")
        .args([
            OUTPUT_FILE,
            "/O2",
            "/arch:SSE2",
            "/Istatic",
            "/link",
            &format!("/out:{}", output),
        ])
        .output()
        .expect("failed to execute process");
    println!("{}", String::from_utf8_lossy(&hello.stdout));
}
