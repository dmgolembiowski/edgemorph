from edb.errors import EdgeQLSyntaxError
from prompt_toolkit import lexers as pt_lexers
from edb.tools.pygments import edgeql as eql_pygments
from prompt_toolkit import document as pt_document
from prompt_toolkit import styles as pt_styles

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

    @property
    def style(self):
        return pt_styles.Style.from_dict({
            'prompt': '#aaa',
            'continuation': '#888',

            'bottom-toolbar': 'bg:#222222 #aaaaaa noreverse',
            'bottom-toolbar.on': 'bg:#222222 #ffffff',

            # See prompt_tookit/styles/defaults.py for the reference.
            'pygments.name.builtin': '#A6E22E',
            'pygments.punctuation.navigation': '#e8364f',
            'pygments.comment': '#555',
            'pygments.keyword': '#e8364f',
            'pygments.keyword.constant': 'green',
            'pygments.operator': '#e8364f',
            'pygments.literal.string': '#d3c970',
            'pygments.literal.number': '#9a79d7',
            'pygments.key': '#555',
            'pygments.value': '#888',
        })

    def render(self):
        desc_doc = pt_document.Document(self.source)
        lexer = pt_lexers.PygmentsLexer(eql_pygments.EdgeQLLexer)
        formatter = lexer.lex_document(desc_doc)

        for line in range(desc_doc.line_count):
            pt_shortcuts.print_formatted_text(
                pt_formatted_text.FormattedText(formatter(line)),
                style=self.style
            )
        print()

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
                    idx += 1

            else:
                idx += 1
        
        line = self.err.line
        err_lint = red(underline)
        self.array[line].insert(0, list(err_lint))

        for i in range(len(self.array)):         
            for j in range(len(self.array[i])):
                self.friendly += ''.join(self.array[i][j])
            self.friendly += "\n"
