/*
   This file was originally created by [jam1garner/rust-dyn-call](https://github.com/jam1garner/rust-dyn-call),
   and I'll almost certainly get some retribution for this little stunt.

   That said, Jam's dyn-call macro is a useful way to glue together Edgemorph's test harness by seriously
   diminishing the amounts of boilerplate code to write.
   `dyn_call` allows the dynamic dispatch of function names by providing the names as strings, which we interpolate
   using dtolnay's `quote` crate.
   
    Again, Jam, I really am sorry.
 */

macro_rules! anything_to_nothing {
    ($($tt:tt)*) => { _ }
}

#[macro_export]
macro_rules! dyn_call {
    ( $str:literal ($($arg:expr),* $(,)?)) => {{
        let func: fn($(anything_to_nothing!($arg)),*) -> _ = unsafe { core::mem::transmute(get_sym($str)) };
        func($($arg),*)
    }};
    
    ($name:ident ($($arg:expr),* $(,)?)) => {{
        let func: fn($(anything_to_nothing!($arg)),*) -> _ = unsafe { core::mem::transmute(get_sym($name)) };
        func($($arg),*)
    }};
}

use goblin::Object;

fn get_sym_offset(name: &str) -> usize {
    let argv0 = std::env::args().nth(0).unwrap();
    let executable = std::fs::read(argv0).unwrap();

    match Object::parse(&executable).unwrap() {
        Object::Elf(elf) => {
            let sym = elf.dynsyms.iter().find(|sym| elf.dynstrtab.get(sym.st_name).unwrap().unwrap() == name);
            let sym = match sym {
                Some(sym) => sym,
                None => panic!("Symbol '{}' not found. Be sure you're using #[no_mangle].", name)
            };

            sym.st_value as usize
        }
        _ => todo!("Only linux is supported currently")
    }
}

fn get_sym(name: &str) -> *const () {
    (((indicator as usize) - get_sym_offset("indicator")) + get_sym_offset(name)) as *const ()
}

#[no_mangle] pub fn indicator() {}
