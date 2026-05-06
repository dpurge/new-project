from src.main import main


def test_main_prints_greeting(capsys) -> None:
    main()

    captured = capsys.readouterr()

    assert captured.out == "Hello from {{ cookiecutter.project_name }}!\n"
