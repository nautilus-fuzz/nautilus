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

use snafu::{Backtrace, Snafu};

use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum SubprocessError {
    #[snafu(display("Could not handle qemu trace file to {} {}", path.display(), source))]
    ReadQemuTrace {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Could not parse integer in {} {}", line, source))]
    ParseIntQemuTrace {
        line: String,
        source: std::num::ParseIntError,
    },

    #[snafu(display("Could not parse line {}", line))]
    ParseLineQemuTrace { line: String, backtrace: Backtrace },

    #[snafu(display("Qemu did not produce any output"))]
    NoQemuOutput { backtrace: Backtrace },

    #[snafu(display("Could not communicate with QemuForkServer {} {} ", task, source))]
    QemuRunNix { task: String, source: nix::Error },

    #[snafu(display("Could not communicate with QemuForkServer {} {} ", task, source))]
    QemuRunIO {
        task: String,
        source: std::io::Error,
    },

    #[snafu(display("Could not disassemble {}", task))]
    DisassemblyError { task: String, backtrace: Backtrace },
}
