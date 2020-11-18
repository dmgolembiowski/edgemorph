#!/usr/bin/env python3

"""
This is a Python prototype of the EDM compiler.
To quickly iterate on the design of `edm`, this pseudo-compiler
bootlegs the official EdgeDB lexer to validate SDL grammar.
"""
import argparse
import heapq
import multiprocessing
import os
import re
import sys
from copy import copy
from pathlib import Path
from time import sleep
from typing import Any, Callable, List, Optional, Union
import json
import toml
from edb.errors import EdgeQLSyntaxError
from edb.common.markup import _serialize as serialize
from edb.edgeql import parser as qlparser
import pprint
from functools import wraps
from random import choice
from yaspin import yaspin
import time
from edm.source import SDLSource

__datastore__ = {}


class Ok:
    """Ok

    This class emulates the Ok slice in Rust's `Result` enum.
    """

    def __init__(self, *args, **kwargs):
        self.args = args
        self.__dict__.update(kwargs)


Result = Union[Ok, Exception]


class IOResult:
    def __init__(self, found: bool, annotation: Optional[str] = None) -> None:
        self.annotation: Optional[str] = annotation
        self.found: bool = found

'''
TODO: This section should be moved to its own file.
'''

def red(s: str) -> str:
    return f"\033[91m{s}\033[0m"

def yellow(s: str) -> str:
    return f"\033[33m{s}\033[0m"

def spin_to_win(function):
    """spin_to_win(function)

    This function is a graphical context manager.
    It runs some terminal-bound task in a `yaspin`
    context, and supplies visual feedback to the user.
    """
    on_success = "✔"
    on_failure = red("✖")
    options = (
        "Recombobulating discombobulators...\n",
        "Initiaiting rocket launch...\n",
        "Normalizing the vectors...\n",
        "Calculating loss values...\n",
        "Brewing imaginery tea/coffee\n",
        "Proving the Riemann Hypothesis...\n",
        "Performing a topological sort...\n",
        "Fending off volatile hackers...\n",
        "[ Elevator Music Playing ]\n",
    )
    
    @wraps(function)
    def terminal_spinner(*args, **kw):
        
        nonlocal on_success
        nonlocal on_failure
        nonlocal options
        
        text_to_display: str
        color: str

        if "text" not in kw.keys():
            text_to_display = choice(options)
        else:
            text_to_display = kw["text"]

        if "color" not in kw.keys():
            color = "cyan"
        else:
            color = kw["color"]

        with yaspin(text=text_to_display, color=color) as sp:
            some_result: Optional[any]
            try:
                some_result = function(*args, **kw)
                sp.write(hype_str(f"{on_success} The task was successful.\n"))
                
                return some_result
            except Exception as e:
                
                sp.write(on_failure + " could not be completed.")
                raise(e)

    return terminal_spinner

def init(loc: str = ".") -> Result:
    """init

    This function initializes an edgemorph project.
    For the full explanation, see
    https://github.com/dmgolembiowski/edgemorph/tree/master/edm#formal-specification

    PARAMETERS
    - loc: Optional[str] = '.'
        This parameter is the location on the filesystem where edgemorph
        projects can be initialized. The default value is the current
        working directory.

    RETURN
    - Union[Ok, Exception]: The result of the project creation command.
    """
    # Initializing some variables
    project_root: str
    path: Path
    exit_status: Union[Path, Exception]
    loc = loc.rstrip("/")

    # Create the project directory
    if (path := Path(loc)).exists():
        pass
    else:
        path.mkdir(parents=True, exist_ok=True)

    # Need to check this directory for an `edgemorph.toml`
    edm_conf = check_for_edgemorph(path)
    if edm_conf.found:
        # Prefer to not dump a bunch of traceback
        print(red(f"Error: Project already exists in {path.resolve()}"))
        sys.exit(1)

    # Create edgemorph-framework files and directories
    project_dir = path.absolute()
    project_root = project_dir.stem
    project_dir = str(project_dir)

    # Starting with `edgemorph.toml`
    sys.stdout.write("\033[;1m")
    print("Initializing your new Edgemorph project!")
    schema: str = input(
        "Enter your project's schema name (or default to `Edgemorph`): "
    )

    # Use with a map in the Rust verison
    if not schema:
        schema = "Edgemorph"

    # Prepare edgemorph.toml's content
    file_content: str = build_toml(project_root, schema)
    with open(project_dir + "/edgemorph.toml", "w") as f:
        f.write(file_content)

    # Make the default modules/output folder(s)
    Path(f"{project_dir}/edb_modules").mkdir()
    Path(f"{project_dir}/edm_{project_root}").mkdir()

    # Make the default SDL module file
    with open(f"{project_dir}/edb_modules/{project_root}.esdl", "w") as f:
        stream = f"""module {project_root} {{

}}"""
        f.write(stream)
    
    # Initialize a credentials file for the client
    cred_file = f"{project_dir}/credentials.json"
    init_credentials_file(cred_file)
    
    hype_print(f"Success! Your project was created at {project_dir}")

