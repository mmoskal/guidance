from guidance._rust.guidancerust import engine_start, Engine, TokenMask
from guidance import select, capture, gen
from dataclasses import dataclass, field
import json


@dataclass
class Splice:
    # If one of the tokens in when_sampled is sampled, this sequence is appended.
    # When empty, this sequence is appended unconditionally, regardless of sampling.
    ff_tokens: list[int]
    # Backtrack this much before appending this sequence (this includes sampled token if any).
    when_sampled: list[int]
    # Append these tokens after backtracking.
    backtrack: int


@dataclass
class Branch:
    mask: TokenMask | None = None
    splices: list[Splice] = field(default_factory=list)


class Engine2:
    def __init__(self, tokenizer_name, grammar):
        self.engine = Engine(tokenizer_name, grammar.serialize())

    def process(self, backtrack: int, tokens: list[int]):
        token_sets, res_str = self.engine.mid_process(backtrack, tokens)
        r = json.loads(res_str)
        if len(r["branches"]) == 0:
            return None  # stop
        assert len(r["branches"]) == 1, "forking not expected"
        b: dict = r["branches"][0]
        rb = Branch()
        mask_idx = b.get("sample_mask", None)
        if mask_idx is not None:
            rb.mask = token_sets[mask_idx]
        for sp in b["splices"]:
            rb.splices.append(
                Splice(
                    ff_tokens=sp["ff_tokens"],
                    when_sampled=sp["when_sampled"],
                    backtrack=sp["backtrack"],
                )
            )
        return rb


def main():
    grm = (
        "<joke>Parallel lines have so much in common. It’s a shame they’ll never meet.</joke>\n"
        + "<joke>"
        + capture(gen(regex=r"[A-Z\(].*", stop="</joke>"), "joke")
        + "</joke>\nScore (of 10): "
        + capture(gen(regex=r"\d{1,3}"), "score")
        + "\n"
    )
    e = Engine2("llama", grm)
    r0 = e.process(0, [])
    print(r0)
    tokens = r0.splices[0].ff_tokens
    r1 = e.process(0, tokens)
    print(r1)

    # s = engine_start("fo", "bar", False)
    # print(s)


main()
