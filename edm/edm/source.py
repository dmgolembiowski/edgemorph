from edb.errors import EdgeQLSyntaxError

class SDLSource:
    def __init__(self, source: str, err: EdgeQLSyntaxError):
        self.source   = source
        self.err      = err
        self.friendly = ""

        # `self.newlines` helps us apply any vertical/horizontal
        # shifts to the help message in a convenient way

        self.newlines = [
                i 
                for i in range(len(source)) 
                if source[i] == "\n"
        ]

        self.array = [list(line) for line in source.splitlines()]

        # Add the underline section
        self._insert_underline()

    def __str__(self):
        entire = self.header + "\n" + self.friendly
        return entire

    @property
    def header(self):
        red = lambda chars: f"\033[091m{chars}\033[0m"
        return red(f"error: {self.err.args[0]}\n")

    def _insert_underline(self):
        '''
        Using `self.err.line` and `self.err.column`
        we discern where to inject a new charlist.
        '''

        red = lambda chars: f"\033[091m{chars}\033[0m"
        
        start_line: int = self.err.line
        start_pos: int = self.err.col
        end_pos:   int = 0
        underline: str = ""

        # Prepare a copy of the `self.array` container
        arr = iter(self.array)
        idx: int = 1
        
        for line in arr:
            # line is a dynamically-sized list of single characters
            if idx == start_line:
                for char_idx in range(len(line)):
                    if char_idx < start_pos - 1:
                        underline += " "
                        continue
                    if line[char_idx] == " ":
                        underline += " error\n"
                        break
                    elif char_idx == len(line):
                        underline += " error\n"
                        break
                    else:
                        underline += "^"

            else:
                idx += 1
        
        line = self.err.line
        err_lint = red(underline)
        self.array[line].insert(0, list(err_lint))

        for i in range(len(self.array)):         
            for j in range(len(self.array[i])):
                self.friendly += ''.join(self.array[i][j])
            self.friendly += "\n"

