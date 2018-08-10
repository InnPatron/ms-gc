#[macro_use]
extern crate ms_gc;

#[cfg(test)]
mod circular;
#[cfg(test)]
mod simple_cleanup;

use std::alloc::System;
#[global_allocator]
static GLOBAL: System = System;

fn main() { }
