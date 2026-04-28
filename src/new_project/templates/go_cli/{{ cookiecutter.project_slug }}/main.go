package main

import (
	"flag"
	"fmt"
	"os"
)

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	switch os.Args[1] {
	case "greet":
		if err := runGreet(os.Args[2:]); err != nil {
			fmt.Fprintf(os.Stderr, "error: %v\n", err)
			os.Exit(1)
		}
	case "config-path":
		if err := runConfigPath(); err != nil {
			fmt.Fprintf(os.Stderr, "error: %v\n", err)
			os.Exit(1)
		}
	case "-h", "--help", "help":
		printUsage()
	default:
		fmt.Fprintf(os.Stderr, "unknown command %q\n\n", os.Args[1])
		printUsage()
		os.Exit(1)
	}
}

func runGreet(args []string) error {
	flags := flag.NewFlagSet("greet", flag.ContinueOnError)
	flags.SetOutput(os.Stderr)

	name := flags.String("name", "", "Name to greet")
	count := flags.Int("count", 0, "Number of greetings to print")

	if err := flags.Parse(args); err != nil {
		return err
	}

	config, err := loadConfig()
	if err != nil {
		return err
	}

	resolvedName := *name
	if resolvedName == "" {
		resolvedName = config.Name
	}
	if resolvedName == "" {
		resolvedName = "World"
	}

	resolvedCount := *count
	if resolvedCount == 0 {
		resolvedCount = config.Count
	}
	if resolvedCount == 0 {
		resolvedCount = 1
	}

	for i := 0; i < resolvedCount; i++ {
		fmt.Printf("Hello, %s!\n", resolvedName)
	}

	return nil
}

func runConfigPath() error {
	path, err := configPath()
	if err != nil {
		return err
	}

	fmt.Println(path)
	return nil
}

func printUsage() {
	fmt.Printf(`{{ cookiecutter.binary_name }} is an example Go CLI.

Usage:
  {{ cookiecutter.binary_name }} greet [-name value] [-count value]
  {{ cookiecutter.binary_name }} config-path
`)
}
