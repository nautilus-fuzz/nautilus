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

use nix::sys::wait::WaitStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExitReason {
    Normal(i32),
    Timeouted,
    Signaled(i32),
    Stopped(i32),
}

impl ExitReason {
    #[must_use]
    pub fn from_wait_status(status: WaitStatus) -> ExitReason {
        match status {
            WaitStatus::Exited(_, return_value) => ExitReason::Normal(return_value),
            WaitStatus::Signaled(_, signal, _) => ExitReason::Signaled(signal as i32),
            WaitStatus::Stopped(_, signal) => ExitReason::Stopped(signal as i32),
            _ => panic!("{}", "Unknown WaitStatus: {status:?}"),
        }
    }
}
