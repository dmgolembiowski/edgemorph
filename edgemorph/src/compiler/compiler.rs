/// Continue converting https://github.com/edgedb/edgedb/blob/7bbc7b8c0193308850fa7e1154b304aaae06e902/edb/common/compiler.py
/// to Rust once I've figured out valid ellisions for
/// line 98's incredibly confusing ownership relationship,
/// which reads: `level._stack = cast(CompilerContext[ContextLevel], self)`
/// where `level` is `self.ContextLevelClass(prevlevel: T, mode: Any)`
use std::collections::HashMap;
use std::ops::AddAssign;
use regex::{Regex, Match};

#[derive(Debug)]
struct Count(u32);

impl Default for Count {
    fn default() -> Count{
        Count(0 as u32)
    }
}
impl AddAssign for Count {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

#[derive(Debug)]
struct AliasGenerator {
    counts: HashMap<String, Count>
}

trait SimpleCounter {
    fn new() -> Self;
    fn next_val(&mut self, name: &str) -> u32;
}

impl SimpleCounter for AliasGenerator {
    
    fn new() -> AliasGenerator {
        AliasGenerator { counts: HashMap::<String, Count>::new() }
    }

    fn next_val(&mut self, name: &str) -> u32 {
        let Self { counts } = self;
        counts.entry(name.to_string()) //.to_owned())
            .and_modify(|e| { *e += Count(1 as u32) })
            .or_insert(Count(1 as u32)).0
    }
}

impl AliasGenerator {
    pub fn get(&mut self, hint: Option<&str>) -> String {
        let mat: Option<Match>;
        let index: u32;
        let alias: String;
    
        if let Some(mut alias_hint) = hint {
            // I don't know for sure, but I suspect this 
            // is needed to match sequences resembling:
            //      ~010000~010001~010010~010011~010100~010101~010110
            // so that it can take the very last one: (~010110)
            mat = Regex::new(r"~\d+$")
                    .unwrap()
                    .find(&alias_hint);

            match mat {
                Some(m) => {
                    alias_hint = &alias_hint[m.start()..];
                    index   = self.next_val(&alias_hint);
                    alias = format!("{hint}~{index}", hint=alias_hint, index=index);
                    return alias;
                },
                None => {
                    index = self.next_val(&"");
                    alias = format!("{hint}~{index}", hint=alias_hint, index=index);
                    return alias;
                }
            }
        }
        format!("v~{index}", index=1 as u32)
    }
}

trait CompilerCtx<'ctx, T, M> 
    where T: impl CtxLevel,
          M: impl Mode
{
    fn new() -> CompilerCtx<'ctx, T>;
    fn push(&'ctx mut self, mode: Option<M>, prev_level: Option<T>) -> T;
    fn _push(&'ctx mut self, mode: Option<M>, prev_level)
}

struct CompilerContext<'lv, Lv, T> {
    stack: Box<Vec<T>>,
    level: &'lv Lv,

}

impl<Lv> CompilerContext<Lv> {
    pub fn new() -> CompilerContext {
        
    }
}

trait CtxLevel;

struct ContextLevel<'ctx, 'lv> {
    _stack: &'ctx CompilerContext<'lv> 
}

trait CtxSwitchMode;
trait TransactionMode;
trait QueryMode;
trait SingletonMode;
trait DescriptiveMode;