def hype_print(msg: str):
    sys.stdout.write("\033[;1m")
    print(msg)

def hype_str(msg: str):
    sys.stdout.write("\033[;1m")
    return msg

def hype_input(msg: str):
    sys.stdout.write("\033[1;m")
    response = input(msg)
    return response

def init_credentials_file(loc: str):
    
    """init_credentials_file(loc: str)

    This function interactively creates the `credentials.json`
    file. `loc` is expected to be the absolute path to the
    JSON file to be written by this procedure.
    """

    # Called at the end for saving credentials
    def save_credentials_file(path: str, creds: dict):
        with open(path, "w") as cred_file:
            json.dump(creds, cred_file, indent=2)

    # Effective start point begins here
    msg = (
        "Do you want to populate a `credentials.json` file now? " 
        "[ (Y)es | (N)o {default} | (S)kip ]: "
    )
    
    build_cred: bool

    while True:
        resp = hype_input(msg)
        resp = resp.lower()
        if resp in set(("y", "1", "yes")):
            build_cred = True
            break
        elif resp in set(("n", "0", "no", " ", "")):
            build_cred = False
            break
        elif resp in set(("skip", "s")):
            print("INFO: Edgemorph may not function properly without credentials.")
            return
        else:
            print(red(f"ERROR: Response {resp} not understood."))
            continue
    
    def generate_template():
        return dict([
            ("port", "5656"),
            ("user", "edgedb"),
            ("password", ""),
            ("database", "edgemorph")
        ])

    base_template = generate_template()

    if build_cred:
        while True:
            # intentional copy here
            base = generate_template()
            for (key, value) in base.items():
                resp = hype_input(f"Enter the value for `{key}` (default: `{value}`): ")
                if resp == "":
                    resp = value
                base[key] = resp
            
            port: int
            
            try:
                port = int(base.get("port"))
            except:
                port = 5656

            verify_msg: str = (
                "Is this correct?\n"
                f"{pprint.pformat(str(base), indent=2)}\n"
                "[ (Y)es | (N)o ]: "
            )

            resp = hype_input(verify_msg)
            if resp.lower() not in set(("yes", "y")):
                continue
            else:
                save_credentials_file(loc, base)
                return
    else:
        base_template["port"] = int(base_template["port"])
        save_credentials_file(loc, base_template)
        hype_print("Saving default credentials file...")
        return
    
def glob_paths(cwd: Path, rel_path: Path) -> Path:
    return cwd.joinpath(*[inode for inode in rel_path.parts if inode not in cwd.parts])


