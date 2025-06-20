// Nautilus
// Copyright (C) 2024  Daniel Teuchert, Cornelius Aschermann, Sergej Schumilo

use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use grammartec::chunkstore::ChunkStoreWrapper;
use grammartec::context::Context;
use grammartec::mutator::Mutator;
use grammartec::tree::{TreeLike, TreeMutation};

use config::Config;
use forksrv::newtypes::SubprocessError;
use fuzzer::{ExecutionReason, Fuzzer};
use queue::QueueItem;

pub struct FuzzingState {
    pub cks: Arc<ChunkStoreWrapper>,
    pub ctx: Context,
    pub config: Config,
    pub fuzzer: Fuzzer,
    pub mutator: Mutator,
}

impl FuzzingState {
    pub fn new(fuzzer: Fuzzer, config: Config, cks: Arc<ChunkStoreWrapper>) -> Self {
        let ctx = Context::new();
        let mutator = Mutator::new(&ctx);
        return FuzzingState {
            cks,
            ctx,
            config,
            fuzzer,
            mutator,
        };
    }

    //Return value indicates if minimization is complete: true: complete, false: not complete
    pub fn minimize(
        &mut self,
        input: &mut QueueItem,
        start_index: usize,
        end_index: usize,
    ) -> Result<bool, SubprocessError> {
        let ctx = &mut self.ctx;
        let fuzzer = &mut self.fuzzer;

        let min_simple = self.mutator.minimize_tree(
            &mut input.tree,
            &input.fresh_bits,
            ctx,
            start_index,
            end_index,
            &mut |t: &TreeMutation, fresh_bits: &HashSet<usize>, ctx: &Context| {
                let res = fuzzer.has_bits(t, fresh_bits, ExecutionReason::Min, ctx)?;
                Ok(res)
            },
        )?;

        let min_rec = self.mutator.minimize_rec(
            &mut input.tree,
            &input.fresh_bits,
            ctx,
            start_index,
            end_index,
            &mut |t: &TreeMutation, fresh_bits: &HashSet<usize>, ctx: &Context| {
                let res = fuzzer.has_bits(t, fresh_bits, ExecutionReason::MinRec, ctx)?;
                Ok(res)
            },
        )?;

        if min_simple && min_rec {
            //Only do this when minimization is completely done
            let now = Instant::now();
            while self
                .cks
                .is_locked
                .compare_and_swap(false, true, Ordering::Acquire)
            {
                if now.elapsed().as_secs() > 30 {
                    panic!("minimize starved!");
                }
            }
            self.cks
                .chunkstore
                .write()
                .expect("RAND_1217841466")
                .add_tree(input.tree.clone(), &ctx);
            self.cks.is_locked.store(false, Ordering::Release);

            input.recursions = input.tree.calc_recursions(ctx);
            //let timestamp = start_time.elapsed();
            //Update file corresponding to this entry
            let mut file = File::create(format!(
                "{}/outputs/queue/id:{:06},time:{},er:{:?}.min", //TODO FIX PATH TO WORKDIR
                &self.config.path_to_workdir, input.id, input.timestamp, input.exitreason
            ))
            .expect("Could not create queue entry, are you sure $workdir/outputs exists?");
            input.tree.unparse_to(&ctx, &mut file);

            // Delete the original file.
            match fs::remove_file(format!(
                "{}/outputs/queue/id:{:06},time:{},er:{:?}",
                &self.config.path_to_workdir, input.id, input.timestamp, input.exitreason
            )) {
                Err(ref err) if err.kind() != ErrorKind::NotFound => {
                    println!("Error while deleting file: {}", err);
                }
                _ => {}
            }

            return Ok(true);
        }

        return Ok(false);
    }

    pub fn deterministic_tree_mutation(
        &mut self,
        input: &mut QueueItem,
        start_index: usize,
        end_index: usize,
    ) -> Result<bool, SubprocessError> {
        let ctx = &mut self.ctx;
        let fuzzer = &mut self.fuzzer;
        let done = self.mutator.mut_rules(
            &input.tree,
            ctx,
            start_index,
            end_index,
            &mut |t: &TreeMutation, ctx: &Context| {
                fuzzer
                    .run_on_with_dedup(t, ExecutionReason::Det, ctx)
                    .map(|_| ())
            },
        )?;
        return Ok(done);
    }

    pub fn havoc(&mut self, input: &mut QueueItem) -> Result<(), SubprocessError> {
        let ctx = &mut self.ctx;
        let fuzzer = &mut self.fuzzer;
        for _i in 0..100 {
            self.mutator
                .mut_random(&input.tree, ctx, &mut |t: &TreeMutation, ctx: &Context| {
                    fuzzer
                        .run_on_with_dedup(t, ExecutionReason::Havoc, ctx)
                        .map(|_| ())
                })?;
        }
        return Ok(());
    }

    pub fn havoc_recursion(&mut self, input: &mut QueueItem) -> Result<(), SubprocessError> {
        if let Some(ref mut recursions) = input.recursions
        /* input.tree.calc_recursions() */
        {
            for _i in 0..20 {
                let ctx = &mut self.ctx;
                let fuzzer = &mut self.fuzzer;
                self.mutator.mut_random_recursion(
                    &input.tree,
                    recursions,
                    ctx,
                    &mut |t: &TreeMutation, ctx: &Context| {
                        fuzzer
                            .run_on_with_dedup(t, ExecutionReason::HavocRec, ctx)
                            .map(|_| ())
                    },
                )?;
            }
        }
        return Ok(());
    }

    pub fn splice(&mut self, input: &mut QueueItem) -> Result<(), SubprocessError> {
        let ctx = &mut self.ctx;
        let fuzzer = &mut self.fuzzer;
        for _i in 0..100 {
            let now = Instant::now();
            while self.cks.is_locked.load(Ordering::SeqCst) {
                if now.elapsed().as_secs() > 30 {
                    panic!("splice starved!");
                }
            }
            self.mutator.mut_splice(
                &input.tree,
                ctx,
                &*self.cks.chunkstore.read().expect("RAND_1290117799"),
                &mut |t: &TreeMutation, ctx: &Context| {
                    fuzzer
                        .run_on_with_dedup(t, ExecutionReason::Splice, ctx)
                        .map(|_| ())
                },
            )?;
        }
        return Ok(());
    }

    pub fn generate_random(&mut self, nt: &str) -> Result<(), SubprocessError> {
        let nonterm = self.ctx.nt_id(nt);
        let len = self.ctx.get_random_len_for_nt(&nonterm);
        let tree = self.ctx.generate_tree_from_nt(nonterm, len);
        self.fuzzer
            .run_on_with_dedup(&tree, ExecutionReason::Gen, &mut self.ctx)?;
        return Ok(());
    }
    #[allow(dead_code)]
    pub fn inspect(&self, input: &QueueItem) -> String {
        return String::from_utf8_lossy(&input.tree.unparse_to_vec(&self.ctx)).into_owned();
    }
}
