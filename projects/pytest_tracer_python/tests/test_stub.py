from pytest_tracer_python.code_stub import f_stub


def test_stub() -> None:
    assert True


def test_f_stub() -> None:
    assert f_stub(2) == 3
