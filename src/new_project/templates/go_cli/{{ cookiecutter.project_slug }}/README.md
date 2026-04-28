# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## Features

- small CLI implemented with Go's standard library
- command-specific flag parsing with `flag.FlagSet`
- optional config file loaded from the user's home directory

## Layout

- `main.go`: bootstrap and command dispatch
- `config.go`: optional config loading and config path resolution

## Commands

- `{{ cookiecutter.binary_name }} greet`
- `{{ cookiecutter.binary_name }} config-path`

## Config file

The app looks for an optional config file at:

```text
~/.config/{{ cookiecutter.project_slug }}/config.env
```

Example contents:

```text
name=CLI User
count=2
```

## Run

```bash
go run . greet
```

Override values from the command line:

```bash
go run . greet -name Alice -count 3
```

Print the resolved config path:

```bash
go run . config-path
```
