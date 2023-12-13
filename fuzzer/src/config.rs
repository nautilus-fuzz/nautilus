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

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub number_of_threads: u8,
    pub thread_size: usize,
    pub number_of_generate_inputs: u16,
    pub number_of_deterministic_mutations: usize,
    pub max_tree_size: usize,
    pub bitmap_size: usize,
    pub timeout_in_millis: u64,
    pub path_to_bin_target: String,
    pub path_to_grammar: String,
    pub path_to_workdir: String,
    pub arguments: Vec<String>,
    pub hide_output: bool,
    pub extension: String,
}
