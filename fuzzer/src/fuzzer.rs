// Nautilus
// Copyright (C) 2024  Daniel Teuchert, Cornelius Aschermann, Sergej Schumilo

extern crate time as othertime;
use othertime::strftime;

use std::collections::HashSet;
use std::collections::VecDeque;
use std::fs::File;
use std::io::stdout;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use forksrv::exitreason::ExitReason;
use forksrv::newtypes::SubprocessError;
use forksrv::ForkServer;
use grammartec::context::Context;
use grammartec::tree::TreeLike;
use shared_state::GlobalSharedState;

pub enum ExecutionReason {
    Havoc,
    HavocRec,
    Min,
    MinRec,
    Splice,
    Det,
    Gen,
}

pub struct Fuzzer {
    forksrv: ForkServer,
    last_tried_inputs: HashSet<Vec<u8>>,
    last_inputs_ring_buffer: VecDeque<Vec<u8>>,
    pub global_state: Arc<Mutex<GlobalSharedState>>,
    pub target_path: String,
    pub target_args: Vec<String>,
    pub execution_count: u64,
    pub average_executions_per_sec: f32,
    pub bits_found_by_havoc: u64,
    pub bits_found_by_havoc_rec: u64,
    pub bits_found_by_min: u64,
    pub bits_found_by_min_rec: u64,
    pub bits_found_by_splice: u64,
    pub bits_found_by_det: u64,
    pub bits_found_by_det_afl: u64,
    pub bits_found_by_gen: u64,
    pub asan_found_by_havoc: u64,
    pub asan_found_by_havoc_rec: u64,
    pub asan_found_by_min: u64,
    pub asan_found_by_min_rec: u64,
    pub asan_found_by_splice: u64,
    pub asan_found_by_det: u64,
    pub asan_found_by_det_afl: u64,
    pub asan_found_by_gen: u64,
    work_dir: String,
}

impl Fuzzer {
    pub fn new(
        path: String,
        args: Vec<String>,
        global_state: Arc<Mutex<GlobalSharedState>>,
        work_dir: String,
        timeout_in_millis: u64,
        bitmap_size: usize,
    ) -> Result<Self, SubprocessError> {
        let fs = ForkServer::new(
            path.clone(),
            args.clone(),
            true,
            timeout_in_millis,
            bitmap_size,
        );
        return Ok(Fuzzer {
            forksrv: fs,
            last_tried_inputs: HashSet::new(),
            last_inputs_ring_buffer: VecDeque::new(),
            global_state,
            target_path: path,
            target_args: args,
            execution_count: 0,
            average_executions_per_sec: 0.0,
            bits_found_by_havoc: 0,
            bits_found_by_havoc_rec: 0,
            bits_found_by_min: 0,
            bits_found_by_min_rec: 0,
            bits_found_by_splice: 0,
            bits_found_by_det: 0,
            bits_found_by_det_afl: 0,
            bits_found_by_gen: 0,
            asan_found_by_havoc: 0,
            asan_found_by_havoc_rec: 0,
            asan_found_by_min: 0,
            asan_found_by_min_rec: 0,
            asan_found_by_splice: 0,
            asan_found_by_det: 0,
            asan_found_by_det_afl: 0,
            asan_found_by_gen: 0,
            work_dir: work_dir,
        });
    }

    pub fn run_on_with_dedup<T: TreeLike>(
        &mut self,
        tree: &T,
        exec_reason: ExecutionReason,
        ctx: &Context,
    ) -> Result<bool, SubprocessError> {
        let code: Vec<u8> = tree.unparse_to_vec(ctx);
        if self.input_is_known(&code) {
            return Ok(false);
        }
        self.run_on(&code, tree, exec_reason, ctx)?;
        return Ok(true);
    }

    pub fn run_on_without_dedup<T: TreeLike>(
        &mut self,
        tree: &T,
        exec_reason: ExecutionReason,
        ctx: &Context,
    ) -> Result<(), SubprocessError> {
        let code = tree.unparse_to_vec(ctx);
        return self.run_on(&code, tree, exec_reason, ctx);
    }