def make(target: str):
    files: Box = find_edgemorph_toml()
    path: Path = files.unwrap().pop()[1].loc
    edm_toml = load_edgemorph_toml(path)
    try:
        assert edm_toml is not None
    except AssertionError:
        print(red("ERROR: `edgemorph.toml` not found."))
        sys.exit(1)

    """
    First order of business is to case out the argument supplied to `make`;
    if it's (*), then `make` all of the modules.

    These are determined by recursively exploring `edgedb.databases.[X].modules`.

    Otherwise, perform a single `make` on `target` = `Y` where `Y` is valid for
    `edgedb.databases.[X].modules.[Y].

    When `target` == '*', get all `*.esdl` files in `edgemorph.toml` and
    1) check for their existence on the filesystem
    [1) compare the diffs of of files for each extant on the filesystem]
    2) if any are missing, note it, but do not exit
    3) do: parse -> (analyze) -> compile for the available paths
    [3) Recompile/Analyze only those that have changed.]

    When `target` == `mod_<something>.esdl`, or simply `mod_something`,
    we search for it in `edm_toml["edgedb"]["databases"][X]["modules"]
    where `X` is any of the valid database names.

    Note: ToDo items are enclosed in square brackets "[" and "]".
    These are future enhancements that will involve incremental compilation,
    rather than the single-pass implementation.
    """
    # pprint(edm_toml)
    start_path = Path(path).resolve().parent

    modules: List[Path] = []
    for mod_path in edm_toml["edgedb"]["databases"]["primary"]["modules"].values():
        mod_path = Path(mod_path)
        modules.append(start_path / mod_path.relative_to(mod_path.anchor))

    # Allocate a list/mutable vector of matching `.esdl` files
    retrieved: list[str] = []

    # For Rust, allocate a `String` of the current working directory
    # and pass a borrow to `glob_paths`
    cwd = Path(os.getcwd())
    matched: Optional[str]
    untracked: [str] = []
    broken_refs: [str] = []
    if target[0] == "*" and len(target) == 1:
        # In Rust, do if-let/match here instead
        for rel_path in modules:
            matched = glob_paths(cwd, rel_path)
            if matched is not None:
                retrieved.append(matched)
            else:
                print(f"rel_path = {rel_path}")
        """
        If we've reached this 'else' block, it implies 1 of 4 scenarios:
          1) target[0] = ['something.esdl']
          2) target[0] = ['edb_modules/something.esdl', 'something_else.esdl', ...]
          3) target[0] = ['../../way_up.esdl', ...]
          4) target[0] = ['.', ...]
        Cases (1) and (2) are no problem. We can basically replicate the above logic
        with another for loop. Case (3) poses some challenges because `glob_paths`
        will not resolve relative paths and I'd rather not increase its complexity.

        To resolve Case (3), there needs to be some additional checking to ensure
        `glob_paths` will always return the absolute path.
        """
    elif "." not in set(target):
        # modpath: Union[Path, str]
        for modpath in modules:
            if not modpath.exists():
                # edgemorph.toml has it but the local filesystem doesn't
                broken_refs.append(str(modpath))
                continue
            else:
                # `modpath` does exist in `edgemorph.toml`, but
                # does it also match a destination on the filesystem?

                match_set: tuple[str] = tuple()
                extant_path: Union[Path, str]
                for some_path in target:
                    if (extant_path := Path(some_path)).exists():
                        extant_path = extant_path.resolve()
                        if extant_path == modpath and extant_path not in match_set:
                            match_set += (extant_path,)
                for mat in iter(set(match_set)):
                    retrieved.append(mat)
                if modpath not in set(retrieved):
                    untracked.append(modpath)

    elif target[0] == "." and len(target) == 1:
        # Get all .esdl files in the current working directory
        # and add them to `retrieved`.

        # modpath: Union[Path, str]
        for modpath in modules:
            if not (modpath := Path(rel_path)).exists():
                # edgemorph.toml has it but the local filesystem doesn't
                broken_refs.append(str(modpath))
                continue
            else:
                # `modpath` does exist in `edgemorph.toml`, but
                # does it also match a destination on the filesystem?

                modpath = str(modpath.resolve())
                match_set: tuple[str] = tuple()
                extant_path: Union[Path, str]
                some_path = target[0]  # == "."
                if (extant_path := Path(some_path)).exists():
                    extant_path = extant_path.resolve()
                    if extant_path == modpath and extant_path not in match_set:
                        match_set += (extant_path,)
                # How can this be improved without the use of `try`:
                # ... `except IndexError:`?
                for mat in iter(set(match_set)):
                    retrieved.append(mat)
                if modpath not in set(retrieved):
                    untracked.append(modpath)
    else:
        # Print an error message suggesting that the user
        # needs to run `edm add` to register module files
        # then sys.exit before reaching the end of this scope
        print("HELP: You supplied `edm make [A|.] [*|B] ....`.")
        print("HELP: Try reducing the complexity to simpler statements: `edm make <X>`")
        sys.exit(1)

    # If `broken_refs`, then raise a critical warning that
    # missing .esdl module file names were supplied, which are being tracked
    # but they are not available on the filesystem
    if broken_refs:
        msg = (
            "CRITICAL WARNING: "
            "One or more EdgeDB module files are missing from the local filesystem!\n"
            "These include:\n"
        )
        for broken in broken_refs:
            msg += f" + {broken}\n"

        msg += (
            "If this does not seem correct, please double check `edgemorph.toml` "
            "and remove any unused entries before running this command again."
        )
        print(red(msg))

    # If `untracked`, mention running `edm add`
    if untracked:
        msg = (
            "ERROR: Cannot run `make` command on untracked EdgeDB module files. "
            "Please try again after running:"
        )
        print(red(msg) + "\n")
        for filename in untracked:
            print(f" -> edm add {filename}")

    # Finally, the moment of truth, calling `compile` on these paths
    if retrieved:
        batch_compilation(retrieved)
    sys.exit()


