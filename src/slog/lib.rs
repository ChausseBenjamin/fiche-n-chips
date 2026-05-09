use attr::*;
use logvalue::*;

mod attr;
mod context;
mod level;
mod logvalue;
mod record;

// tmp contains example code, I don't want it compiled in the binary...
#[cfg(test)]
mod tmp;
