package main

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

type Config struct {
	Name  string
	Count int
}

func loadConfig() (Config, error) {
	path, err := configPath()
	if err != nil {
		return Config{}, err
	}

	file, err := os.Open(path)
	if err != nil {
		if os.IsNotExist(err) {
			return Config{}, nil
		}
		return Config{}, err
	}
	defer file.Close()

	var config Config

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		key, value, found := strings.Cut(line, "=")
		if !found {
			continue
		}

		switch strings.TrimSpace(key) {
		case "name":
			config.Name = strings.TrimSpace(value)
		case "count":
			parsed, err := strconv.Atoi(strings.TrimSpace(value))
			if err != nil {
				return Config{}, fmt.Errorf("invalid count in config: %w", err)
			}
			config.Count = parsed
		}
	}

	if err := scanner.Err(); err != nil {
		return Config{}, err
	}

	return config, nil
}

func configPath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}

	return filepath.Join(home, ".config", "{{ cookiecutter.project_slug }}", "config.env"), nil
}