def batch_compilation(
        module_paths: List[str], 
        text="Compiling your EdgeDB modules..."):

    poolsize: int = len(module_paths)
    async_pool = multiprocessing.Pool(processes=poolsize)
    timed_out: bool = False
    results = []
 
    with async_pool as pool:
        async_jobs = [pool.apply_async(compile, module_paths) for i in range(poolsize)]
        try:
            async_res = [job.get(timeout=1.7) for job in async_jobs]
            results = copy(async_res)
        except multiprocessing.context.TimeoutError as e:
            print(yellow("ERROR: Please correct any errors and try re-compiling."))
            timed_out = True
    if timed_out:
        sys.exit(1)

    for i in range(poolsize):
        
        """
        Assume that paths are given as:
        `/path/to/(edb_modules/repositories)/model.esdl`
        so keeping the folder constant, we modify the
        destination as `"." + f"{filename}" + ".desastr"`
        """

        # This condition can technically happen
        # if the `compile` call returns `None` instead
        # of a proper AST
        try:
            assert results[i] is not None
        except AssertionError:
            print(red(f"ERROR: {module_paths[i]} could not be compiled."))
            continue

        # Otherwise, we continue with confidence

        ast_src  = module_paths[i]
        folder   = str(ast_src.parent) 
        filename = "." + ast_src.name.rstrip(".esdl") + ".desastr"
        ast_path = folder + "/" + filename
        sys.stdout = open(ast_path, "w")
        results[i].dump()
        sys.stdout.close()

class Box:
    def __init__(self, ty: Any):
        self.ty = ty

    def unwrap(self):
        if self.ty is not None:
            return self.ty
        else:
            raise TypeError(
                "This is a bug. Please report this at "
                "https://github.com/dmgolembiowski/edgemorph/issues"
            )


def memcache(key: str, value: Optional[Box]) -> Optional[Box]:
    """Uses the `__datastore__` for allocating
    `Box` instances."""
    if key not in __datastore__ and value is not None:
        __datastore__[key] = value
    elif value is None:
        try:
            return __datastore__[key]
        except:
            return None
    return __datastore__[key]


class Entry:
    def __init__(self, loc: Path) -> None:
        self.loc: Path = loc

    def __str__(self) -> str:
        return f"{str(self.loc.resolve().absolute())}"


