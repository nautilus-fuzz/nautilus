// Nautilus
// Copyright (C) 2020  Daniel Teuchert, Cornelius Aschermann, Sergej Schumilo

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

extern crate clap;
extern crate grammartec;
extern crate pyo3;
extern crate ron;
extern crate serde_json;

mod python_grammar_loader;
use grammartec::context::Context;
use grammartec::tree::TreeLike;

use clap::{value_parser, Arg, ArgAction, Command};
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() {
    //Parse parameters
    let matches = Command::new("generator")
        .about("Generate strings using a grammar. This can also be used to generate a corpus")
        .arg(Arg::new("grammar_path")
             .short('g')
             .value_name("GRAMMAR")
             .required(true)
             .value_parser(value_parser!(PathBuf))
             .help("Path to grammar"))
        .arg(Arg::new("tree_depth")
             .short('t')
             .value_name("DEPTH")
             .required(true)
             .value_parser(value_parser!(usize))
             .help("Size of trees that are generated"))
        .arg(Arg::new("number_of_trees")
             .short('n')
             .value_name("NUMBER")
             .value_parser(value_parser!(usize))
             .default_value("1")
             .help("Number of trees to generate"))
        .arg(Arg::new("store")
             .short('s')
             .action(ArgAction::SetTrue)
             .help("Store output to files. This will create a folder called corpus containing one file for each generated tree."))
        .arg(Arg::new("verbose")
             .short('v')
             .action(ArgAction::SetTrue)
             .help("Be verbose"))
        .get_matches();

    let grammar_path = matches
        .get_one::<PathBuf>("grammar_path")
        .expect("grammar_path is a required parameter");
    let tree_depth = *matches
        .get_one::<usize>("tree_depth")
        .expect("tree_depth is a requried parameter");
    let number_of_trees = *matches.get_one::<usize>("number_of_trees").unwrap_or(&1);
    let store = matches.get_flag("store");
    let verbose = matches.get_flag("verbose");

    let mut ctx = Context::new();
    //Create new Context and saved it
    match grammar_path.extension() {
        Some(ext) if ext == "json" => {
            let gf = File::open(grammar_path).expect("cannot read grammar file");
            let rules: Vec<Vec<String>> =
                serde_json::from_reader(&gf).expect("cannot parse grammar file");
            assert!(!rules.is_empty(), "rule file didn_t include any rules");
            let root = "{".to_string() + &rules[0][0] + "}";
            ctx.add_rule("START", root.as_bytes());
            for rule in rules {
                ctx.add_rule(&rule[0], rule[1].as_bytes());
            }
        }
        Some(ext) if ext == "py" => {
            ctx = python_grammar_loader::load_python_grammar(grammar_path.to_str().unwrap())
        }
        Some(_) | None => panic!("Unknown grammar type"),
    }

    ctx.initialize(tree_depth);

    //Generate Tree
    if store {
        if Path::new("corpus").exists() {
        } else {
            fs::create_dir("corpus").expect("Could not create corpus directory");
        }
    }
    for i in 0..number_of_trees {
        let nonterm = ctx.nt_id("START");
        let len = ctx.get_random_len_for_nt(&nonterm);
        let generated_tree = ctx.generate_tree_from_nt(nonterm, len); //1 is the index of the "START" Node
        if verbose {
            println!("Generating tree {} from {number_of_trees}", i + 1);
        }
        if store {
            let mut output =
                File::create(&format!("corpus/{}", i + 1)).expect("cannot create output file");
            generated_tree.unparse_to(&ctx, &mut output);
        } else {
            let stdout = io::stdout();
            let mut stdout_handle = stdout.lock();
            generated_tree.unparse_to(&ctx, &mut stdout_handle);
        }

        let mut of_tree = File::create("/tmp/test_tree.ron").expect("cannot create output file");
        of_tree
            .write_all(
                ron::ser::to_string(&generated_tree)
                    .expect("Serialization of Tree failed!")
                    .as_bytes(),
            )
            .expect("Writing to tree file failed");
    }
}