    fn run_on<T: TreeLike>(
        &mut self,
        code: &Vec<u8>,
        tree: &T,
        exec_reason: ExecutionReason,
        ctx: &Context,
    ) -> Result<(), SubprocessError> {
        let (new_bits, term_sig) = self.exec(code, tree, ctx)?;
        match term_sig {
            ExitReason::Normal(223) => {
                if new_bits.is_some() {
                    //ASAN
                    self.global_state
                        .lock()
                        .expect("RAND_3390206382")
                        .total_found_asan += 1;
                    self.global_state
                        .lock()
                        .expect("RAND_202860771")
                        .last_found_asan = strftime("[%Y-%m-%d] %H:%M:%S", &othertime::now())
                        .expect("RAND_2888070412");
                    let mut file = File::create(format!(
                        "{}/outputs/signaled/ASAN_{:09}_{}",
                        self.work_dir,
                        self.execution_count,
                        thread::current().name().expect("RAND_4086695190")
                    ))
                    .expect("RAND_3096222153");
                    tree.unparse_to(ctx, &mut file);
                }
            }
            ExitReason::Normal(_) => {
                if new_bits.is_some() {
                    match exec_reason {
                        ExecutionReason::Havoc => {
                            self.bits_found_by_havoc += 1; /*print!("Havoc+")*/
                        }
                        ExecutionReason::HavocRec => {
                            self.bits_found_by_havoc_rec += 1; /*print!("HavocRec+")*/
                        }
                        ExecutionReason::Min => {
                            self.bits_found_by_min += 1; /*print!("Min+")*/
                        }
                        ExecutionReason::MinRec => {
                            self.bits_found_by_min_rec += 1; /*print!("MinRec+")*/
                        }
                        ExecutionReason::Splice => {
                            self.bits_found_by_splice += 1; /*print!("Splice+")*/
                        }
                        ExecutionReason::Det => {
                            self.bits_found_by_det += 1; /*print!("Det+")*/
                        }
                        ExecutionReason::Gen => {
                            self.bits_found_by_gen += 1; /*print!("Gen+")*/
                        }
                    }
                }
            }
            ExitReason::Timeouted => {
                self.global_state
                    .lock()
                    .expect("RAND_1706238230")
                    .last_timeout =
                    strftime("[%Y-%m-%d] %H:%M:%S", &othertime::now()).expect("RAND_1894162412");
                let mut file = File::create(format!(
                    "{}/outputs/timeout/{:09}",
                    self.work_dir, self.execution_count
                ))
                .expect("RAND_452993103");
                tree.unparse_to(ctx, &mut file);
            }
            ExitReason::Signaled(sig) => {
                if new_bits.is_some() {
                    self.global_state
                        .lock()
                        .expect("RAND_1858328446")
                        .total_found_sig += 1;
                    self.global_state
                        .lock()
                        .expect("RAND_4287051369")
                        .last_found_sig =
                        strftime("[%Y-%m-%d] %H:%M:%S", &othertime::now()).expect("RAND_76391000");
                    let mut file = File::create(format!(
                        "{}/outputs/signaled/{:?}_{:09}",
                        self.work_dir, sig, self.execution_count
                    ))
                    .expect("RAND_3690294970");
                    tree.unparse_to(ctx, &mut file);
                }
            }
            ExitReason::Stopped(_sig) => {}
        }
        stdout().flush().expect("RAND_2937475131");
        return Ok(());
    }

    pub fn has_bits<T: TreeLike>(
        &mut self,
        tree: &T,
        bits: &HashSet<usize>,
        exec_reason: ExecutionReason,
        ctx: &Context,
    ) -> Result<bool, SubprocessError> {
        self.run_on_without_dedup(tree, exec_reason, ctx)?;
        let run_bitmap = self.forksrv.get_shared();
        let mut found_all = true;
        for bit in bits.iter() {
            if run_bitmap[*bit] == 0 {
                //TODO: handle edge counts properly
                found_all = false;
            }
        }
        return Ok(found_all);
    }