def find_edgemorph_toml(depth_limit: int = 3) -> Box:
    """find_edgemorph_toml

    This function scans the surrounding neighborhood
    for a potential configuration file. It imagines that
    the user will try to invoke an edm command
    in one of the possible locations:
    - The top level project directory `project_root`;
        .
        ├── Cargo.toml
        ├── edb_modules
        │   ├── backend
        │   │   ├── mod_databases.esdl
        │   │   ├── mod_load_balancer.esdl
        │   │   └── mod_network.esdl
        │   ├── frontend
        │   │   ├── mod_api_v1.esdl
        │   │   └── mod_api_v2.esdl
        │   └── mod_app.esdl
        ├── edgemorph.toml
        ├── edm_app
        └── src
            └── main.rs

    - Under `edb_modules` or any of its branches:
        .
        ├── backend
        │   ├── mod_databases.esdl
        │   ├── mod_load_balancer.esdl
        │   └── mod_network.esdl
        ├── frontend
        │   ├── mod_api_v1.esdl
        │   └── mod_api_v2.esdl
        └── mod_app.esdl

    - Or under one of the two directories where native codegen
      files are written:
        ├── edm_app
        └── src
            └── main.rs

    Using this heuristic, this function will check in the order of
    0 levels higher, 1 level higher, and then 2 levels higher for a
    file named `edgemorph.toml`. Other checks for corner cases are
    implemented as well.
    """
    # Try lazily loading this first
    stored: Box
    if (stored := memcache("edgemorph.toml", None)) is not None:
        return stored

    # Initialize some variables
    dirs_walked: int = 0
    path: Path
    found = []
    entry: Entry

    # A singleton type for prepending relative paths
    class Prefix:
        def __init__(self, initial_mark: str = "") -> None:
            self.mark: str = initial_mark

        def prepend(self) -> None:
            self.mark = "../" + self.mark

        def __str__(self) -> str:
            return self.mark

    prefix: Prefix = Prefix()

    def traverse(toml: Path) -> Union[Callable, None]:
        nonlocal depth_limit
        nonlocal dirs_walked
        nonlocal found
        nonlocal entry
        nonlocal prefix
        path: Path
        if depth_limit - dirs_walked > 0:
            path = Path(str(prefix) + str(toml))
            if path.exists():
                entry = Entry(path)
                heapq.heappush(found, (depth_limit - dirs_walked, entry))
            dirs_walked += 1
            prefix.prepend()
            return traverse(toml)
        else:
            return None

    # Prepare a priority queue of the possible `edgemorph.toml` locations.
    # `Entry`s are ranked by an inverse proximity score.
    traverse(Path("./edgemorph.toml"))

    # For some additional checks
    pref: (int, Entry)
    slice_idx: int
    path_idx: Union[int, str]

    try:
        pref = heapq.heappop(found)
    except IndexError:
        print(red("ERROR: `edgemorph.toml` not found!"))
        print(f"HELP: Try running {red('edm init <project_name>')}\n")
        sys.exit(1)

    # Being explicit about clones that
    heapq.heappush(found, copy(pref))

    if len(found) > 1:
        msg: str = f"Warning: Multiple {red('edgemorph.toml')} files were detected:"
        print(msg + "\n")
        print(" ID  Files")
        print(" --  -----")
        allowed: set[int] = set()
        for slice_idx in range(len(found)):
            allowed.add(slice_idx + 1)
            # ToDo: Impove this line's readability
            allowed_path = found[slice_idx][1].loc.resolve().absolute()
            print(f"  {slice_idx + 1}) {allowed_path}")
        while True:
            path_idx = input("\nPlease enter the appropriate ID number: ")
            try:
                path_idx = int(path_idx)
                if path_idx not in allowed:
                    continue
            except ValueError:
                continue
            try:
                pref = found[path_idx - 1]
            except IndexError:
                continue
            break

    # Store this in a cache for later retrieval
    stored = memcache("edgemorph.toml", Box([pref]))

    return stored


def build_toml(project_root: str, schema: str) -> str:
    return f"""[edgemorph]
project_root    = "{project_root}"
mod_directories = ["/edb_modules"]

[edgemorph.codegen]
schema_name = "{schema}"

[edgemorph.codegen.rust]
enabled = "true"

[edgemorph.codegen.rust.modules]
    [edgemorph.codegen.rust.modules.{project_root}]
    source = "/edb_modules/{project_root}.esdl"
    output = "/src/lib/edm_{project_root}.rs"

[edgemorph.codegen.python]
enabled = "true"

[edgemorph.codegen.python.modules]
    [edgemorph.codegen.python.modules.{project_root}]
    source = "/edb_modules/{project_root}.esdl"
    output = "/{project_root}/edm_{project_root}.py"

[edgedb]
[edgedb.databases]
[edgedb.databases.primary]
name = ""
dsn = ""

[edgedb.databases.primary.modules]
{project_root} = "/edb_modules/{project_root}.esdl"

"""


def load_edgemorph_toml(path: Path) -> Optional[dict]:
    edm_toml: dict
    try:
        with open(path.resolve().absolute(), "r") as f:
            edm_toml = toml.load(f)
    except FileNotFoundError:
        return None
    return edm_toml


def check_for_edgemorph(path: Path) -> IOResult:
    exit_status: IOResult
    if Path(str(path) + "/edgemorph.toml").exists():
        exit_status = IOResult(found=True)
        return exit_status
    else:
        exit_status = IOResult(found=False)
    return exit_status


def make_install(args):
    print("Running make install....")


def add(args):
    print("Running add.....")

@spin_to_win
def compile(args, text="Compiling your EdgeDB SDL source modules... "):
    
    if not isinstance(args, (str, Path)):
        print("ERROR: Method of compilation not yet implemented. Exiting.")
        return None
    else:
        # Assuming args to be the file path
        try:
            with open(args, "r") as f:
                source = f.read()
        except FileNotFoundError:
            print(red("ERROR")+f": `{args}` is not available.")
            sys.exit(1)
        # In progress: more helpful error messages
        # when parsing the source of the SDL fails
        syntax_lex = helpful_parsing(source)
        return syntax_lex

