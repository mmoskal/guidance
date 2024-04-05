from guidance._rust.guidancerust import engine_start, Engine
from guidance import select, capture, gen

def main():
    grm = (
        "<joke>Parallel lines have so much in common. It’s a shame they’ll never meet.</joke>\n"
        + "<joke>"
        + capture(gen(regex=r'[A-Z\(].*', stop="</joke>"), "joke")
        + "</joke>\nScore (of 10): "
        + capture(gen(regex=r"\d{1,3}"), "score")
        + "\n"
    )
    b = grm.serialize()
    e = Engine("llama", b)
    print(e)
    s = engine_start("fo", "bar", False)
    print(s)

main()

