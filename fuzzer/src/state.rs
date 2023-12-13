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

use std::collections::HashSet;
use std::fs::File;
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
        FuzzingState {
            cks,
            ctx,
            config,
            fuzzer,
            mutator,
        }
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
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
                .is_err()
            {
                if now.elapsed().as_secs() > 30 {
                    panic!("minimize starved!");
                }
            }
            self.cks
                .chunkstore
                .write()
                .expect("RAND_1217841466")
                .add_tree(input.tree.clone(), ctx);
            self.cks.is_locked.store(false, Ordering::Release);

            input.recursions = input.tree.calc_recursions(ctx);

            //Update file corresponding to this entry
            let mut file = File::create(format!(
                "{}/outputs/queue/id:{:09},er:{:?}.min{}", //TODO FIX PATH TO WORKDIR
                &self.config.path_to_workdir, input.id, input.exitreason, &self.config.extension
            ))
            .expect("Could not create queue entry, are you sure $workdir/outputs exists?");
            input.tree.unparse_to(ctx, &mut file);
            return Ok(true);
        }

        Ok(false)
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
        Ok(done)
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
        Ok(())
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
        Ok(())
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
                &self.cks.chunkstore.read().expect("RAND_1290117799"),
                &mut |t: &TreeMutation, ctx: &Context| {
                    fuzzer
                        .run_on_with_dedup(t, ExecutionReason::Splice, ctx)
                        .map(|_| ())
                },
            )?;
        }
        Ok(())
    }

    pub fn generate_random(&mut self, nt: &str) -> Result<(), SubprocessError> {
        let nonterm = self.ctx.nt_id(nt);
        let len = self.ctx.get_random_len_for_nt(&nonterm);
        let tree = self.ctx.generate_tree_from_nt(nonterm, len);
        self.fuzzer
            .run_on_with_dedup(&tree, ExecutionReason::Gen, &self.ctx)?;
        Ok(())
    }
    #[allow(dead_code)]
    pub fn inspect(&self, input: &QueueItem) -> String {
        return String::from_utf8_lossy(&input.tree.unparse_to_vec(&self.ctx)).into_owned();
    }
}