def helpful_parsing(source: str) -> Optional[Any]:
    """helpful_parsing(source: str) -> Optional[Any]:
    
    When EdgeDB 1-alpha was released to the public,
    error reporting was not as sophisticated as it
    is now, but nevertheless I am adding this functionality
    to edgemorph because I wanted it back then
    and did not have it.

    This function helps you debug your EdgeDB SDL module
    files without needing to connect to the database.
    By accessing `col`, `position`, and `line` on the `err`
    argument, we can format a new error message
    to indicate where the EdgeDB syntax error is occuring
    with a style similar to the one found on the database CLI.
    """
    try:
        syntax_lex = qlparser.parse_sdl(source)
        return syntax_lex
    except EdgeQLSyntaxError as err:
        print("\n"+red("ERROR")+": Syntax correction(s) needed here\n")
        formatted_err = str(SDLSource(source, err))
        print(formatted_err)
        sys.exit(1)

def test(args):
    print("Testing connectivity...")


def main(args: argparse.Namespace):
    # ToDo: Replace this with `match - case` from PEP 622
    try:
        func_name = next(iter(vars(args)))
    except StopIteration:
        return

    # Extract the necessary destinations and arguments
    if func_name == "make":
        if vars(args).get("make") == ["install"] or "make_install" in vars(args):
            func_name = "make_install"

    arg = vars(args).get(func_name)

    # This can be made more secure by enforcing known
    # function targets. For example:
    permitted = {"init", "add", "make", "compile", "test", "make_install"} 
    if func_name in permitted:
        eval(f"{func_name}('{arg}')")
    else:
        msg: str = "\033[91m" + f"{func_name} is not available." + "\033[0m"
        print(msg)
        sys.exit(1)


def usage():
    return """usage: edm { positional argument } { argument value }

positional arguments:
    help                shows this message
    init                initializes a new edgemorph project
    add                 creates a new module file and/or updates `edgemorph.toml`
    make                high-level utility for AST checking and generating artifacts
    make install        migrate the schema to an EdgeDB instance
    compile             low-level utility for AST checking
    test                test connectivity to an `edgemorph.toml` registered database

argument values:
    init                [ directory_name | . ]
    add                 [ new_module ]
    make                [ edb_module | * ]
    make install        [ (edb_module)+ | * ]
    compile             [ edb_module_path ]
    test                [ database_name ]

"""


def cli_main():
    # This shim may be a source of errors in the future.
    # Currently, it is responsible for allowing the argument
    # parser to allow argument names to not use dashed names
    lastarg = sys.argv[-1]
    if len(sys.argv) > 1 and lastarg[0] != "-":
        sys.argv[-1] = "-f"
        sys.argv.append(lastarg)

    # The primary argument parser
    edm_parser = argparse.ArgumentParser(prog="edm", usage=usage())

    # Auxillary parsers beneath `edm_parser`
    subparser = edm_parser.add_subparsers()
    edm_init = subparser.add_parser("init")
    edm_add = subparser.add_parser("add")
    edm_make = subparser.add_parser("make")
    make_subparsers = edm_make.add_subparsers()
    edm_make_install = make_subparsers.add_parser("install")
    edm_compile = subparser.add_parser("compile")
    edm_test = subparser.add_parser("test")

    # `edm init`
    edm_init.add_argument("-f", dest="init", default=".", metavar="PROJECT_DIR")

    # `edm add`
    edm_add.add_argument("-f", dest="add", metavar="MODULE_FILE")

    # `edm make`: '.' means ./*.esdl ; '*' means **/*.esdl
    edm_make.add_argument("-f", default="*", dest="make", nargs="*", metavar="SOURCE")

    # `edm make install`
    edm_make_install.add_argument(
        "-f",
        dest="make_install",
        default="*",
        nargs="*",
        metavar="SOURCE",
    )

    # `edm compile`
    edm_compile.add_argument("-f", dest="compile", metavar="SOURCE")

    # `edm test`: '*' means all database connections in edgemorph.toml
    edm_test.add_argument(
        "-f",
        dest="test",
        nargs="*",
        metavar="MODULE DATABASE_IDENTITY",
    )

    args, _ = edm_parser.parse_known_args()
    main(args)
