from pytest_tracer_python.code_stub import f_stub


def main() -> None:
    print(f"Hello, World! {f_stub(2)}")


if __name__ == "__main__":
    main()
