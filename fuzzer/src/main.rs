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

extern crate chrono;
extern crate clap;
extern crate forksrv;
extern crate grammartec;
extern crate pyo3;
extern crate ron;
extern crate serde;
extern crate serde_json;

mod config;
mod fuzzer;
mod python_grammar_loader;
mod queue;
mod shared_state;
mod state;

use config::Config;
use forksrv::newtypes::SubprocessError;
use fuzzer::Fuzzer;
use grammartec::chunkstore::ChunkStoreWrapper;
use grammartec::context::Context;
use queue::{InputState, QueueItem};
use shared_state::GlobalSharedState;
use state::FuzzingState;

use chrono::Local;
use clap::{Arg, Command};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{thread, time};

fn process_input(
    state: &mut FuzzingState,
    inp: &mut QueueItem,
    config: &Config,
) -> Result<(), SubprocessError> {
    match inp.state {
        InputState::Init(start_index) => {
            let end_index = start_index + 200;

            if state.minimize(inp, start_index, end_index)? {
                inp.state = InputState::Det((0, 0));
            } else {
                inp.state = InputState::Init(end_index);
            }
        }
        InputState::Det((cycle, start_index)) => {
            let end_index = start_index + 1;
            if state.deterministic_tree_mutation(inp, start_index, end_index)? {
                if cycle == config.number_of_deterministic_mutations {
                    inp.state = InputState::Random;
                } else {
                    inp.state = InputState::Det((cycle + 1, 0));
                }
            } else {
                inp.state = InputState::Det((cycle, end_index));
            }
            state.splice(inp)?;
            state.havoc(inp)?;
            state.havoc_recursion(inp)?;
        }
        InputState::Random => {
            state.splice(inp)?;
            state.havoc(inp)?;
            state.havoc_recursion(inp)?;
        }
    }
    Ok(())
}