    pub fn exec_raw<'a>(&'a mut self, code: &[u8]) -> Result<(ExitReason, u32), SubprocessError> {
        self.execution_count += 1;

        let start = Instant::now();

        let exitreason = self.forksrv.run(&code)?;

        let execution_time = start.elapsed().subsec_nanos();

        self.average_executions_per_sec = self.average_executions_per_sec * 0.9
            + ((1.0 / (execution_time as f32)) * 1000000000.0) * 0.1;

        return Ok((exitreason, execution_time));
    }

    fn input_is_known(&mut self, code: &[u8]) -> bool {
        if self.last_tried_inputs.contains(code) {
            return true;
        } else {
            self.last_tried_inputs.insert(code.to_vec());
            if self.last_inputs_ring_buffer.len() == 10000 {
                self.last_tried_inputs.remove(
                    &self
                        .last_inputs_ring_buffer
                        .pop_back()
                        .expect("No entry in last_inputs_ringbuffer"),
                );
            }
            self.last_inputs_ring_buffer.push_front(code.to_vec());
        }
        return false;
    }

    fn exec<T: TreeLike>(
        &mut self,
        code: &[u8],
        tree_like: &T,
        ctx: &Context,
    ) -> Result<(Option<Vec<usize>>, ExitReason), SubprocessError> {
        let (exitreason, execution_time) = self.exec_raw(&code)?;

        let is_crash = match exitreason {
            ExitReason::Normal(223) => true,
            ExitReason::Signaled(_) => true,
            _ => false,
        };

        let mut final_bits = None;
        if let Some(mut new_bits) = self.new_bits(is_crash) {
            //Only if not Timeout
            if exitreason != ExitReason::Timeouted {
                //Check for non deterministic bits
                let old_bitmap: Vec<u8> = self.forksrv.get_shared().to_vec();
                self.check_deterministic_behaviour(&old_bitmap, &mut new_bits, &code)?;
                if new_bits.len() > 0 {
                    let start_time: Instant = self.global_state.lock().unwrap().start_time;
                    final_bits = Some(new_bits);
                    let tree = tree_like.to_tree(ctx);
                    self.global_state
                        .lock()
                        .expect("RAND_2835014626")
                        .queue
                        .add(tree, old_bitmap, exitreason, ctx, execution_time, start_time);
                    //println!("Entry added to queue! New bits: {:?}", bits.clone().expect("RAND_2243482569"));
                }
            }
        }
        return Ok((final_bits, exitreason));
    }

    fn check_deterministic_behaviour(
        &mut self,
        old_bitmap: &[u8],
        new_bits: &mut Vec<usize>,
        code: &[u8],
    ) -> Result<(), SubprocessError> {
        for _ in 0..5 {
            let (_, _) = self.exec_raw(code)?;
            let run_bitmap = self.forksrv.get_shared();
            for (i, &v) in old_bitmap.iter().enumerate() {
                if run_bitmap[i] != v {
                    //println!("found fucky bit {}", i);
                }
            }
            new_bits.retain(|&i| run_bitmap[i] != 0);
        }
        return Ok(());
    }

    pub fn new_bits(&mut self, is_crash: bool) -> Option<Vec<usize>> {
        let mut res = vec![];
        let run_bitmap = self.forksrv.get_shared();
        let mut gstate_lock = self.global_state.lock().expect("RAND_2040280272");
        let shared_bitmap = gstate_lock
            .bitmaps
            .get_mut(&is_crash)
            .expect("Bitmap missing! Maybe shared state was not initialized correctly?");

        for (i, elem) in shared_bitmap.iter_mut().enumerate() {
            if (run_bitmap[i] != 0) && (*elem == 0) {
                *elem |= run_bitmap[i];
                res.push(i);
                //println!("Added new bit to bitmap. Is Crash: {:?}; Added bit: {:?}", is_crash, i);
            }
        }

        if res.len() > 0 {
            //print!("New path found:\nNew bits: {:?}\n", res);
            return Some(res);
        }
        return None;
    }
}
