# Project-Level Environment Configuration

This document explains how to use the project-level configuration feature in enman, allowing you to specify the versions of various tools your project requires directly in your repository.

## Overview

The project-level configuration feature enables teams to define which versions of tools their project depends on using a configuration file. This ensures consistent development environments across all team members.

## Configuration File Format

The configuration file uses TOML format and defines tools under a `[tools]` section:

```toml
[tools]
node = "18.17.0"
python = "3.11.5"
java = "17.0.8"
mysql = "8.0.33"
```

By default, enman looks for a file named `.enmanrc` in the current directory, but you can specify a custom path with the `-f` flag.

## Commands

### Initialize Configuration

To create a new configuration file interactively:

```bash
enman config init
```

This creates a `.enmanrc` file with the basic structure.

### Show Current Configuration

To display the tools and versions defined in your configuration:

```bash
enman config show
```

This command parses the configuration file and lists all tools with their specified versions.

### Apply Configuration

To install and switch to the versions specified in your configuration:

```bash
enman config apply
```

This command will:
1. Read the configuration file
2. For each tool-version pair:
   - Check if the version is already installed
   - If not installed, download and install it using the standard enman installation process
   - Switch to that version using the `use` command logic
3. Provide feedback on the process

## Example Configuration Files

### Basic Web Development Setup

```toml
[tools]
node = "18.17.0"
npm = "9.6.7"
python = "3.11.5"
```

### Full Stack Application

```toml
[tools]
node = "18.17.0"
python = "3.11.5"
java = "17.0.8"
mysql = "8.0.33"
redis = "7.0.12"
```

## Benefits

- **Consistency**: Ensures all team members use the same tool versions
- **Onboarding**: New team members can get the correct environment with one command
- **Reproducibility**: Builds and tests behave consistently across different machines
- **Version Control**: Track your environment requirements alongside your code
- **Clean Output**: Tools run with clean output without extra debug information

## Integration

The configuration system integrates with the rest of enman's functionality:
- Uses the same download and installation mechanisms
- Works with both `use` and `global` commands
- Respects existing enman paths and shims
- Provides transparent command interception without adding debug messages

## Supported Tools

Currently supports all tools that enman supports, including:
- Node.js and associated npm/yarn
- Python
- Java/JDK
- MySQL
- And more as added to enman