fn fuzzing_thread(
    global_state: &Arc<Mutex<GlobalSharedState>>,
    config: &Config,
    ctx: &Context,
    cks: &Arc<ChunkStoreWrapper>,
) {
    let path_to_bin_target = config.path_to_bin_target.clone();
    let args = config.arguments.clone();

    let fuzzer = Fuzzer::new(
        path_to_bin_target.clone(),
        args,
        global_state.clone(),
        config.path_to_workdir.clone(),
        config.hide_output,
        config.timeout_in_millis,
        config.bitmap_size,
        config.extension.clone(),
    );
    let mut state = FuzzingState::new(fuzzer, config.clone(), cks.clone());
    state.ctx = ctx.clone();
    let mut old_execution_count = 0;
    let mut old_executions_per_sec = 0;
    //Normal mode
    loop {
        let inp = global_state.lock().expect("RAND_2191486322").queue.pop();
        if let Some(mut inp) = inp {
            //If subprocess died restart forkserver
            if process_input(&mut state, &mut inp, config).is_err() {
                let args = vec![];
                let fuzzer = Fuzzer::new(
                    path_to_bin_target.clone(),
                    args,
                    global_state.clone(),
                    config.path_to_workdir.clone(),
                    config.hide_output,
                    config.timeout_in_millis,
                    config.bitmap_size,
                    config.extension.clone(),
                );
                state = FuzzingState::new(fuzzer, config.clone(), cks.clone());
                state.ctx = ctx.clone();
                old_execution_count = 0;
                old_executions_per_sec = 0;
            }
            global_state
                .lock()
                .expect("RAND_788470278")
                .queue
                .finished(inp);
        } else {
            for _ in 0..config.number_of_generate_inputs {
                //If subprocess dies restart forkserver
                if state.generate_random("START").is_err() {
                    let args = vec![];
                    let fuzzer = Fuzzer::new(
                        path_to_bin_target.clone(),
                        args,
                        global_state.clone(),
                        config.path_to_workdir.clone(),
                        config.hide_output,
                        config.timeout_in_millis,
                        config.bitmap_size,
                        config.extension.clone(),
                    );
                    state = FuzzingState::new(fuzzer, config.clone(), cks.clone());
                    state.ctx = ctx.clone();
                    old_execution_count = 0;
                    old_executions_per_sec = 0;
                }
            }
            global_state
                .lock()
                .expect("RAND_2035137253")
                .queue
                .new_round();
        }
        let mut stats = global_state.lock().expect("RAND_2403514078");
        stats.execution_count += state.fuzzer.execution_count - old_execution_count;
        old_execution_count = state.fuzzer.execution_count;
        stats.average_executions_per_sec += state.fuzzer.average_executions_per_sec as u32;
        stats.average_executions_per_sec -= old_executions_per_sec;
        old_executions_per_sec = state.fuzzer.average_executions_per_sec as u32;
        if state.fuzzer.bits_found_by_havoc > 0 {
            stats.bits_found_by_havoc += state.fuzzer.bits_found_by_havoc;
            state.fuzzer.bits_found_by_havoc = 0;
        }
        if state.fuzzer.bits_found_by_gen > 0 {
            stats.bits_found_by_gen += state.fuzzer.bits_found_by_gen;
            state.fuzzer.bits_found_by_gen = 0;
        }
        if state.fuzzer.bits_found_by_min > 0 {
            stats.bits_found_by_min += state.fuzzer.bits_found_by_min;
            state.fuzzer.bits_found_by_min = 0;
        }
        if state.fuzzer.bits_found_by_det > 0 {
            stats.bits_found_by_det += state.fuzzer.bits_found_by_det;
            state.fuzzer.bits_found_by_det = 0;
        }
        if state.fuzzer.bits_found_by_splice > 0 {
            stats.bits_found_by_splice += state.fuzzer.bits_found_by_splice;
            state.fuzzer.bits_found_by_splice = 0;
        }
        if state.fuzzer.bits_found_by_havoc_rec > 0 {
            stats.bits_found_by_havoc_rec += state.fuzzer.bits_found_by_havoc_rec;
            state.fuzzer.bits_found_by_havoc_rec = 0;
        }
        if state.fuzzer.bits_found_by_min_rec > 0 {
            stats.bits_found_by_min_rec += state.fuzzer.bits_found_by_min_rec;
            state.fuzzer.bits_found_by_min_rec = 0;
        }
    }
}

