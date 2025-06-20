// Nautilus
// Copyright (C) 2024  Daniel Teuchert, Cornelius Aschermann, Sergej Schumilo

extern crate forksrv;
extern crate grammartec;
extern crate serde_json;
extern crate time as othertime;
#[macro_use]
extern crate serde_derive;
extern crate clap;
extern crate pyo3;
extern crate ron;

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

use clap::{App, Arg};
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
    return Ok(());
}

fn fuzzing_thread(
    global_state: Arc<Mutex<GlobalSharedState>>,
    config: Config,
    ctx: Context,
    cks: Arc<ChunkStoreWrapper>,
) {
    let path_to_bin_target = config.path_to_bin_target.to_owned();
    let args = config.arguments.clone();

    let fuzzer = Fuzzer::new(
        path_to_bin_target.clone(),
        args,
        global_state.clone(),
        config.path_to_workdir.clone(),
        config.timeout_in_millis.clone(),
        config.bitmap_size.clone(),
    )
    .expect("RAND_3617502350");
    let mut state = FuzzingState::new(fuzzer, config.clone(), cks.clone());
    state.ctx = ctx.clone();
    let mut old_execution_count = 0;
    let mut old_executions_per_sec = 0;
    //Normal mode
    loop {
        let inp = global_state.lock().expect("RAND_2191486322").queue.pop();
        if let Some(mut inp) = inp {
            //If subprocess died restart forkserver
            if process_input(&mut state, &mut inp, &config).is_err() {
                let args = vec![];
                let fuzzer = Fuzzer::new(
                    path_to_bin_target.clone(),
                    args,
                    global_state.clone(),
                    config.path_to_workdir.clone(),
                    config.timeout_in_millis.clone(),
                    config.bitmap_size.clone(),
                )
                .expect("RAND_3077320530");
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
                        config.timeout_in_millis.clone(),
                        config.bitmap_size.clone(),
                    )
                    .expect("RAND_357619639");
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
    
    pyo3::prepare_freethreaded_python();

    //Parse parameters
    let matches = App::new("nautilus")
        .about("Grammar fuzzer")
        .setting(clap::AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("config")
                .short("c")
                .value_name("CONFIG")
                .takes_value(true)
                .help("Path to configuration file")
                .default_value("config.ron"),
        )
        .arg(
            Arg::with_name("grammar")
                .short("g")
                .takes_value(true)
                .help("Overwrite the grammar file specified in the CONFIG"),
        )
        .arg(
            Arg::with_name("workdir")
                .short("o")
                .takes_value(true)
                .help("Overwrite the workdir specified in the CONFIG"),
        )
        .arg(Arg::with_name("cmdline").multiple(true))
        .get_matches();

    let config_file_path = matches
        .value_of("config")
        .expect("the path to the configuration file has a default value");

    println!(
        "{} Starting Fuzzing...",
        othertime::now()
            .strftime("[%Y-%m-%d] %H:%M:%S")
            .expect("RAND_1939191497")
    );

    //Set Config
    let mut config_file = File::open(&config_file_path).expect("cannot read config file");
    let mut config_file_contents = String::new();
    config_file
        .read_to_string(&mut config_file_contents)
        .expect("RAND_1413661228");
    let mut config: Config =
        ron::de::from_str(&config_file_contents).expect("Failed to deserialize");

    let workdir = matches
        .value_of("workdir")
        .unwrap_or(&config.path_to_workdir)
        .to_string();
    config.path_to_workdir = workdir;

    //Check if specified workdir exists:
    if !Path::new(&config.path_to_workdir).exists() {
        panic!(
            "Specified working directory does not exist!\nGiven path: {}",
            config.path_to_workdir
        );
    }

    if let Some(mut cmdline) = matches.values_of("cmdline") {
        if cmdline.len() > 0 {
            config.path_to_bin_target = cmdline.next().unwrap().to_string();
            config.arguments = cmdline.map(|x| x.to_string()).collect();
        }
    }
    //Check if target binary exists:
    if !Path::new(&config.path_to_bin_target).exists() {
        panic!(
            "Target binary does not exist!\nGiven path: {}",
            config.path_to_bin_target
        );
    }

    let start_time = Instant::now();
    let shared = Arc::new(Mutex::new(GlobalSharedState::new(
        config.path_to_workdir.clone(),
        config.bitmap_size,
        start_time,
    )));
    let shared_chunkstore = Arc::new(ChunkStoreWrapper::new(config.path_to_workdir.clone()));

    let mut my_context;
    let grammar_path = matches
        .value_of("grammar")
        .unwrap_or(&config.path_to_grammar)
        .to_owned();

    //Check if grammar file exists:
    if !Path::new(&grammar_path).exists() {
        panic!("Grammar does not exist!\nGiven path: {}", grammar_path);
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
    for f in folders.iter() {
        fs::create_dir_all(format!("{}/{}", config.path_to_workdir, f))
            .expect("Could not create folder in workdir");
    }

    // Create an empty file named "is_main_node" in the workdir: Expected by secondary AFL instances.
    let empty_file_path = format!("{}/outputs/{}", config.path_to_workdir, "is_main_node");
    fs::File::create(empty_file_path).expect("Could not create empty file");

    //Start fuzzing threads
    let mut thread_number = 0;
    let threads = (0..config.number_of_threads).map(|_| {
        let state = shared.clone();
        let config = config.clone();
        let ctx = my_context.clone();
        let cks = shared_chunkstore.clone();
        thread_number += 1;
        thread::Builder::new()
            .name(format!("fuzzer_{}", thread_number))
            .stack_size(config.thread_size)
            .spawn(move || fuzzing_thread(state, config, ctx, cks))
    });

    //Start status thread
    let status_thread = {
        let global_state = shared.clone();
        let shared_cks = shared_chunkstore.clone();
        thread::Builder::new()
            .name("status_thread".to_string())
            .spawn(move || {
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

    for t in threads.collect::<Vec<_>>().into_iter() {
        t.expect("RAND_2698731594").join().expect("RAND_2698731594");
    }
    status_thread.join().expect("RAND_399292929");
}