fn main() {
    //Parse parameters
    let matches = Command::new("nautilus")
        .about("Grammar fuzzer")
        .arg(
            Arg::new("config")
                .short('c')
                .value_name("CONFIG")
                .action(clap::ArgAction::Set)
                .help("Path to configuration file")
                .default_value("config.ron"),
        )
        .arg(
            Arg::new("grammar")
                .short('g')
                .action(clap::ArgAction::Set)
                .help("Overwrite the grammar file specified in the CONFIG"),
        )
        .arg(
            Arg::new("workdir")
                .short('o')
                .action(clap::ArgAction::Set)
                .help("Overwrite the workdir specified in the CONFIG"),
        )
        .arg(
            Arg::new("cmdline")
                .action(clap::ArgAction::Append)
                .trailing_var_arg(true),
        )
        .get_matches();

    let config_file_path = matches
        .get_one::<String>("config")
        .expect("the path to the configuration file has a default value");

    println!(
        "{} Starting Fuzzing...",
        Local::now().format("[%Y-%m-%d] %H:%M:%S")
    );

    //Set Config
    let mut config_file = File::open(config_file_path).expect("cannot read config file");
    let mut config_file_contents = String::new();
    config_file
        .read_to_string(&mut config_file_contents)
        .expect("RAND_1413661228");
    let mut config: Config =
        ron::de::from_str(&config_file_contents).expect("Failed to deserialize");

    let workdir = matches
        .get_one("workdir")
        .unwrap_or(&config.path_to_workdir)
        .to_string();
    config.path_to_workdir = workdir;

    //Check if specified workdir exists:
    assert!(
        Path::new(&config.path_to_workdir).exists(),
        "Specified working directory does not exist!\nGiven path: {}",
        config.path_to_workdir
    );

    if let Some(mut cmdline) = matches.get_many::<String>("cmdline") {
        if cmdline.len() > 0 {
            config.path_to_bin_target = cmdline.next().unwrap().to_string();
            config.arguments = cmdline.map(std::string::ToString::to_string).collect();
        }
    }
    //Check if target binary exists:
    assert!(
        Path::new(&config.path_to_bin_target).exists(),
        "Target binary does not exist!\nGiven path: {}",
        config.path_to_bin_target
    );

    let shared = Arc::new(Mutex::new(GlobalSharedState::new(
        config.path_to_workdir.clone(),
        config.bitmap_size,
        config.extension.clone(),
    )));
    let shared_chunkstore = Arc::new(ChunkStoreWrapper::new(
        config.path_to_workdir.clone(),
        config.extension.clone(),
    ));

    let mut my_context;
    let grammar_path = matches
        .get_one::<String>("grammar")
        .unwrap_or(&config.path_to_grammar)
        .to_owned();

    //Check if grammar file exists:
    if !Path::new(&grammar_path).exists() {
        panic!("{}", "Grammar does not exist!\nGiven path: {grammar_path}");
    }

    //Generate rules using a grammar
    my_context = Context::new();
    if grammar_path.ends_with(".json") {
        let gf = File::open(grammar_path).expect("cannot read grammar file");
        let rules: Vec<Vec<String>> =
            serde_json::from_reader(&gf).expect("cannot parse grammar file");
        let root = "{".to_string() + &rules[0][0] + "}";
        my_context.add_rule("START", root.as_bytes());
        for rule in rules {
            my_context.add_rule(&rule[0], rule[1].as_bytes());
        }
    } else if grammar_path.ends_with(".py") {
        my_context = python_grammar_loader::load_python_grammar(&grammar_path);
    } else {
        panic!("Unknown grammar type");
    }

    my_context.initialize(config.max_tree_size);

    //Create output folder
    let folders = [
        "/outputs/signaled",
        "/outputs/queue",
        "/outputs/timeout",
        "/outputs/chunks",
    ];
    for f in &folders {
        fs::create_dir_all(format!("{}/{f}", config.path_to_workdir))
            .expect("Could not create folder in workdir");
    }

    //Start fuzzing threads
    let mut thread_number = 0;
    let threads = (0..config.number_of_threads).map(|_| {
        let state = shared.clone();
        let config = config.clone();
        let ctx = my_context.clone();
        let cks = shared_chunkstore.clone();
        thread_number += 1;
        thread::Builder::new()
            .name(format!("fuzzer_{thread_number}"))
            .stack_size(config.thread_size)
            .spawn(move || fuzzing_thread(&state, &config, &ctx, &cks))
    });

    //Start status thread
    let status_thread = {
        let global_state = shared.clone();
        let shared_cks = shared_chunkstore.clone();
        thread::Builder::new()
            .name("status_thread".to_string())
            .spawn(move || {
                let start_time = Instant::now();
                thread::sleep(time::Duration::from_secs(1));
                print!("{}[2J", 27 as char);
                print!("{}[H", 27 as char);
                loop {
                    let execution_count;
                    let average_executions_per_sec;
                    let queue_len;
                    let bits_found_by_gen;
                    let bits_found_by_min;
                    let bits_found_by_min_rec;
                    let bits_found_by_det;
                    let bits_found_by_splice;
                    let bits_found_by_havoc;
                    let bits_found_by_havoc_rec;
                    let last_found_asan;
                    let last_found_sig;
                    let last_timeout;
                    let total_found_asan;
                    let total_found_sig;
                    {
                        let shared_state = global_state.lock().expect("RAND_597319831");
                        execution_count = shared_state.execution_count;
                        average_executions_per_sec = shared_state.average_executions_per_sec;
                        queue_len = shared_state.queue.len();
                        bits_found_by_gen = shared_state.bits_found_by_gen;
                        bits_found_by_min = shared_state.bits_found_by_min;
                        bits_found_by_min_rec = shared_state.bits_found_by_min_rec;
                        bits_found_by_det = shared_state.bits_found_by_det;
                        bits_found_by_splice = shared_state.bits_found_by_splice;
                        bits_found_by_havoc = shared_state.bits_found_by_havoc;
                        bits_found_by_havoc_rec = shared_state.bits_found_by_havoc_rec;
                        last_found_asan = shared_state.last_found_asan.clone();
                        last_found_sig = shared_state.last_found_sig.clone();
                        last_timeout = shared_state.last_timeout.clone();
                        total_found_asan = shared_state.total_found_asan;
                        total_found_sig = shared_state.total_found_sig;
                    }
                    let secs = start_time.elapsed().as_secs();
                    let minutes = secs / 60;
                    let hours = minutes / 60;
                    let days = hours / 24;

                    print!("{}[H", 27 as char);

                    println!("         _   _             _   _ _             ");
                    println!("        | \\ | |           | | (_) |            ");
                    println!("        |  \\| | __ _ _   _| |_ _| |_   _ ___   ");
                    println!("        | . ` |/ _` | | | | __| | | | | / __|  ");
                    println!("        | |\\  | (_| | |_| | |_| | | |_| \\__ \\  ");
                    println!("        |_| \\_|\\__,_|\\__,_|\\__|_|_|\\__,_|___/  ");
                    println!("      ");

                    println!("------------------------------------------------------    ");
                    println!(
                        "Run Time: {} days, {} hours, {} minutes, {} seconds       ",
                        days,
                        hours % 24,
                        minutes % 60,
                        secs % 60
                    );
                    println!(
                        "Execution Count:          {}                              ",
                        execution_count
                    );
                    println!(
                        "Executions per Sec:       {}                              ",
                        average_executions_per_sec
                    );
                    println!(
                        "Left in queue:            {}                              ",
                        queue_len
                    );
                    let now = Instant::now();
                    while shared_cks.is_locked.load(Ordering::SeqCst) {
                        if now.elapsed().as_secs() > 30 {
                            panic!("Printing thread starved!");
                        }
                    }
                    println!(
                        "Trees in Chunkstore:      {}                              ",
                        shared_cks
                            .chunkstore
                            .read()
                            .expect("RAND_351823021")
                            .trees()
                    );
                    println!("------------------------------------------------------    ");
                    println!(
                        "Last ASAN crash:          {}                              ",
                        last_found_asan
                    );
                    println!(
                        "Last SIG crash:           {}                              ",
                        last_found_sig
                    );
                    println!(
                        "Last Timeout:             {}                              ",
                        last_timeout
                    );
                    println!(
                        "Total ASAN crashes:       {}                              ",
                        total_found_asan
                    );
                    println!(
                        "Total SIG crashes:        {}                              ",
                        total_found_sig
                    );
                    println!("------------------------------------------------------    ");
                    println!(
                        "New paths found by Gen:          {}                       ",
                        bits_found_by_gen
                    );
                    println!(
                        "New paths found by Min:          {}                       ",
                        bits_found_by_min
                    );
                    println!(
                        "New paths found by Min Rec:      {}                       ",
                        bits_found_by_min_rec
                    );
                    println!(
                        "New paths found by Det:          {}                       ",
                        bits_found_by_det
                    );
                    println!(
                        "New paths found by Splice:       {}                       ",
                        bits_found_by_splice
                    );
                    println!(
                        "New paths found by Havoc:        {}                       ",
                        bits_found_by_havoc
                    );
                    println!(
                        "New paths found by Havoc Rec:    {}                       ",
                        bits_found_by_havoc_rec
                    );
                    println!("------------------------------------------------------    ");
                    //println!("Global bitmap: {:?}", global_state.lock().expect("RAND_1887203473").bitmaps.get(&false).expect("RAND_1887203473"));
                    thread::sleep(time::Duration::from_secs(1));
                }
            })
            .expect("RAND_3541874337")
    };

    for t in threads.collect::<Vec<_>>() {
        t.expect("RAND_2698731594").join().expect("RAND_2698731594");
    }
    status_thread.join().expect("RAND_399292929");
}
